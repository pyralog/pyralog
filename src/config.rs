use pyralog_consensus::RaftConfig;
use pyralog_replication::ReplicationConfig;
use pyralog_storage::{LogStorageConfig, SegmentConfig, WriteCacheConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DLogConfig {
    /// Node configuration
    pub node: NodeConfig,
    
    /// Storage configuration
    pub storage: LogStorageConfig,
    
    /// Replication configuration
    pub replication: ReplicationConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node ID
    pub node_id: u64,
    
    /// Data directory
    pub data_dir: PathBuf,
    
    /// Cluster nodes (for consensus)
    pub cluster_nodes: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address for client connections
    pub listen_address: String,
    
    /// Listen address for internal cluster communication
    pub internal_address: String,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
}

impl Default for DLogConfig {
    fn default() -> Self {
        Self {
            node: NodeConfig {
                node_id: 1,
                data_dir: PathBuf::from("./data"),
                cluster_nodes: vec![1],
            },
            storage: LogStorageConfig {
                segment_config: SegmentConfig {
                    max_size: 1024 * 1024 * 1024, // 1GB
                    use_mmap: true,
                    sync_on_write: false,
                },
                cache_config: WriteCacheConfig {
                    max_size: 16 * 1024 * 1024, // 16MB
                    max_buffer_time: tokio::time::Duration::from_millis(10),
                    enabled: true,
                },
            },
            replication: ReplicationConfig::default(),
            network: NetworkConfig {
                listen_address: "0.0.0.0:9092".to_string(),
                internal_address: "0.0.0.0:9093".to_string(),
                max_connections: 10000,
                request_timeout_ms: 30000,
            },
        }
    }
}

impl DLogConfig {
    /// Load configuration from a file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn to_file(&self, path: &str) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

