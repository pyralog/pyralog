use serde::{Deserialize, Serialize};
use std::fmt;

use crate::partition::PartitionId;

/// Unique identifier for a log
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LogId {
    pub namespace: String,
    pub name: String,
}

impl LogId {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }
}

impl fmt::Display for LogId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.namespace, self.name)
    }
}

/// Metadata about a log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetadata {
    pub id: LogId,
    pub partition_count: u32,
    pub replication_factor: u32,
    pub retention_policy: RetentionPolicy,
    pub config: LogConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Maximum size of a single segment file
    pub segment_size: u64,
    
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
    
    /// Enable compression
    pub compression_enabled: bool,
    
    /// Enable tiered storage
    pub tiered_storage_enabled: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            segment_size: 1024 * 1024 * 1024, // 1GB
            flush_interval_ms: 1000,           // 1 second
            compression_enabled: true,
            tiered_storage_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetentionPolicy {
    /// Retain data for a specific duration (in seconds)
    Time(u64),
    
    /// Retain data up to a specific size (in bytes)
    Size(u64),
    
    /// Retain data based on both time and size (whichever is reached first)
    TimeAndSize { time_seconds: u64, size_bytes: u64 },
    
    /// Keep data forever
    Forever,
}

