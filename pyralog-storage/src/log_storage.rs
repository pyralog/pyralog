use bytes::Bytes;
use pyralog_core::{LogOffset, Record, RecordBatch, Result, PyralogError, OffsetRange};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::segment::{Segment, SegmentConfig};
use crate::index::Index;
use crate::write_cache::{WriteCache, WriteCacheConfig};

/// Main log storage implementation
pub struct LogStorage {
    base_path: PathBuf,
    segments: Arc<RwLock<Vec<Arc<SegmentWithIndex>>>>,
    write_cache: WriteCache,
    config: LogStorageConfig,
    current_offset: Arc<RwLock<LogOffset>>,
}

struct SegmentWithIndex {
    segment: Segment,
    index: Index,
}

#[derive(Debug, Clone)]
pub struct LogStorageConfig {
    pub segment_config: SegmentConfig,
    pub cache_config: WriteCacheConfig,
}

impl Default for LogStorageConfig {
    fn default() -> Self {
        Self {
            segment_config: SegmentConfig::default(),
            cache_config: WriteCacheConfig::default(),
        }
    }
}

impl LogStorage {
    /// Create a new log storage
    pub async fn create(base_path: PathBuf, config: LogStorageConfig) -> Result<Self> {
        std::fs::create_dir_all(&base_path)
            .map_err(|e| PyralogError::StorageError(e.to_string()))?;

        let segment = Segment::create(
            LogOffset::ZERO,
            &base_path,
            config.segment_config.clone(),
        )?;

        let index = Index::create(segment.path())?;

        let segment_with_index = Arc::new(SegmentWithIndex { segment, index });

        Ok(Self {
            base_path,
            segments: Arc::new(RwLock::new(vec![segment_with_index])),
            write_cache: WriteCache::new(config.cache_config.clone()),
            config,
            current_offset: Arc::new(RwLock::new(LogOffset::ZERO)),
        })
    }

    /// Open an existing log storage
    pub async fn open(base_path: PathBuf, config: LogStorageConfig) -> Result<Self> {
        let mut segment_files = std::fs::read_dir(&base_path)
            .map_err(|e| PyralogError::StorageError(e.to_string()))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension().and_then(|s| s.to_str()) == Some("log")
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        segment_files.sort();

        if segment_files.is_empty() {
            return Self::create(base_path, config).await;
        }

        let mut segments = Vec::new();
        let mut max_offset = LogOffset::ZERO;

        for segment_path in segment_files {
            let segment = Segment::open(segment_path.clone(), config.segment_config.clone())?;
            let index_path = segment_path.with_extension("index");
            let index = if index_path.exists() {
                Index::open(index_path)?
            } else {
                Index::create(&segment_path)?
            };

            if let Some((offset, _, _)) = index.entries().last() {
                max_offset = offset.next();
            }

            segments.push(Arc::new(SegmentWithIndex { segment, index }));
        }

        Ok(Self {
            base_path,
            segments: Arc::new(RwLock::new(segments)),
            write_cache: WriteCache::new(config.cache_config.clone()),
            config,
            current_offset: Arc::new(RwLock::new(max_offset)),
        })
    }

    /// Append a record to the log
    pub async fn append(&self, mut record: Record) -> Result<LogOffset> {
        // Assign offset
        let offset = {
            let mut current = self.current_offset.write();
            let offset = *current;
            *current = current.next();
            offset
        };
        record.offset = offset;

        // Try to add to write cache
        if self.write_cache.push(record.clone())? {
            // Check if we should flush
            if self.write_cache.should_flush() {
                self.flush_cache().await?;
            }
            return Ok(offset);
        }

        // Cache is full, flush and write directly
        self.flush_cache().await?;
        self.write_record(record).await?;

        Ok(offset)
    }

    /// Append a batch of records
    pub async fn append_batch(&self, mut batch: RecordBatch) -> Result<LogOffset> {
        let base_offset = {
            let mut current = self.current_offset.write();
            let offset = *current;
            *current = LogOffset::new(current.as_u64() + batch.count() as u64);
            offset
        };

        batch.base_offset = base_offset;

        // Assign offsets to records
        for (i, record) in batch.records.iter_mut().enumerate() {
            record.offset = LogOffset::new(base_offset.as_u64() + i as u64);
        }

        self.write_batch(batch).await?;

        Ok(base_offset)
    }

    /// Read a record at the given offset
    pub async fn read(&self, offset: LogOffset) -> Result<Option<Record>> {
        let segments = self.segments.read();

        for seg in segments.iter().rev() {
            if offset >= seg.segment.base_offset() {
                if let Some((position, size)) = seg.index.lookup(offset) {
                    let data = seg.segment.read(position, size as usize)?;
                    let record: Record = bincode::deserialize(&data)
                        .map_err(|e| PyralogError::SerializationError(e.to_string()))?;
                    return Ok(Some(record));
                }
            }
        }

        Ok(None)
    }

    /// Read a range of records
    pub async fn read_range(&self, range: OffsetRange) -> Result<Vec<Record>> {
        let mut records = Vec::new();

        for offset_val in range.start.as_u64()..range.end.as_u64() {
            if let Some(record) = self.read(LogOffset::new(offset_val)).await? {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Flush the write cache
    pub async fn flush(&self) -> Result<()> {
        self.flush_cache().await
    }

    /// Get the high watermark
    pub fn high_watermark(&self) -> LogOffset {
        *self.current_offset.read()
    }

    /// Write a single record directly to storage
    async fn write_record(&self, record: Record) -> Result<()> {
        let data = bincode::serialize(&record)
            .map_err(|e| PyralogError::SerializationError(e.to_string()))?;

        let segments = self.segments.read();
        let current_segment = segments.last()
            .ok_or_else(|| PyralogError::StorageError("No segments available".to_string()))?;

        if !current_segment.segment.can_fit(data.len() as u64) {
            drop(segments);
            self.roll_segment().await?;
            return self.write_record(record).await;
        }

        let position = current_segment.segment.append(&data)?;
        current_segment.index.append(record.offset, position, data.len() as u32)?;

        Ok(())
    }

    /// Write a batch of records
    async fn write_batch(&self, batch: RecordBatch) -> Result<()> {
        for record in batch.records {
            self.write_record(record).await?;
        }
        Ok(())
    }

    /// Flush the write cache to storage
    async fn flush_cache(&self) -> Result<()> {
        let records = self.write_cache.drain();
        
        for record in records {
            self.write_record(record).await?;
        }

        let segments = self.segments.read();
        if let Some(seg) = segments.last() {
            seg.segment.sync()?;
            seg.index.sync()?;
        }

        Ok(())
    }

    /// Create a new segment
    async fn roll_segment(&self) -> Result<()> {
        let base_offset = *self.current_offset.read();
        
        let segment = Segment::create(
            base_offset,
            &self.base_path,
            self.config.segment_config.clone(),
        )?;

        let index = Index::create(segment.path())?;

        self.segments.write().push(Arc::new(SegmentWithIndex { segment, index }));

        Ok(())
    }
}

