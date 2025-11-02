use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use crate::offset::LogOffset;
use crate::epoch::Epoch;

/// A single log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Unique offset of this record in the log
    pub offset: LogOffset,
    
    /// Epoch number (which sequencer/leader wrote this)
    pub epoch: Epoch,
    
    /// Timestamp when the record was created
    pub timestamp: SystemTime,
    
    /// Optional key for the record (used for partitioning and compaction)
    pub key: Option<Bytes>,
    
    /// The actual payload data
    pub value: Bytes,
    
    /// Optional headers for metadata
    pub headers: Vec<RecordHeader>,
}

impl Record {
    pub fn new(key: Option<Bytes>, value: Bytes) -> Self {
        Self {
            offset: LogOffset::ZERO,
            epoch: Epoch::INVALID,
            timestamp: SystemTime::now(),
            key,
            value,
            headers: Vec::new(),
        }
    }

    pub fn with_epoch(mut self, epoch: Epoch) -> Self {
        self.epoch = epoch;
        self
    }

    pub fn with_headers(mut self, headers: Vec<RecordHeader>) -> Self {
        self.headers = headers;
        self
    }

    pub fn size_bytes(&self) -> usize {
        self.key.as_ref().map_or(0, |k| k.len()) + self.value.len()
    }
}

/// Record header for metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordHeader {
    pub key: String,
    pub value: Bytes,
}

impl RecordHeader {
    pub fn new(key: String, value: Bytes) -> Self {
        Self { key, value }
    }
}

/// A batch of records for efficient I/O
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordBatch {
    /// Starting offset of this batch
    pub base_offset: LogOffset,
    
    /// Epoch this batch belongs to
    pub epoch: Epoch,
    
    /// Records in this batch
    pub records: Vec<Record>,
    
    /// Compression codec used (if any)
    pub compression: CompressionType,
    
    /// CRC checksum for integrity
    pub crc: u32,
}

impl RecordBatch {
    pub fn new(base_offset: LogOffset, records: Vec<Record>) -> Self {
        Self {
            base_offset,
            epoch: Epoch::INVALID,
            records,
            compression: CompressionType::None,
            crc: 0,
        }
    }

    pub fn with_epoch(mut self, epoch: Epoch) -> Self {
        self.epoch = epoch;
        self
    }

    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }

    pub fn size_bytes(&self) -> usize {
        self.records.iter().map(|r| r.size_bytes()).sum()
    }

    pub fn last_offset(&self) -> Option<LogOffset> {
        self.records.last().map(|r| r.offset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Snappy,
    Lz4,
    Zstd,
}

