use async_trait::async_trait;
use bytes::Bytes;

use crate::{
    error::Result,
    offset::{LogOffset, OffsetRange},
    partition::PartitionId,
    record::{Record, RecordBatch},
};

/// Core trait for appending records to a log
#[async_trait]
pub trait LogAppender: Send + Sync {
    /// Append a single record to the log
    async fn append(&mut self, record: Record) -> Result<LogOffset>;

    /// Append a batch of records
    async fn append_batch(&mut self, batch: RecordBatch) -> Result<LogOffset>;

    /// Flush any buffered writes to durable storage
    async fn flush(&mut self) -> Result<()>;
}

/// Core trait for reading records from a log
#[async_trait]
pub trait LogReader: Send + Sync {
    /// Read a single record at the given offset
    async fn read(&self, offset: LogOffset) -> Result<Option<Record>>;

    /// Read a range of records
    async fn read_range(&self, range: OffsetRange) -> Result<Vec<Record>>;

    /// Read records starting from an offset up to a maximum count
    async fn read_from(&self, offset: LogOffset, max_count: usize) -> Result<Vec<Record>>;

    /// Get the current high watermark (last committed offset)
    async fn high_watermark(&self) -> Result<LogOffset>;

    /// Get the low watermark (earliest available offset)
    async fn low_watermark(&self) -> Result<LogOffset>;
}

/// Trait for storage engine operations
#[async_trait]
pub trait StorageEngine: Send + Sync {
    /// Write data to storage
    async fn write(&mut self, data: Bytes) -> Result<u64>;

    /// Read data from storage
    async fn read(&self, offset: u64, length: usize) -> Result<Bytes>;

    /// Sync data to durable storage
    async fn sync(&mut self) -> Result<()>;

    /// Truncate storage to the given offset
    async fn truncate(&mut self, offset: u64) -> Result<()>;
}

/// Trait for consensus protocol operations
#[async_trait]
pub trait ConsensusProtocol: Send + Sync {
    /// Propose a value to be committed
    async fn propose(&mut self, value: Bytes) -> Result<LogOffset>;

    /// Get the current committed offset
    async fn committed_offset(&self) -> Result<LogOffset>;

    /// Check if this node is the leader
    fn is_leader(&self) -> bool;

    /// Get the current leader node ID
    fn leader_id(&self) -> Option<u64>;
}

/// Trait for replication operations
#[async_trait]
pub trait ReplicationManager: Send + Sync {
    /// Replicate a record batch to followers
    async fn replicate(&mut self, batch: RecordBatch) -> Result<()>;

    /// Get replication status for a partition
    async fn replication_status(&self, partition: PartitionId) -> Result<ReplicationStatus>;

    /// Wait for replication to reach a specific offset
    async fn wait_for_replication(&self, offset: LogOffset) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct ReplicationStatus {
    pub partition: PartitionId,
    pub leader_offset: LogOffset,
    pub follower_offsets: Vec<(u64, LogOffset)>,
    pub in_sync_replicas: Vec<u64>,
}

