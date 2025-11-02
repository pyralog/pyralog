use bytes::Bytes;
use pyralog_core::{LogId, Result, PyralogError};
use serde::{Deserialize, Serialize};

/// Wire format for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Produce(crate::api::ProduceResponse),
    Consume(crate::api::ConsumeResponse),
    CreateLog(Result<()>),
    DeleteLog(Result<()>),
    ListLogs(Result<Vec<LogId>>),
    Error(String),
}

impl Response {
    /// Serialize response to bytes
    pub fn to_bytes(&self) -> Result<Bytes> {
        bincode::serialize(self)
            .map(Bytes::from)
            .map_err(|e| PyralogError::SerializationError(e.to_string()))
    }

    /// Deserialize response from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| PyralogError::SerializationError(e.to_string()))
    }
}

