# Pyralog Implementation Plan

Comprehensive roadmap for building Pyralog from the ground up.

## Table of Contents

1. [Overview](#overview)
2. [Phase 1: Foundation](#phase-1-foundation)
3. [Phase 2: Core Storage](#phase-2-core-storage)
4. [Phase 3: Consensus & Replication](#phase-3-consensus--replication)
5. [Phase 4: Epochs & Smart Clients](#phase-4-epochs--smart-clients)
6. [Phase 5: Production Hardening](#phase-5-production-hardening)
7. [Phase 6: Advanced Features](#phase-6-advanced-features)
8. [Testing Strategy](#testing-strategy)
9. [Deployment Strategy](#deployment-strategy)
10. [Success Criteria](#success-criteria)

---

## Overview

### Guiding Principles

1. **Build vertically first** - Get end-to-end working, then optimize
2. **Test continuously** - Every component has comprehensive tests
3. **Benchmark early** - Measure performance from day one
4. **Document as you go** - Code and docs in sync
5. **Production-minded** - Design for real-world use from the start
6. **Evolutionary architecture** - Start simple (per-partition), evolve to maximum scale (coordinator-only)

### Timeline Estimate

```
Phase 1: Foundation                    (2-3 weeks)
Phase 2: Core Storage                  (3-4 weeks)
Phase 3: Consensus & Replication       (4-6 weeks)
Phase 4: Epochs & Smart Clients        (3-4 weeks)
Phase 5: Production Hardening          (4-6 weeks)
Phase 6: Advanced Features             (Ongoing)
───────────────────────────────────────────────
Total to Production-Ready:             16-23 weeks (~4-6 months)
```

### Success Metrics

**Performance:**
- ✅ 1M+ writes/sec on 3-node cluster (per-partition mode)
- ✅ 5M+ writes/sec on 10-node cluster (coordinator-only mode)
- ✅ < 1ms p99 write latency
- ✅ 99%+ reduction in leader disk I/O (coordinator-only vs traditional)
- ✅ 20x-50x more partitions per node (coordinator-only mode)

**Reliability:**
- ✅ 100% test coverage for critical paths
- ✅ Zero data loss with RF=3, W=2
- ✅ < 300ms leader failover time

**Scalability:**
- ✅ Linear scaling to 10 nodes (partitioning)
- ✅ Linear scaling to 50+ nodes (coordinator-only mode)
- ✅ 100-500 partitions per node (coordinator-only mode)

---

## Architecture Evolution Path

Pyralog's architecture supports **three CopySet strategies** that can be adopted progressively:

```
┌────────────────────────────────────────────────────────────┐
│   Strategy Progression                                     │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  Phase 3-4: START HERE                                     │
│  ─────────────────────                                     │
│  Mode 1: Per-Partition CopySet                             │
│  ✅ Simplest to implement                                   │
│  ✅ Kafka-compatible                                        │
│  ✅ 10-20 partitions/node                                   │
│  📝 Phase 3 deliverable                                    │
│                                                            │
│  Phase 4-5: OPTIMIZE                                       │
│  ────────────────────                                      │
│  Mode 2: Per-Record + Leader Storage (Hybrid)              │
│  ✅ Better load distribution                                │
│  ✅ Leader still serves reads                               │
│  ✅ 20-100 partitions/node                                  │
│  📝 Phase 4 optimization                                   │
│                                                            │
│  Phase 5+: MAXIMUM SCALE                                   │
│  ───────────────────────                                   │
│  Mode 3: Per-Record Coordinator-Only                       │
│  ✅ Leader disk-free (99%+ less I/O!)                       │
│  ✅ 20x-50x more partitions/node                            │
│  ✅ 100-500 partitions/node                                 │
│  📝 Production optimization for large deployments          │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Key Innovation: Leader as Coordinator**

With per-record CopySet in coordinator-only mode:
- Leader doesn't store data locally (just metadata)
- Leader assigns LSN and routes to storage nodes
- 99%+ reduction in leader disk I/O
- Enables massive partition density per node

**Implementation Priority:**
1. **Phase 3**: Implement Mode 1 (per-partition) - Get it working
2. **Phase 4**: Add Mode 2 (hybrid) - Improve distribution
3. **Phase 5**: Add Mode 3 (coordinator-only) - Maximize scale

All three modes are **configurable** via `copyset_strategy` in config file.

---

## Phase 1: Foundation

**Goal**: Basic project structure and core types

**Duration**: 2-3 weeks

### 1.1 Project Setup

```bash
# Create workspace
cargo new --lib pyralog
cd pyralog

# Create crates
cargo new --lib pyralog-core
cargo new --lib pyralog-storage
cargo new --lib pyralog-consensus
cargo new --lib pyralog-replication
cargo new --lib pyralog-protocol
cargo new --bin pyralog-server

# Setup CI/CD
# - GitHub Actions for tests
# - Code coverage with tarpaulin
# - Clippy for lints
# - Rustfmt for formatting
```

**Tasks:**
- [x] Create workspace structure
- [x] Setup Cargo.toml with dependencies
- [x] Configure CI/CD pipeline
- [x] Setup pre-commit hooks
- [x] Create LICENSE files

### 1.2 Core Types (pyralog-core)

**Files to create:**
- `src/error.rs` - Error types with thiserror
- `src/offset.rs` - LogOffset type
- `src/record.rs` - Record and RecordBatch
- `src/log.rs` - LogId and metadata
- `src/partition.rs` - PartitionId
- `src/traits.rs` - Core traits

**Example implementation:**

```rust
// src/offset.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogOffset(u64);

impl LogOffset {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(u64::MAX);
    
    pub fn new(offset: u64) -> Self {
        Self(offset)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// src/record.rs
pub struct Record {
    pub offset: LogOffset,
    pub epoch: Epoch,
    pub timestamp: SystemTime,
    pub key: Option<Bytes>,
    pub value: Bytes,
    pub headers: Vec<RecordHeader>,
}

// src/traits.rs
#[async_trait]
pub trait Storage: Send + Sync {
    async fn append(&self, record: Record) -> Result<LogOffset>;
    async fn read(&self, offset: LogOffset) -> Result<Option<Record>>;
    async fn read_range(&self, start: LogOffset, end: LogOffset) -> Result<Vec<Record>>;
}
```

**Tests:**
- Unit tests for all types
- Serialization/deserialization tests
- Property-based tests with proptest

**Milestone**: Core types compile, have tests, pass CI

---

## Phase 2: Core Storage

**Goal**: Working single-node storage engine

**Duration**: 3-4 weeks

### 2.1 Segment Storage (Week 1-2)

**Implementation order:**

1. **Basic Segment** (`pyralog-storage/src/segment.rs`)

```rust
pub struct Segment {
    base_offset: LogOffset,
    path: PathBuf,
    file: File,
    size: u64,
    max_size: u64,
}

impl Segment {
    pub fn create(base_offset: LogOffset, dir: &Path, max_size: u64) -> Result<Self>;
    pub fn open(path: &Path) -> Result<Self>;
    pub fn append(&mut self, data: &[u8]) -> Result<u64>; // Returns position
    pub fn read(&self, position: u64, length: usize) -> Result<Bytes>;
    pub fn size(&self) -> u64;
    pub fn is_full(&self) -> bool;
    pub fn flush(&mut self) -> Result<()>;
}
```

2. **Sparse Index** (`pyralog-storage/src/index.rs`)

```rust
pub struct Index {
    entries: Vec<IndexEntry>,
    interval: u32, // Index every N records
}

struct IndexEntry {
    offset: LogOffset,
    position: u64,
    size: u32,
}

impl Index {
    pub fn append(&mut self, offset: LogOffset, position: u64, size: u32);
    pub fn lookup(&self, offset: LogOffset) -> Option<(u64, u32)>;
    pub fn save(&self, path: &Path) -> Result<()>;
    pub fn load(path: &Path) -> Result<Self>;
}
```

3. **Log Storage** (`pyralog-storage/src/log_storage.rs`)

```rust
pub struct LogStorage {
    partition_id: PartitionId,
    data_dir: PathBuf,
    segments: Vec<Segment>,
    active_segment: Segment,
    config: StorageConfig,
}

impl LogStorage {
    pub async fn new(partition_id: PartitionId, config: StorageConfig) -> Result<Self>;
    pub async fn append(&mut self, record: Record) -> Result<LogOffset>;
    pub async fn read(&self, offset: LogOffset) -> Result<Option<Record>>;
    pub async fn read_batch(&self, start: LogOffset, max: usize) -> Result<Vec<Record>>;
    
    async fn roll_segment(&mut self) -> Result<()>;
    fn find_segment(&self, offset: LogOffset) -> Option<&Segment>;
}
```

**Tests:**
- Create segment, write data, read back
- Test segment rolling at max size
- Test index lookups (hit, miss, edge cases)
- Test recovery after crash (reopen segments)
- Concurrent read tests

**Benchmark:**
```bash
cargo bench --bench storage_benchmark
```

Target: 100K writes/sec, 1M reads/sec (single-threaded)

### 2.2 Write Cache (Week 2-3)

**Implementation:**

```rust
pub struct WriteCache {
    buffer: RwLock<Vec<Record>>,
    max_size: usize,
    max_records: usize,
    last_flush: Instant,
    flush_interval: Duration,
}

impl WriteCache {
    pub async fn append(&self, record: Record) -> Result<LogOffset>;
    pub async fn flush(&self) -> Result<Vec<Record>>;
    
    async fn should_flush(&self) -> bool;
}
```

**Tests:**
- Test flush on size threshold
- Test flush on time threshold
- Test concurrent writes
- Test crash recovery (WAL)

**Optimization**: Add Write-Ahead Log for durability

### 2.3 Memory-Mapped I/O (Week 3-4)

**Implementation:**

```rust
use memmap2::Mmap;

pub struct MmapSegment {
    mmap: Option<Mmap>,
    file: File,
    size: u64,
}

impl MmapSegment {
    pub fn open_with_mmap(path: &Path) -> Result<Self>;
    pub fn read_zero_copy(&self, offset: u64, len: usize) -> &[u8];
}
```

**Tests:**
- Compare mmap vs file I/O performance
- Test large files (> 1GB)
- Test concurrent reads

**Milestone**: Single-node storage engine working with benchmarks showing target performance

---

## Phase 3: Consensus & Replication

**Goal**: Multi-node cluster with Raft consensus

**Duration**: 4-6 weeks

### 3.1 Raft Consensus (Week 1-3)

**Use existing crate or implement from scratch?**

Option A: Use `raft` crate (faster, battle-tested)
```toml
[dependencies]
raft = "0.7"
protobuf = "3.5"
```

Option B: Implement from scratch (more control, learning)

**Recommended: Use raft crate**, focus on integration

**Implementation:**

```rust
// pyralog-consensus/src/raft.rs
pub struct RaftNode {
    id: u64,
    peers: Vec<u64>,
    raw_node: RawNode<MemStorage>,
    // ... state
}

impl RaftNode {
    pub fn new(id: u64, peers: Vec<u64>) -> Result<Self>;
    pub fn tick(&mut self);
    pub fn propose(&mut self, data: Vec<u8>) -> Result<()>;
    pub fn step(&mut self, msg: Message) -> Result<()>;
    pub fn ready(&mut self) -> Option<Ready>;
}
```

**Tasks:**
- Implement Raft node wrapper
- Implement persistent storage for Raft log
- Implement RPC layer (use Tonic gRPC)
- Leader election
- Log replication
- Snapshot support

**Tests:**
- Test leader election
- Test log replication
- Test network partitions (Jepsen-style)
- Test crash recovery

### 3.2 Cluster Management (Week 3-4)

**Implementation:**

```rust
pub struct ClusterManager {
    nodes: HashMap<NodeId, NodeMetadata>,
    raft: RaftNode,
}

pub struct NodeMetadata {
    id: NodeId,
    addr: String,
    status: NodeStatus,
    partitions: Vec<PartitionId>,
}

impl ClusterManager {
    pub async fn join_cluster(&mut self, addr: String) -> Result<()>;
    pub async fn get_leader(&self, partition: PartitionId) -> Result<NodeId>;
    pub async fn get_nodes(&self) -> Vec<NodeMetadata>;
}
```

### 3.3 Replication System (Week 4-6)

**Implementation:**

```rust
// pyralog-replication/src/replicator.rs
pub struct Replicator {
    partition: PartitionId,
    strategy: CopySetStrategy,
    write_quorum: usize,
    connections: HashMap<NodeId, Connection>,
    node_id: NodeId,
    local_storage: Option<Arc<LogStorage>>,  // Optional for coordinator mode
}

impl Replicator {
    pub async fn replicate(&self, record: Record) -> Result<()> {
        // Select CopySet based on strategy
        let copyset = self.select_copyset(&record);
        
        // Write to local storage if leader stores data
        if self.should_store_locally() {
            self.local_storage.as_ref().unwrap().append(record.clone()).await?;
        }
        
        // Send to remote nodes
        let futures = self.send_to_replicas(&copyset, record).await;
        self.wait_for_quorum(futures).await
    }
    
    fn should_store_locally(&self) -> bool;
    async fn send_to_replica(&self, node: NodeId, record: Record) -> Result<()>;
    async fn wait_for_quorum(&self, futures: Vec<JoinHandle<Result<()>>>) -> Result<()>;
}
```

**CopySet Strategy (Two Modes):**

```rust
// pyralog-replication/src/copyset.rs
#[derive(Debug, Clone)]
pub enum CopySetStrategy {
    /// Fixed CopySet per partition (Kafka-style)
    PerPartition,
    
    /// Dynamic CopySet per record (LogDevice-style)
    PerRecord {
        seed: u64,
        /// Leader as coordinator (doesn't store data)
        leader_stores_data: bool,
    },
}

// Strategy 1: Per-Partition (simpler)
pub struct PartitionCopySetSelector {
    assignments: HashMap<PartitionId, Vec<NodeId>>,
}

impl PartitionCopySetSelector {
    pub fn select(&self, partition: PartitionId) -> Vec<NodeId> {
        self.assignments.get(&partition).cloned().unwrap()
    }
}

// Strategy 2: Per-Record (maximum distribution)
pub struct RecordCopySetSelector {
    nodes: Vec<NodeId>,
    replication_factor: usize,
    seed: u64,
}

impl RecordCopySetSelector {
    pub fn select(&self, lsn: u64) -> Vec<NodeId> {
        // Deterministic selection based on LSN
        // NO storage needed - pure function!
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        lsn.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Select RF unique nodes
        let mut selected = Vec::with_capacity(self.replication_factor);
        let mut offset = hash as usize;
        
        while selected.len() < self.nodes.len() && 
              selected.len() < self.replication_factor {
            let idx = offset % self.nodes.len();
            let node = self.nodes[idx];
            
            if !selected.contains(&node) {
                selected.push(node);
            }
            offset += 1;
        }
        
        selected
    }
}
```

**Configuration:**

```toml
[replication]
replication_factor = 3
write_quorum = 2
read_quorum = 2

# Start simple: Per-partition
copyset_strategy = "PerPartition"

# Later: Per-record with coordinator-only mode for maximum scale
[replication.copyset_strategy]
type = "PerRecord"
seed = 42
leader_stores_data = false  # Leader is disk-free!
```

**Tests:**
- Test replication to all nodes
- Test partial failure (1 node down)
- Test quorum success/failure
- Test CopySet distribution (both strategies)
- Test per-record load balancing
- Test leader-as-coordinator mode (no local storage)
- Test hybrid mode (leader stores data)

**Milestone**: 3-node cluster with Raft consensus and replication working, both CopySet strategies implemented

---

## Phase 4: Epochs & Smart Clients

**Goal**: High-throughput writes with epochs, smart client routing

**Duration**: 3-4 weeks

### 4.1 Epoch System (Week 1-2)

**Implementation:**

```rust
// pyralog-core/src/epoch.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Epoch(pub u64);

pub struct EpochMetadata {
    pub epoch: Epoch,
    pub sequencer_id: NodeId,
    pub start_offset: LogOffset,
    pub end_offset: Option<LogOffset>,
    pub sealed: bool,
    pub sealed_at: Option<SystemTime>,
}

#[async_trait]
pub trait EpochStore: Send + Sync {
    async fn get_current_epoch(&self, partition: &PartitionId) -> Result<EpochMetadata>;
    async fn activate_epoch(&self, partition: &PartitionId, node: NodeId) -> Result<EpochMetadata>;
    async fn seal_epoch(&self, partition: &PartitionId, epoch: Epoch, end_offset: LogOffset) -> Result<()>;
}

// Store in RocksDB
pub struct RocksDBEpochStore {
    db: Arc<DB>,
}
```

**Sequencer implementation (with coordinator mode support):**

```rust
// pyralog-core/src/sequencer.rs
pub struct Sequencer {
    partition: PartitionId,
    current_epoch: RwLock<EpochMetadata>,
    next_offset: AtomicU64,
    epoch_store: Arc<dyn EpochStore>,
    copyset_selector: Option<RecordCopySetSelector>,  // For per-record CopySet
    local_storage: Option<Arc<LogStorage>>,  // None in coordinator-only mode
    replicator: Arc<Replicator>,
}

impl Sequencer {
    pub async fn activate(&self, node_id: NodeId) -> Result<Epoch>;
    pub async fn seal(&self, epoch: Epoch, end_offset: LogOffset) -> Result<()>;
    pub fn next_offset(&self) -> LogOffset;
    pub async fn can_write(&self, epoch: Epoch) -> bool;
    
    /// Append records (coordinator mode or traditional)
    pub async fn append(&self, record: Record) -> Result<LogOffset> {
        // 1. Get current epoch
        let epoch = self.current_epoch();
        
        // 2. Check if can write
        if !self.can_write(epoch).await {
            return Err(PyralogError::EpochSealed(epoch));
        }
        
        // 3. Assign offset (local increment - no consensus!)
        let offset = self.next_offset();
        
        // 4. Set epoch and offset on record
        let mut record = record;
        record.epoch = epoch;
        record.offset = offset;
        
        // 5. Replicate (may or may not include local storage)
        self.replicator.replicate(record).await?;
        
        Ok(offset)
    }
}
```

**Two modes of operation:**

```rust
// Mode 1: Traditional (leader stores data)
impl LogStorage {
    pub async fn append_with_epoch(
        &mut self, 
        record: Record, 
        sequencer: &Sequencer
    ) -> Result<LogOffset> {
        let offset = sequencer.append(record).await?;
        // Sequencer writes to local storage + replicates
        Ok(offset)
    }
}

// Mode 2: Coordinator-only (leader doesn't store data)
impl Sequencer {
    pub async fn append(&self, record: Record) -> Result<LogOffset> {
        // 1. Assign LSN (metadata only, no disk I/O!)
        let epoch = self.current_epoch();
        let offset = self.next_offset();
        let lsn = EpochOffset::new(epoch, offset).as_u64();
        
        // 2. Calculate CopySet
        let copyset = self.copyset_selector
            .as_ref()
            .unwrap()
            .select(lsn);
        
        // 3. Set record metadata
        let mut record = record;
        record.epoch = epoch;
        record.offset = offset;
        
        // 4. Send DIRECTLY to storage nodes (NOT local disk!)
        for node in copyset {
            self.send_to_storage_node(node, record.clone()).await?;
        }
        
        // 5. Wait for quorum
        self.wait_for_quorum(copyset.len()).await?;
        
        // 6. Done! Leader never touched disk! ✅
        Ok(offset)
    }
}
```

**Tests:**
- Test epoch activation on leader election
- Test epoch sealing on failover
- Test writes rejected with old epoch
- Test offset assignment is local (no Raft)
- Test coordinator-only mode (leader doesn't write to disk)
- Test hybrid mode (leader writes to disk)
- Test leader I/O reduction in coordinator mode

**Benchmarks:**
- Verify 100x throughput improvement vs. consensus-per-record
- Measure leader disk I/O in both modes:
  - Traditional: ~100GB/hour per partition
  - Coordinator-only: ~10MB/hour per partition (99%+ reduction!)
- Verify leader can handle 20x-50x more partitions in coordinator mode

**Implementation Strategy:**
1. **Week 1-2**: Start with traditional mode (leader stores data)
2. **Week 2-3**: Add per-record CopySet with leader storage (hybrid)
3. **Week 3-4**: Implement coordinator-only mode (maximum scale)
4. **Week 4**: Benchmark and optimize all three modes

### 4.2 Smart Client Protocol (Week 2-3)

**Metadata protocol:**

```rust
// pyralog-protocol/src/metadata.rs
#[derive(Serialize, Deserialize)]
pub struct MetadataRequest {
    pub log_ids: Vec<LogId>,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataResponse {
    pub logs: Vec<LogMetadata>,
    pub brokers: Vec<BrokerMetadata>,
}

pub struct LogMetadata {
    pub log_id: LogId,
    pub partitions: Vec<PartitionMetadata>,
}

pub struct PartitionMetadata {
    pub partition_id: PartitionId,
    pub leader: NodeId,
    pub replicas: Vec<NodeId>,
    pub isr: Vec<NodeId>,
}
```

**Client implementation:**

```rust
// pyralog-protocol/src/client.rs
pub struct PyralogClient {
    bootstrap_servers: Vec<String>,
    metadata_cache: Arc<RwLock<MetadataCache>>,
    connections: Arc<RwLock<HashMap<NodeId, Connection>>>,
    partitioner: Box<dyn Partitioner>,
}

impl PyralogClient {
    pub async fn produce(&self, log_id: LogId, record: Record) -> Result<LogOffset> {
        // 1. Calculate partition
        let partition = self.partitioner.partition(&record.key, &log_id)?;
        
        // 2. Get leader from cache
        let leader = self.get_leader(&log_id, partition).await?;
        
        // 3. Send directly to leader
        match self.send_to_node(leader, record).await {
            Ok(offset) => Ok(offset),
            Err(PyralogError::NotLeader(new_leader)) => {
                // Refresh metadata and retry
                self.refresh_metadata(&log_id).await?;
                self.send_to_node(new_leader, record).await
            }
            Err(e) => Err(e),
        }
    }
    
    async fn refresh_metadata(&self, log_id: &LogId) -> Result<()>;
}
```

**Tests:**
- Test metadata fetch and caching
- Test direct routing to leader
- Test failover (NotLeader error handling)
- Test stale metadata refresh

### 4.3 gRPC Protocol (Week 3-4)

**Define protocol:**

```protobuf
// proto/pyralog.proto
syntax = "proto3";

service PyralogService {
    rpc Produce(ProduceRequest) returns (ProduceResponse);
    rpc Consume(ConsumeRequest) returns (ConsumeResponse);
    rpc GetMetadata(MetadataRequest) returns (MetadataResponse);
}

message ProduceRequest {
    string log_id = 1;
    uint32 partition_id = 2;
    repeated Record records = 3;
}

message ProduceResponse {
    repeated OffsetResult offsets = 1;
}

message Record {
    bytes key = 1;
    bytes value = 2;
    repeated Header headers = 3;
}
```

**Implementation with Tonic:**

```rust
// pyralog-server/src/grpc.rs
use tonic::{Request, Response, Status};

pub struct PyralogServiceImpl {
    storage: Arc<LogStorage>,
    sequencer: Arc<Sequencer>,
    replicator: Arc<Replicator>,
}

#[tonic::async_trait]
impl PyralogService for PyralogServiceImpl {
    async fn produce(
        &self,
        request: Request<ProduceRequest>,
    ) -> Result<Response<ProduceResponse>, Status> {
        let req = request.into_inner();
        
        // Process records
        let mut offsets = Vec::new();
        for record in req.records {
            let offset = self.storage
                .append_with_epoch(record, &self.sequencer)
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            
            offsets.push(OffsetResult { offset: offset.as_u64() });
        }
        
        Ok(Response::new(ProduceResponse { offsets }))
    }
}
```

**Milestone**: Smart client can produce and consume with direct leader routing, epochs working

---

## Phase 5: Production Hardening

**Goal**: Production-ready system with monitoring, recovery, and reliability

**Duration**: 4-6 weeks

### 5.1 Observability (Week 1-2)

**Metrics with Prometheus:**

```rust
use prometheus::{Counter, Histogram, IntGauge, Registry};

lazy_static! {
    static ref RECORDS_WRITTEN: Counter = register_counter!(
        "pyralog_records_written_total",
        "Total records written"
    ).unwrap();
    
    static ref WRITE_LATENCY: Histogram = register_histogram!(
        "pyralog_write_latency_seconds",
        "Write latency"
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge!(
        "pyralog_active_connections",
        "Active client connections"
    ).unwrap();
}
```

**Structured logging with tracing:**

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn append(&self, record: Record) -> Result<LogOffset> {
    info!("Appending record with key: {:?}", record.key);
    
    let start = Instant::now();
    let result = self.append_internal(record).await;
    let elapsed = start.elapsed();
    
    WRITE_LATENCY.observe(elapsed.as_secs_f64());
    
    match result {
        Ok(offset) => {
            RECORDS_WRITTEN.inc();
            info!(offset = %offset, "Record appended");
            Ok(offset)
        }
        Err(e) => {
            error!(error = %e, "Failed to append record");
            Err(e)
        }
    }
}
```

**OpenTelemetry for distributed tracing:**

```rust
use opentelemetry::trace::Tracer;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    .install_batch(opentelemetry::runtime::Tokio)?;
```

### 5.2 Recovery & Persistence (Week 2-3)

**Crash recovery:**

```rust
impl LogStorage {
    pub async fn recover(partition_id: PartitionId, data_dir: PathBuf) -> Result<Self> {
        info!("Recovering partition {}", partition_id);
        
        // 1. Find all segments
        let mut segments = Self::discover_segments(&data_dir)?;
        segments.sort_by_key(|s| s.base_offset());
        
        // 2. Verify each segment
        for segment in &mut segments {
            segment.verify_integrity()?;
        }
        
        // 3. Recover index for active segment
        let active_segment = segments.last_mut().unwrap();
        let index = Self::rebuild_index(active_segment)?;
        
        // 4. Find high watermark
        let high_watermark = Self::find_high_watermark(&segments)?;
        
        info!("Recovery complete, high watermark: {}", high_watermark);
        
        Ok(Self {
            partition_id,
            data_dir,
            segments,
            // ...
        })
    }
}
```

**Epoch recovery:**

```rust
impl Sequencer {
    pub async fn recover(&self) -> Result<()> {
        // 1. Load epoch metadata from persistent store
        let metadata = self.epoch_store.get_current_epoch(&self.partition).await?;
        
        // 2. If sealed, activate new epoch
        if metadata.sealed {
            let new_epoch = self.activate(self.node_id).await?;
            info!("Activated new epoch: {}", new_epoch.as_u64());
        } else {
            // 3. Continue with current epoch
            info!("Continuing with epoch: {}", metadata.epoch.as_u64());
        }
        
        // 4. Find next offset
        let next_offset = self.find_next_offset().await?;
        self.next_offset.store(next_offset.as_u64(), Ordering::SeqCst);
        
        Ok(())
    }
}
```

### 5.3 Configuration Management (Week 3-4)

**Configuration file:**

```toml
# config/default.toml
[server]
node_id = 1
bind_addr = "0.0.0.0:9092"
data_dir = "/var/lib/pyralog"

[cluster]
bootstrap_servers = ["node1:9092", "node2:9092", "node3:9092"]

[storage]
segment_max_size = 1073741824  # 1GB
index_interval = 4096           # Index every 4KB
enable_mmap = true

[storage.cache]
max_size = 33554432             # 32MB
max_records = 10000
flush_interval_ms = 100

[replication]
replication_factor = 3
write_quorum = 2
read_quorum = 2

# CopySet Strategy Evolution:
# Stage 1 (Phase 3): Simple per-partition
copyset_strategy = "PerPartition"

# Stage 2 (Phase 4): Per-record with leader storage (hybrid)
# [replication.copyset_strategy]
# type = "PerRecord"
# seed = 42
# leader_stores_data = true

# Stage 3 (Phase 5+): Per-record coordinator-only (maximum scale)
# [replication.copyset_strategy]
# type = "PerRecord"
# seed = 42
# leader_stores_data = false

[raft]
election_timeout_ms = 300
heartbeat_interval_ms = 100

[observability]
metrics_port = 9090
tracing_endpoint = "http://jaeger:4317"
log_level = "info"
```

**Load configuration:**

```rust
use config::{Config, File, Environment};

#[derive(Deserialize)]
pub struct PyralogConfig {
    pub server: ServerConfig,
    pub cluster: ClusterConfig,
    pub storage: StorageConfig,
    pub replication: ReplicationConfig,
    pub raft: RaftConfig,
    pub observability: ObservabilityConfig,
}

impl PyralogConfig {
    pub fn load() -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("PYRALOG"))
            .build()?;
        
        config.try_deserialize()
    }
}
```

### 5.4 Health Checks & Admin API (Week 4)

```rust
// Admin endpoints
#[derive(Deserialize)]
enum AdminCommand {
    GetStatus,
    GetPartitions,
    GetMetrics,
    RebalancePartitions,
    ShutdownGraceful,
}

pub struct AdminApi {
    cluster: Arc<ClusterManager>,
    storage: Arc<LogStorage>,
}

impl AdminApi {
    pub async fn get_status(&self) -> StatusResponse {
        StatusResponse {
            node_id: self.cluster.node_id(),
            state: self.cluster.state(),
            partitions: self.storage.partitions(),
            // ...
        }
    }
    
    pub async fn health_check(&self) -> HealthStatus {
        // Check:
        // - Can write to storage
        // - Raft is healthy
        // - Replicas are reachable
        // - Disk space available
        HealthStatus::Healthy
    }
}
```

### 5.5 Integration Tests (Week 5-6)

**End-to-end test:**

```rust
#[tokio::test]
async fn test_full_cluster() {
    // 1. Start 3-node cluster
    let cluster = TestCluster::new(3).await;
    
    // 2. Create client
    let client = cluster.client();
    
    // 3. Write records
    for i in 0..1000 {
        let record = Record::new(
            Some(format!("key-{}", i).into()),
            format!("value-{}", i).into(),
        );
        client.produce("test-log", record).await.unwrap();
    }
    
    // 4. Read back
    let records = client.consume("test-log", 0, LogOffset::ZERO, 1000).await.unwrap();
    assert_eq!(records.len(), 1000);
    
    // 5. Kill leader
    cluster.kill_leader().await;
    
    // 6. Wait for new leader
    tokio::time::sleep(Duration::from_secs(1)).await;
    
    // 7. Continue writing
    for i in 1000..2000 {
        let record = Record::new(
            Some(format!("key-{}", i).into()),
            format!("value-{}", i).into(),
        );
        client.produce("test-log", record).await.unwrap();
    }
    
    // 8. Verify all 2000 records
    let all_records = client.consume("test-log", 0, LogOffset::ZERO, 2000).await.unwrap();
    assert_eq!(all_records.len(), 2000);
}
```

**Chaos testing:**

```rust
#[tokio::test]
async fn test_network_partition() {
    let cluster = TestCluster::new(5).await;
    
    // Create network partition: [1,2] | [3,4,5]
    cluster.partition_network(vec![1, 2], vec![3, 4, 5]).await;
    
    // Majority side should continue working
    let client = cluster.client_for_nodes(vec![3, 4, 5]);
    client.produce("test", Record::new(None, b"test".to_vec())).await.unwrap();
    
    // Minority side should fail
    let minority_client = cluster.client_for_nodes(vec![1, 2]);
    assert!(minority_client.produce("test", Record::new(None, b"test".to_vec())).await.is_err());
    
    // Heal partition
    cluster.heal_network().await;
    
    // All nodes should converge
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(cluster.verify_consistency().await);
}
```

**CopySet Strategy Testing:**

```rust
#[tokio::test]
async fn test_coordinator_only_mode() {
    // Configure cluster with coordinator-only mode
    let config = TestConfig::new()
        .with_copyset_strategy(CopySetStrategy::PerRecord {
            seed: 42,
            leader_stores_data: false,
        });
    
    let cluster = TestCluster::with_config(config, 5).await;
    let client = cluster.client();
    
    // Write 10K records
    for i in 0..10000 {
        client.produce("test", Record::new(
            Some(format!("key-{}", i).into()),
            format!("value-{}", i).into(),
        )).await.unwrap();
    }
    
    // Verify leader has minimal disk usage (just metadata)
    let leader = cluster.get_leader_for_partition("test", 0).await;
    let leader_disk_usage = cluster.get_disk_usage(leader).await;
    assert!(leader_disk_usage < 100_000_000); // < 100MB (should be ~10MB)
    
    // Verify data is distributed across storage nodes
    let storage_nodes = cluster.get_storage_nodes().await;
    for node in storage_nodes {
        let disk_usage = cluster.get_disk_usage(node).await;
        assert!(disk_usage > 1_000_000_000); // > 1GB
    }
    
    // Verify reads work (clients calculate CopySet)
    let records = client.consume("test", 0, LogOffset::ZERO, 10000).await.unwrap();
    assert_eq!(records.len(), 10000);
}

#[tokio::test]
async fn test_copyset_load_distribution() {
    let cluster = TestCluster::new(10).await;
    
    // Test per-partition: fixed replicas
    cluster.set_copyset_strategy(CopySetStrategy::PerPartition).await;
    cluster.produce_records("test", 100000).await;
    let per_partition_distribution = cluster.measure_load_distribution().await;
    
    // Test per-record: dynamic distribution
    cluster.set_copyset_strategy(CopySetStrategy::PerRecord {
        seed: 42,
        leader_stores_data: false,
    }).await;
    cluster.produce_records("test2", 100000).await;
    let per_record_distribution = cluster.measure_load_distribution().await;
    
    // Per-record should have better distribution
    assert!(per_record_distribution.std_dev < per_partition_distribution.std_dev);
    assert!(per_record_distribution.max_load < per_partition_distribution.max_load * 1.1);
}
```

**Milestone**: Production-ready system with observability, recovery, and passing all integration tests (including all three CopySet modes)

---

## Phase 6: Advanced Features

**Goal**: Additional features for production use

**Duration**: Ongoing

**Note on Competitive Advantage:**

By this phase, Pyralog will have a **unique architecture** that combines:
- ✅ Epochs (LogDevice) - 100x throughput vs consensus-per-record
- ✅ Smart Clients (Kafka) - Direct routing, no proxy bottleneck
- ✅ Three CopySet modes - Configurable from simple to maximum scale
- ✅ Leader-as-coordinator - 99%+ less leader I/O, 20x-50x partition density

**No other distributed log system offers all three CopySet strategies in one system!**

### 6.1 Tiered Storage

- Implement S3/Azure/GCS backends
- Automatic archival of old segments
- Transparent retrieval on read
- **Optimization**: Coordinator-only mode makes tiering even more valuable (minimal local storage)

### 6.2 Compaction

- Key-based log compaction
- Tombstone handling
- Background compaction workers
- **Optimization**: Per-record CopySet distributes compaction load

### 6.3 Transactions

- Two-phase commit
- Transaction coordinator
- Idempotent producers
- **Integration**: Works with all three CopySet modes

### 6.4 Consumer Groups

- Group coordination
- Partition assignment
- Rebalancing protocol
- **Optimization**: Consumers can read from any CopySet node (not just leader)

### 6.5 Schema Registry

- Avro/Protobuf support
- Schema evolution
- Compatibility checks

---

## Testing Strategy

### Unit Tests

**Coverage target: 80%+ for all crates**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_offset_ordering() {
        assert!(LogOffset::new(1) < LogOffset::new(2));
    }
    
    #[tokio::test]
    async fn test_append_and_read() {
        let storage = LogStorage::new_temp().await.unwrap();
        let record = Record::new(None, b"test".to_vec());
        
        let offset = storage.append(record.clone()).await.unwrap();
        let read = storage.read(offset).await.unwrap().unwrap();
        
        assert_eq!(record.value, read.value);
    }
}
```

### Integration Tests

**Test scenarios:**
- 3-node cluster startup
- Leader election
- Write and read
- Failover
- Network partitions
- Rolling restart

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_offset_serialization(offset in 0u64..1000000) {
        let log_offset = LogOffset::new(offset);
        let bytes = bincode::serialize(&log_offset).unwrap();
        let deserialized: LogOffset = bincode::deserialize(&bytes).unwrap();
        assert_eq!(log_offset, deserialized);
    }
}
```

### Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_write(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(LogStorage::new_temp()).unwrap();
    
    c.bench_function("write", |b| {
        b.to_async(&rt).iter(|| async {
            let record = Record::new(None, black_box(vec![0u8; 1024]));
            storage.append(record).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_write);
criterion_main!(benches);
```

### Chaos Engineering

**Use tools like:**
- Jepsen for linearizability checking
- Chaos Mesh for Kubernetes chaos testing
- Custom network partition simulator

---

## Deployment Strategy

### Docker

```dockerfile
# Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/pyralog-server /usr/local/bin/
EXPOSE 9092 9090
CMD ["pyralog-server"]
```

### Kubernetes

```yaml
# k8s/statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: pyralog
spec:
  serviceName: pyralog
  replicas: 3
  selector:
    matchLabels:
      app: pyralog
  template:
    metadata:
      labels:
        app: pyralog
    spec:
      containers:
      - name: pyralog
        image: pyralog:latest
        ports:
        - containerPort: 9092
          name: pyralog
        - containerPort: 9090
          name: metrics
        volumeMounts:
        - name: data
          mountPath: /var/lib/pyralog
        env:
        - name: PYRALOG_NODE_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 100Gi
```

### Binary Release

```bash
# Build for multiple platforms
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create release tarball
tar -czf pyralog-v0.1.0-linux-x64.tar.gz -C target/x86_64-unknown-linux-gnu/release pyralog-server
```

---

## Success Criteria

### Phase 1 ✅
- [ ] Project compiles
- [ ] All core types have tests
- [ ] CI/CD pipeline working

### Phase 2 ✅
- [ ] Single-node storage working
- [ ] Benchmarks show 100K writes/sec
- [ ] Zero data loss on crash recovery

### Phase 3 ✅
- [ ] 3-node cluster working
- [ ] Leader election < 300ms
- [ ] Replication with quorum working
- [ ] Per-partition CopySet implemented

### Phase 4 ✅
- [ ] Epochs implemented
- [ ] 1M+ writes/sec achieved (per-partition mode)
- [ ] Smart client routing working
- [ ] Per-record CopySet with leader storage (hybrid mode)
- [ ] Per-record coordinator-only mode implemented
- [ ] 5M+ writes/sec achieved (coordinator mode, 10-node cluster)

### Phase 5 ✅
- [ ] Prometheus metrics exposed
- [ ] Distributed tracing working
- [ ] All integration tests passing
- [ ] < 1ms p99 write latency
- [ ] All three CopySet modes tested and benchmarked
- [ ] 99%+ leader I/O reduction verified (coordinator mode)
- [ ] 20x-50x partition density increase verified

### Phase 6 (Ongoing)
- [ ] Tiered storage working
- [ ] Compaction implemented
- [ ] Production deployment successful

---

## Risk Management

### Technical Risks

**Risk**: Raft implementation has bugs
- **Mitigation**: Use battle-tested `raft` crate, extensive testing

**Risk**: Performance doesn't meet targets
- **Mitigation**: Benchmark continuously, profile and optimize

**Risk**: Data loss on edge cases
- **Mitigation**: Comprehensive testing, formal verification of critical paths

### Timeline Risks

**Risk**: Scope creep
- **Mitigation**: Strict prioritization, MVP first

**Risk**: Dependencies delayed
- **Mitigation**: Parallel work where possible, early integration

---

## Next Steps

1. **Review and approve plan** with stakeholders
2. **Setup development environment** (Rust, tools)
3. **Create GitHub project board** with tasks
4. **Start Phase 1** - Foundation
5. **Weekly progress reviews** and adjustments

---

## Resources

### Team
- 1-2 experienced Rust developers
- 1 distributed systems expert (advisor)
- 1 DevOps engineer (part-time)

### Infrastructure
- CI/CD pipeline (GitHub Actions)
- Test cluster (3-5 VMs)
- Monitoring stack (Prometheus, Grafana, Jaeger)

### Documentation
- Keep this plan updated
- Document decisions in ADRs
- Update API docs as code evolves

---

## Final Notes: Pyralog's Unique Value Proposition

### What Makes Pyralog Special

**1. Three CopySet Strategies in One System**

Pyralog is the **only distributed log** that offers three configurable replication strategies:
- **Per-Partition** (Kafka-style) - Simplicity
- **Per-Record Hybrid** (Best of both) - Balanced
- **Per-Record Coordinator** (LogDevice-inspired) - Maximum scale

**2. Leader-as-Coordinator Innovation**

The coordinator-only mode is **groundbreaking**:
```
Traditional systems:
  Leader disk I/O: 100 GB/hour per partition
  Partitions/node: 10-20

Pyralog coordinator mode:
  Leader disk I/O: 10 MB/hour per partition (99%+ reduction!)
  Partitions/node: 100-500 (20x-50x increase!)
```

**3. Complete Feature Set**

- ✅ **Epochs** (LogDevice) - Decouples offset assignment from consensus
- ✅ **Smart Clients** (Kafka) - Direct routing, no proxies
- ✅ **Flexible Quorums** - Configurable CAP position
- ✅ **Three CopySet modes** - Progressive scaling path

**4. Production-Ready Design**

From day one, Pyralog is designed for:
- High throughput (5M+ writes/sec)
- Low latency (< 1ms p99)
- Massive scale (50+ nodes, 10K+ partitions)
- Zero data loss (RF=3, W=2)
- Fast failover (< 300ms)

### The Competitive Edge

**No other system combines all these:**
- Kafka: ❌ No epochs, ❌ No per-record CopySet, ❌ No coordinator mode
- Pulsar: ❌ No epochs, ❌ Fixed architecture
- LogDevice: ❌ No Kafka compatibility, ❌ Complex deployment
- **Pyralog**: ✅ All three modes + Kafka-compatible + Simple deployment

### Ready to Build?

This plan takes you from zero to production-ready in **4-6 months**. Follow the phases, test continuously, benchmark early, and you'll have a distributed log that **outperforms and outscales** anything available today.

---

**Let's build Pyralog! 🚀**

