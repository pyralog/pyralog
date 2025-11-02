//! Kafka protocol compatibility layer
//!
//! This module provides Kafka wire protocol compatibility,
//! allowing existing Kafka clients to work with DLog.

use pyralog_core::{LogId, PartitionId};

/// Map Kafka topic to DLog LogId
pub fn kafka_topic_to_log_id(topic: &str) -> LogId {
    LogId::new("kafka", topic)
}

/// Map DLog LogId to Kafka topic
pub fn log_id_to_kafka_topic(log_id: &LogId) -> String {
    if log_id.namespace == "kafka" {
        log_id.name.clone()
    } else {
        format!("{}.{}", log_id.namespace, log_id.name)
    }
}

/// Kafka API versions supported
#[derive(Debug, Clone, Copy)]
pub enum KafkaApiVersion {
    V0,
    V1,
    V2,
    V3,
}

/// Kafka request types
#[derive(Debug, Clone, Copy)]
pub enum KafkaApiKey {
    Produce = 0,
    Fetch = 1,
    ListOffsets = 2,
    Metadata = 3,
    LeaderAndIsr = 4,
    StopReplica = 5,
    UpdateMetadata = 6,
    ControlledShutdown = 7,
    OffsetCommit = 8,
    OffsetFetch = 9,
    FindCoordinator = 10,
    JoinGroup = 11,
    Heartbeat = 12,
    LeaveGroup = 13,
    SyncGroup = 14,
    DescribeGroups = 15,
    ListGroups = 16,
    SaslHandshake = 17,
    ApiVersions = 18,
    CreateTopics = 19,
    DeleteTopics = 20,
}

/// Kafka error codes
#[derive(Debug, Clone, Copy)]
#[repr(i16)]
pub enum KafkaErrorCode {
    None = 0,
    OffsetOutOfRange = 1,
    CorruptMessage = 2,
    UnknownTopicOrPartition = 3,
    InvalidMessageSize = 4,
    LeaderNotAvailable = 5,
    NotLeaderForPartition = 6,
    RequestTimedOut = 7,
    BrokerNotAvailable = 8,
    ReplicaNotAvailable = 9,
    MessageSizeTooLarge = 10,
    StaleControllerEpoch = 11,
    OffsetMetadataTooLarge = 12,
    NetworkException = 13,
    GroupLoadInProgress = 14,
    GroupCoordinatorNotAvailable = 15,
    NotCoordinatorForGroup = 16,
    InvalidTopic = 17,
    RecordListTooLarge = 18,
    NotEnoughReplicas = 19,
    NotEnoughReplicasAfterAppend = 20,
}

impl From<&pyralog_core::DLogError> for KafkaErrorCode {
    fn from(error: &pyralog_core::DLogError) -> Self {
        match error {
            pyralog_core::DLogError::InvalidOffset(_) => KafkaErrorCode::OffsetOutOfRange,
            pyralog_core::DLogError::LogNotFound(_) => KafkaErrorCode::UnknownTopicOrPartition,
            pyralog_core::DLogError::LeaderNotAvailable => KafkaErrorCode::LeaderNotAvailable,
            pyralog_core::DLogError::NotLeader(_) => KafkaErrorCode::NotLeaderForPartition,
            pyralog_core::DLogError::Timeout => KafkaErrorCode::RequestTimedOut,
            pyralog_core::DLogError::QuorumNotAvailable => KafkaErrorCode::NotEnoughReplicas,
            _ => KafkaErrorCode::NetworkException,
        }
    }
}

/// Placeholder for Kafka protocol codec
/// In production, this would implement full Kafka wire protocol
pub struct KafkaCodec {
    version: KafkaApiVersion,
}

impl KafkaCodec {
    pub fn new(version: KafkaApiVersion) -> Self {
        Self { version }
    }
}

