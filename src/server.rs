use crate::cluster::ClusterManager;
use crate::config::DLogConfig;
use pyralog_consensus::RaftConfig;
use pyralog_core::{LogId, LogMetadata, LogConfig, PartitionId, Record, RecordHeader, Result, DLogError, RetentionPolicy};
use pyralog_protocol::{
    api::*, Partitioner, PartitionStrategy,
};
use pyralog_replication::ReplicationManager;
use pyralog_storage::LogStorage;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use bytes::Bytes;

/// Main DLog server
pub struct DLogServer {
    config: DLogConfig,
    cluster: Arc<ClusterManager>,
    storage: Arc<RwLock<HashMap<(LogId, PartitionId), Arc<LogStorage>>>>,
    replication: Arc<ReplicationManager>,
}

impl DLogServer {
    /// Create a new DLog server
    pub async fn new(config: DLogConfig) -> Result<Self> {
        let raft_config = RaftConfig {
            node_id: config.node.node_id,
            cluster_nodes: config.node.cluster_nodes.clone(),
            data_dir: config.node.data_dir.join("raft"),
            election_timeout: pyralog_consensus::election::ElectionTimeoutConfig::default(),
        };

        std::fs::create_dir_all(&config.node.data_dir)
            .map_err(|e| DLogError::ConfigError(e.to_string()))?;

        let cluster = Arc::new(ClusterManager::new(raft_config).await?);
        
        let replication = Arc::new(ReplicationManager::new(
            config.replication.clone(),
            config.node.cluster_nodes.clone(),
        ));

        Ok(Self {
            config,
            cluster,
            storage: Arc::new(RwLock::new(HashMap::new())),
            replication,
        })
    }

    /// Start the server
    pub async fn start(self: Arc<Self>) -> Result<()> {
        tracing::info!("Starting DLog server on {}", self.config.network.listen_address);

        // Start cluster manager
        Arc::clone(&self.cluster).start().await?;

        // Start network listeners
        let listener = TcpListener::bind(&self.config.network.listen_address)
            .await
            .map_err(|e| DLogError::NetworkError(e.to_string()))?;

        tracing::info!("DLog server listening on {}", self.config.network.listen_address);

        // Accept connections
        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    tracing::debug!("Accepted connection from {}", addr);
                    let server = Arc::clone(&self);
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(socket).await {
                            tracing::error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a client connection
    async fn handle_connection(&self, socket: tokio::net::TcpStream) -> Result<()> {
        // In production, this would implement the full protocol handler
        // For now, this is a placeholder
        Ok(())
    }

    /// Get or create storage for a log partition
    async fn get_or_create_storage(
        &self,
        log_id: &LogId,
        partition: PartitionId,
    ) -> Result<Arc<LogStorage>> {
        let key = (log_id.clone(), partition);

        // Check if storage already exists
        {
            let storage = self.storage.read();
            if let Some(s) = storage.get(&key) {
                return Ok(Arc::clone(s));
            }
        }

        // Create new storage
        let path = self
            .config
            .node
            .data_dir
            .join(format!("{}/{}/partition-{}", log_id.namespace, log_id.name, partition.as_u32()));

        let storage = Arc::new(
            LogStorage::create(path, self.config.storage.clone()).await?
        );

        self.storage.write().insert(key, Arc::clone(&storage));

        Ok(storage)
    }
}

#[async_trait::async_trait]
impl ProtocolHandler for DLogServer {
    async fn produce(&self, request: ProduceRequest) -> Result<ProduceResponse> {
        // Get log metadata
        let metadata = self
            .cluster
            .get_log(&request.log_id)
            .ok_or_else(|| DLogError::LogNotFound(request.log_id.to_string()))?;

        // Determine partition
        let partitioner = Partitioner::new(
            PartitionStrategy::KeyHash,
            metadata.partition_count,
        );

        let partition = if let Some(p) = request.partition {
            p
        } else {
            // Use first record's key for partitioning
            let first_record = request.records.first()
                .ok_or_else(|| DLogError::InvalidRequest("No records in request".to_string()))?;
            
            partitioner.partition(
                first_record.key.as_ref(),
                &first_record.value,
            )
        };

        // Check if we're the leader for this partition
        if !self.cluster.is_partition_leader(partition) {
            return Err(DLogError::NotLeader(None));
        }

        // Get storage
        let storage = self.get_or_create_storage(&request.log_id, partition).await?;

        // Convert records
        let mut base_offset = None;
        for produce_record in request.records {
            let headers: Vec<RecordHeader> = produce_record
                .headers
                .into_iter()
                .map(|(k, v)| RecordHeader::new(k, v))
                .collect();

            let record = Record::new(produce_record.key, produce_record.value)
                .with_headers(headers);

            let offset = storage.append(record).await?;
            if base_offset.is_none() {
                base_offset = Some(offset);
            }
        }

        let base_offset = base_offset
            .ok_or_else(|| DLogError::InvalidRequest("No records written".to_string()))?;

        // Flush if required
        if matches!(request.acks, AckMode::Leader | AckMode::All) {
            storage.flush().await?;
        }

        Ok(ProduceResponse {
            partition,
            base_offset,
            error: None,
        })
    }

    async fn consume(&self, request: ConsumeRequest) -> Result<ConsumeResponse> {
        // Get storage
        let storage = self
            .get_or_create_storage(&request.log_id, request.partition)
            .await?;

        // Read records
        let records = storage
            .read_from(request.offset, request.max_records)
            .await?;

        let high_watermark = storage.high_watermark();

        Ok(ConsumeResponse {
            partition: request.partition,
            high_watermark,
            records,
            error: None,
        })
    }

    async fn create_log(&self, request: CreateLogRequest) -> Result<()> {
        let metadata = LogMetadata {
            id: request.log_id,
            partition_count: request.partition_count,
            replication_factor: request.replication_factor,
            retention_policy: RetentionPolicy::Forever,
            config: LogConfig::default(),
        };

        self.cluster.create_log(metadata).await
    }

    async fn delete_log(&self, log_id: LogId) -> Result<()> {
        // In production, this would mark the log for deletion
        // and clean up storage
        Ok(())
    }

    async fn list_logs(&self) -> Result<Vec<LogId>> {
        Ok(self.cluster.list_logs())
    }
}

