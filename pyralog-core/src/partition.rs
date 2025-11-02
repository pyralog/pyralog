use serde::{Deserialize, Serialize};
use std::fmt;

use crate::log::LogId;
use crate::offset::LogOffset;

/// Partition identifier within a log
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PartitionId(pub u32);

impl PartitionId {
    pub fn new(id: u32) -> Self {
        PartitionId(id)
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for PartitionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for PartitionId {
    fn from(id: u32) -> Self {
        PartitionId(id)
    }
}

/// Represents a partition in a distributed log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partition {
    pub log_id: LogId,
    pub partition_id: PartitionId,
    pub leader_node: Option<u64>,
    pub replica_nodes: Vec<u64>,
    pub low_watermark: LogOffset,
    pub high_watermark: LogOffset,
}

impl Partition {
    pub fn new(log_id: LogId, partition_id: PartitionId) -> Self {
        Self {
            log_id,
            partition_id,
            leader_node: None,
            replica_nodes: Vec::new(),
            low_watermark: LogOffset::ZERO,
            high_watermark: LogOffset::ZERO,
        }
    }

    pub fn is_leader(&self, node_id: u64) -> bool {
        self.leader_node == Some(node_id)
    }

    pub fn is_replica(&self, node_id: u64) -> bool {
        self.replica_nodes.contains(&node_id)
    }
}

