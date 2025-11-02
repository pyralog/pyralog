use thiserror::Error;

pub type Result<T> = std::result::Result<T, PyralogError>;

#[derive(Error, Debug, Clone)]
pub enum PyralogError {
    #[error("Log not found: {0}")]
    LogNotFound(String),

    #[error("Partition not found: {0}")]
    PartitionNotFound(u64),

    #[error("Invalid offset: {0}")]
    InvalidOffset(u64),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("Replication error: {0}")]
    ReplicationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Leader not available")]
    LeaderNotAvailable,

    #[error("Not leader: current leader is {0:?}")]
    NotLeader(Option<u64>),

    #[error("Quorum not available")]
    QuorumNotAvailable,

    #[error("Operation timeout")]
    Timeout,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("IO error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for PyralogError {
    fn from(err: std::io::Error) -> Self {
        PyralogError::IoError(err.to_string())
    }
}

