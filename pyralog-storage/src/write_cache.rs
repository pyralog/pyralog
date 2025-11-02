use bytes::Bytes;
use pyralog_core::{LogOffset, Record, RecordBatch, Result, PyralogError};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

/// Write cache for improved write latency
/// Inspired by Redpanda's write caching feature
pub struct WriteCache {
    buffer: Arc<Mutex<CacheBuffer>>,
    config: WriteCacheConfig,
}

struct CacheBuffer {
    records: VecDeque<Record>,
    total_size: usize,
    last_flush: Instant,
}

#[derive(Debug, Clone)]
pub struct WriteCacheConfig {
    /// Maximum size of the cache in bytes
    pub max_size: usize,
    
    /// Maximum time to buffer records before flushing
    pub max_buffer_time: Duration,
    
    /// Enable write caching
    pub enabled: bool,
}

impl Default for WriteCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 16 * 1024 * 1024, // 16MB
            max_buffer_time: Duration::from_millis(10),
            enabled: true,
        }
    }
}

impl WriteCache {
    pub fn new(config: WriteCacheConfig) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(CacheBuffer {
                records: VecDeque::new(),
                total_size: 0,
                last_flush: Instant::now(),
            })),
            config,
        }
    }

    /// Add a record to the cache
    pub fn push(&self, record: Record) -> Result<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        let mut buffer = self.buffer.lock();
        let record_size = record.size_bytes();

        // Check if we need to flush
        if buffer.total_size + record_size > self.config.max_size {
            return Ok(false); // Signal that flush is needed
        }

        buffer.records.push_back(record);
        buffer.total_size += record_size;

        Ok(true)
    }

    /// Check if the cache should be flushed
    pub fn should_flush(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        let buffer = self.buffer.lock();
        
        buffer.total_size >= self.config.max_size
            || buffer.last_flush.elapsed() >= self.config.max_buffer_time
            || buffer.records.len() >= 1000
    }

    /// Get all records from the cache and clear it
    pub fn drain(&self) -> Vec<Record> {
        let mut buffer = self.buffer.lock();
        let records = buffer.records.drain(..).collect();
        buffer.total_size = 0;
        buffer.last_flush = Instant::now();
        records
    }

    /// Get the current size of the cache
    pub fn size(&self) -> usize {
        self.buffer.lock().total_size
    }

    /// Get the number of records in the cache
    pub fn len(&self) -> usize {
        self.buffer.lock().records.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_write_cache() {
        let config = WriteCacheConfig::default();
        let cache = WriteCache::new(config);

        let record = Record::new(None, Bytes::from("test"));
        
        assert!(cache.push(record.clone()).unwrap());
        assert_eq!(cache.len(), 1);

        let records = cache.drain();
        assert_eq!(records.len(), 1);
        assert_eq!(cache.len(), 0);
    }
}

