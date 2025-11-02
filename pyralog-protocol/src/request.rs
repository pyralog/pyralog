use bytes::{Bytes, BytesMut};
use pyralog_core::{Result, DLogError};
use serde::{Deserialize, Serialize};

/// Wire format for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Produce(crate::api::ProduceRequest),
    Consume(crate::api::ConsumeRequest),
    CreateLog(crate::api::CreateLogRequest),
    DeleteLog(pyralog_core::LogId),
    ListLogs,
}

impl Request {
    /// Serialize request to bytes
    pub fn to_bytes(&self) -> Result<Bytes> {
        bincode::serialize(self)
            .map(Bytes::from)
            .map_err(|e| DLogError::SerializationError(e.to_string()))
    }

    /// Deserialize request from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| DLogError::SerializationError(e.to_string()))
    }
}

