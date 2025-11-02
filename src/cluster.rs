use pyralog_core::{LogId, LogMetadata, PartitionId, Result, PyralogError};
use pyralog_consensus::{RaftNode, RaftConfig};
use pyralog_storage::LogStorage;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Cluster manager handles log metadata and partition assignments
pub struct ClusterManager {
    node_id: u64,
    raft: Arc<RaftNode>,
    logs: Arc<RwLock<HashMap<LogId, LogMetadata>>>,
    partition_assignments: Arc<RwLock<HashMap<PartitionId, Vec<u64>>>>,
}

impl ClusterManager {
    pub async fn new(config: RaftConfig) -> Result<Self> {
        let node_id = config.node_id;
        let raft = Arc::new(RaftNode::new(config).await?);
        
        Ok(Self {
            node_id,
            raft,
            logs: Arc::new(RwLock::new(HashMap::new())),
            partition_assignments: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        Arc::clone(&self.raft).start().await?;
        Ok(())
    }

    /// Register a new log in the cluster
    pub async fn create_log(&self, metadata: LogMetadata) -> Result<()> {
        // In production, this would go through Raft consensus
        self.logs.write().insert(metadata.id.clone(), metadata);
        Ok(())
    }

    /// Get log metadata
    pub fn get_log(&self, log_id: &LogId) -> Option<LogMetadata> {
        self.logs.read().get(log_id).cloned()
    }

    /// Get partition assignment (which nodes host this partition)
    pub fn get_partition_nodes(&self, partition: PartitionId) -> Option<Vec<u64>> {
        self.partition_assignments.read().get(&partition).cloned()
    }

    /// Check if this node is the leader for a partition
    pub fn is_partition_leader(&self, partition: PartitionId) -> bool {
        self.get_partition_nodes(partition)
            .map(|nodes| nodes.first() == Some(&self.node_id))
            .unwrap_or(false)
    }

    /// List all logs in the cluster
    pub fn list_logs(&self) -> Vec<LogId> {
        self.logs.read().keys().cloned().collect()
    }

    pub fn node_id(&self) -> u64 {
        self.node_id
    }
}

