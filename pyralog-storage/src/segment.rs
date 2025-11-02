use bytes::{Bytes, BytesMut};
use pyralog_core::{LogOffset, Result, DLogError};
use memmap2::{Mmap, MmapMut};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Configuration for segment files
#[derive(Debug, Clone)]
pub struct SegmentConfig {
    pub max_size: u64,
    pub use_mmap: bool,
    pub sync_on_write: bool,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            max_size: 1024 * 1024 * 1024, // 1GB
            use_mmap: true,
            sync_on_write: false,
        }
    }
}

/// A segment file represents a contiguous range of log records
pub struct Segment {
    base_offset: LogOffset,
    path: PathBuf,
    file: RwLock<File>,
    mmap: RwLock<Option<Mmap>>,
    config: SegmentConfig,
    current_size: RwLock<u64>,
}

impl Segment {
    /// Create a new segment
    pub fn create(
        base_offset: LogOffset,
        directory: &Path,
        config: SegmentConfig,
    ) -> Result<Self> {
        let path = directory.join(format!("{:020}.log", base_offset.as_u64()));
        
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        Ok(Self {
            base_offset,
            path,
            file: RwLock::new(file),
            mmap: RwLock::new(None),
            config,
            current_size: RwLock::new(0),
        })
    }

    /// Open an existing segment
    pub fn open(path: PathBuf, config: SegmentConfig) -> Result<Self> {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DLogError::StorageError("Invalid segment path".to_string()))?;

        let base_offset = filename
            .parse::<u64>()
            .map_err(|e| DLogError::StorageError(format!("Invalid offset in filename: {}", e)))?;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        let current_size = file
            .metadata()
            .map_err(|e| DLogError::StorageError(e.to_string()))?
            .len();

        let mut segment = Self {
            base_offset: LogOffset::new(base_offset),
            path,
            file: RwLock::new(file),
            mmap: RwLock::new(None),
            config,
            current_size: RwLock::new(current_size),
        };

        // Create memory map if enabled
        if segment.config.use_mmap && current_size > 0 {
            segment.create_mmap()?;
        }

        Ok(segment)
    }

    /// Write data to the segment
    pub fn append(&self, data: &[u8]) -> Result<u64> {
        let mut file = self.file.write();
        let mut size = self.current_size.write();

        if *size + data.len() as u64 > self.config.max_size {
            return Err(DLogError::StorageError("Segment is full".to_string()));
        }

        let offset = *size;
        
        file.write_all(data)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        if self.config.sync_on_write {
            file.sync_all()
                .map_err(|e| DLogError::StorageError(e.to_string()))?;
        }

        *size += data.len() as u64;

        Ok(offset)
    }

    /// Read data from the segment
    pub fn read(&self, offset: u64, length: usize) -> Result<Bytes> {
        let size = *self.current_size.read();
        
        if offset + length as u64 > size {
            return Err(DLogError::InvalidOffset(offset));
        }

        // Try to read from mmap first
        if let Some(mmap) = self.mmap.read().as_ref() {
            let start = offset as usize;
            let end = start + length;
            return Ok(Bytes::copy_from_slice(&mmap[start..end]));
        }

        // Fallback to file read
        use std::io::{Read, Seek, SeekFrom};
        let mut file = self.file.write();
        let mut buffer = vec![0u8; length];
        
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| DLogError::StorageError(e.to_string()))?;
        
        file.read_exact(&mut buffer)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        Ok(Bytes::from(buffer))
    }

    /// Sync the segment to disk
    pub fn sync(&self) -> Result<()> {
        let file = self.file.read();
        file.sync_all()
            .map_err(|e| DLogError::StorageError(e.to_string()))?;
        Ok(())
    }

    /// Get the base offset of this segment
    pub fn base_offset(&self) -> LogOffset {
        self.base_offset
    }

    /// Get the current size of the segment
    pub fn size(&self) -> u64 {
        *self.current_size.read()
    }

    /// Check if the segment can fit more data
    pub fn can_fit(&self, size: u64) -> bool {
        *self.current_size.read() + size <= self.config.max_size
    }

    /// Create memory map for this segment
    fn create_mmap(&mut self) -> Result<()> {
        let file = self.file.read();
        let mmap = unsafe {
            Mmap::map(&*file)
                .map_err(|e| DLogError::StorageError(e.to_string()))?
        };
        *self.mmap.write() = Some(mmap);
        Ok(())
    }

    /// Get the path to this segment
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_segment_create_and_write() {
        let temp_dir = TempDir::new().unwrap();
        let config = SegmentConfig::default();
        
        let segment = Segment::create(
            LogOffset::new(0),
            temp_dir.path(),
            config,
        ).unwrap();

        let data = b"hello world";
        let offset = segment.append(data).unwrap();
        assert_eq!(offset, 0);

        let read_data = segment.read(offset, data.len()).unwrap();
        assert_eq!(read_data.as_ref(), data);
    }
}

