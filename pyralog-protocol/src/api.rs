use async_trait::async_trait;
use bytes::Bytes;
use pyralog_core::{LogId, LogOffset, PartitionId, Record, Result};
use serde::{Deserialize, Serialize};

/// Request to produce records to a log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProduceRequest {
    pub log_id: LogId,
    pub partition: Option<PartitionId>,
    pub records: Vec<ProduceRecord>,
    pub acks: AckMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProduceRecord {
    pub key: Option<Bytes>,
    pub value: Bytes,
    pub headers: Vec<(String, Bytes)>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AckMode {
    /// Don't wait for acknowledgment
    None,
    
    /// Wait for leader acknowledgment
    Leader,
    
    /// Wait for all in-sync replicas
    All,
}

/// Response from produce request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProduceResponse {
    pub partition: PartitionId,
    pub base_offset: LogOffset,
    pub error: Option<String>,
}

/// Request to consume records from a log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumeRequest {
    pub log_id: LogId,
    pub partition: PartitionId,
    pub offset: LogOffset,
    pub max_records: usize,
    pub max_bytes: usize,
}

/// Response from consume request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumeResponse {
    pub partition: PartitionId,
    pub high_watermark: LogOffset,
    pub records: Vec<Record>,
    pub error: Option<String>,
}

/// Create log request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLogRequest {
    pub log_id: LogId,
    pub partition_count: u32,
    pub replication_factor: u32,
}

/// Protocol handler trait
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Handle produce request
    async fn produce(&self, request: ProduceRequest) -> Result<ProduceResponse>;

    /// Handle consume request
    async fn consume(&self, request: ConsumeRequest) -> Result<ConsumeResponse>;

    /// Create a new log
    async fn create_log(&self, request: CreateLogRequest) -> Result<()>;

    /// Delete a log
    async fn delete_log(&self, log_id: LogId) -> Result<()>;

    /// List all logs
    async fn list_logs(&self) -> Result<Vec<LogId>>;
}

