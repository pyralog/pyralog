# Recommended Rust Libraries for Pyralog

Comprehensive guide to the Rust ecosystem for building high-performance distributed log systems.

## Table of Contents

1. [Core Dependencies](#core-dependencies)
2. [Async Runtime](#async-runtime)
3. [Serialization](#serialization)
4. [Networking](#networking)
5. [Storage & I/O](#storage--io)
6. [Consensus & Replication](#consensus--replication)
7. [Data Structures](#data-structures)
8. [Error Handling](#error-handling)
9. [Observability](#observability)
10. [Testing](#testing)
11. [Performance & Profiling](#performance--profiling)
12. [Utilities](#utilities)

---

## Core Dependencies

### Must-Have Libraries

```toml
[dependencies]
# Async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
bytes = "1.7"
uuid = { version = "1.10", features = ["v4", "serde"] }
```

---

## Async Runtime

### Tokio (Primary Choice)

**Why Tokio?**
- Industry standard for async Rust
- Excellent performance and scalability
- Comprehensive ecosystem
- Great documentation

```toml
[dependencies]
tokio = { version = "1.40", features = [
    "rt-multi-thread",  # Multi-threaded runtime
    "macros",           # #[tokio::main], #[tokio::test]
    "sync",             # Channels, mutexes, semaphores
    "time",             # Sleep, interval, timeout
    "fs",               # Async file I/O
    "net",              # TCP/UDP networking
    "io-util",          # AsyncRead/AsyncWrite utilities
    "signal",           # Unix signals
] }
```

**Usage Example**:

```rust
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
    let mut file = File::open("log.dat").await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(())
}
```

### Alternative: async-std

```toml
[dependencies]
async-std = { version = "1.12", features = ["attributes"] }
```

**When to use**:
- Simpler API closer to std lib
- Different runtime characteristics
- Academic/research projects

---

## Serialization

### bincode (Primary for Storage)

**Why bincode?**
- Extremely fast binary serialization
- Compact representation
- Zero-copy deserialization support
- Perfect for on-disk storage

```toml
[dependencies]
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
```

**Usage**:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Record {
    offset: u64,
    timestamp: u64,
    key: Vec<u8>,
    value: Vec<u8>,
}

// Serialize
let record = Record { /* ... */ };
let bytes = bincode::serialize(&record)?;

// Deserialize
let record: Record = bincode::deserialize(&bytes)?;
```

### MessagePack (Alternative)

```toml
[dependencies]
rmp-serde = "1.3"
```

**When to use**:
- Need schema evolution
- Interoperability with other languages
- Self-describing format

### Protocol Buffers

```toml
[dependencies]
prost = "0.13"
prost-types = "0.13"

[build-dependencies]
prost-build = "0.13"
```

**When to use**:
- RPC interfaces
- Cross-language compatibility
- Strong schema definitions

**Example**:

```proto
// record.proto
syntax = "proto3";

message Record {
    uint64 offset = 1;
    uint64 timestamp = 2;
    bytes key = 3;
    bytes value = 4;
}
```

### JSON (For Config/APIs)

```toml
[dependencies]
serde_json = "1.0"
```

**Usage**:

```rust
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct Config {
    port: u16,
    data_dir: String,
}

// Read config
let config: Config = serde_json::from_str(&json_string)?;

// Write config
let json = serde_json::to_string_pretty(&config)?;
```

---

## Networking

### Tonic (gRPC)

**Why Tonic?**
- High-performance gRPC implementation
- Built on Tokio
- Excellent for service-to-service communication

```toml
[dependencies]
tonic = "0.12"
prost = "0.13"

[build-dependencies]
tonic-build = "0.12"
```

**Usage**:

```rust
// Service definition
#[tonic::async_trait]
impl PyralogService for PyralogServer {
    async fn produce(
        &self,
        request: Request<ProduceRequest>,
    ) -> Result<Response<ProduceResponse>, Status> {
        let req = request.into_inner();
        let offset = self.append_record(req.record).await?;
        Ok(Response::new(ProduceResponse { offset }))
    }
}

// Server
Server::builder()
    .add_service(PyralogServiceServer::new(server))
    .serve(addr)
    .await?;
```

### Hyper (HTTP)

```toml
[dependencies]
hyper = { version = "1.5", features = ["full"] }
hyper-util = { version = "0.1", features = ["full"] }
```

**When to use**:
- REST APIs
- Admin interfaces
- Metrics endpoints

### Quinn (QUIC)

```toml
[dependencies]
quinn = "0.11"
```

**When to use**:
- Low-latency replication
- Modern transport protocol
- Better than TCP for certain workloads

---

## Storage & I/O

### memmap2 (Memory-Mapped Files)

**Why memmap2?**
- Zero-copy access to files
- OS-managed caching
- Excellent read performance

```toml
[dependencies]
memmap2 = "0.9"
```

**Usage**:

```rust
use memmap2::{Mmap, MmapOptions};
use std::fs::File;

pub struct Segment {
    file: File,
    mmap: Option<Mmap>,
}

impl Segment {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe {
            MmapOptions::new().map(&file)?
        };
        Ok(Self { file, mmap: Some(mmap) })
    }
    
    pub fn read(&self, offset: u64, len: usize) -> &[u8] {
        let start = offset as usize;
        &self.mmap.as_ref().unwrap()[start..start + len]
    }
}
```

### tokio-uring (io_uring for Linux)

```toml
[dependencies]
tokio-uring = "0.5"
```

**When to use**:
- Linux-only deployments
- Maximum I/O performance
- High-throughput workloads

**Example**:

```rust
use tokio_uring::fs::File;

fn main() -> Result<()> {
    tokio_uring::start(async {
        let file = File::open("segment.log").await?;
        let buf = vec![0u8; 4096];
        let (res, buf) = file.read_at(buf, 0).await;
        let n = res?;
        println!("Read {} bytes", n);
        Ok(())
    })
}
```

### RocksDB

```toml
[dependencies]
rocksdb = "0.22"
```

**When to use**:
- Metadata storage
- Index storage
- State management

**Example**:

```rust
use rocksdb::{DB, Options};

pub struct MetadataStore {
    db: DB,
}

impl MetadataStore {
    pub fn new(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db })
    }
    
    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.put(key, value)?;
        Ok(())
    }
    
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?)
    }
}
```

### Sled (Pure Rust Alternative)

```toml
[dependencies]
sled = "0.34"
```

**When to use**:
- Pure Rust stack (no C++ dependencies)
- Simpler embedding
- Good enough performance

---

## Consensus & Replication

### Raft Implementation

```toml
[dependencies]
raft = "0.7"
protobuf = "3.5"
```

**Usage**:

```rust
use raft::{Config, RawNode, Storage};

pub struct RaftNode {
    node: RawNode<MemStorage>,
}

impl RaftNode {
    pub fn new(id: u64, peers: Vec<u64>) -> Result<Self> {
        let config = Config {
            id,
            election_tick: 10,
            heartbeat_tick: 3,
            ..Default::default()
        };
        
        let storage = MemStorage::new();
        let node = RawNode::new(&config, storage, &Default::default())?;
        
        Ok(Self { node })
    }
    
    pub fn tick(&mut self) {
        self.node.tick();
    }
    
    pub fn propose(&mut self, data: Vec<u8>) -> Result<()> {
        self.node.propose(vec![], data)?;
        Ok(())
    }
}
```

### Openraft (Modern Alternative)

```toml
[dependencies]
openraft = "0.9"
```

**Why Openraft?**
- More modern API
- Better async integration
- Flexible storage backend

---

## Data Structures

### DashMap (Concurrent HashMap)

```toml
[dependencies]
dashmap = "6.1"
```

**Usage**:

```rust
use dashmap::DashMap;
use std::sync::Arc;

pub struct PartitionManager {
    partitions: Arc<DashMap<PartitionId, Partition>>,
}

impl PartitionManager {
    pub fn get(&self, id: &PartitionId) -> Option<Partition> {
        self.partitions.get(id).map(|r| r.clone())
    }
    
    pub fn insert(&self, id: PartitionId, partition: Partition) {
        self.partitions.insert(id, partition);
    }
}
```

### parking_lot (Fast Synchronization)

```toml
[dependencies]
parking_lot = "0.12"
```

**Why parking_lot?**
- Faster than std::sync
- Smaller memory footprint
- Drop-in replacement

**Usage**:

```rust
use parking_lot::{RwLock, Mutex};

pub struct Segment {
    data: RwLock<Vec<u8>>,
    index: Mutex<HashMap<u64, u64>>,
}

impl Segment {
    pub fn read(&self, offset: u64) -> Option<Vec<u8>> {
        let data = self.data.read();
        let index = self.index.lock();
        
        index.get(&offset).map(|&pos| {
            data[pos as usize..].to_vec()
        })
    }
}
```

### crossbeam (Lock-Free Structures)

```toml
[dependencies]
crossbeam = "0.8"
```

**Usage**:

```rust
use crossbeam::channel::{unbounded, Sender, Receiver};

pub struct ReplicationManager {
    tx: Sender<ReplicationRequest>,
    rx: Receiver<ReplicationRequest>,
}

impl ReplicationManager {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        Self { tx, rx }
    }
    
    pub fn replicate(&self, req: ReplicationRequest) {
        self.tx.send(req).unwrap();
    }
}
```

---

## Error Handling

### thiserror (Library Errors)

```toml
[dependencies]
thiserror = "1.0"
```

**Usage**:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PyralogError {
    #[error("invalid offset: {0}")]
    InvalidOffset(u64),
    
    #[error("partition not found: {0}")]
    PartitionNotFound(String),
    
    #[error("not the leader, redirect to {0}")]
    NotLeader(String),
    
    #[error("quorum not available")]
    QuorumNotAvailable,
    
    #[error("epoch sealed: {0}")]
    EpochSealed(u64),
    
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    
    #[error("serialization error")]
    Serialization(#[from] bincode::Error),
}

pub type Result<T> = std::result::Result<T, PyralogError>;
```

### anyhow (Application Errors)

```toml
[dependencies]
anyhow = "1.0"
```

**Usage** (for main.rs, tests):

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config("config.toml")
        .context("Failed to load configuration")?;
    
    let server = PyralogServer::new(config)
        .context("Failed to initialize server")?;
    
    server.start()
        .context("Failed to start server")?;
    
    Ok(())
}
```

---

## Observability

### tracing (Structured Logging)

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"
```

**Usage**:

```rust
use tracing::{info, warn, error, debug, instrument};

#[instrument(skip(self))]
pub async fn append_record(&self, record: Record) -> Result<LogOffset> {
    debug!("Appending record with key: {:?}", record.key);
    
    let offset = self.sequencer.next_offset().await?;
    info!(offset = %offset, "Assigned offset to record");
    
    match self.storage.append(record).await {
        Ok(offset) => {
            info!(offset = %offset, "Record appended successfully");
            Ok(offset)
        }
        Err(e) => {
            error!(error = %e, "Failed to append record");
            Err(e)
        }
    }
}

// Initialize
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();
}
```

### prometheus (Metrics)

```toml
[dependencies]
prometheus = "0.13"
lazy_static = "1.5"
```

**Usage**:

```rust
use prometheus::{Counter, Histogram, IntGauge, Registry};
use lazy_static::lazy_static;

lazy_static! {
    static ref RECORDS_WRITTEN: Counter = Counter::new(
        "pyralog_records_written_total",
        "Total number of records written"
    ).unwrap();
    
    static ref WRITE_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "pyralog_write_latency_seconds",
            "Write latency distribution"
        )
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: IntGauge = IntGauge::new(
        "pyralog_active_connections",
        "Number of active client connections"
    ).unwrap();
}

pub fn record_metrics() {
    RECORDS_WRITTEN.inc();
    WRITE_LATENCY.observe(0.001); // 1ms
    ACTIVE_CONNECTIONS.set(42);
}

// Metrics endpoint
pub fn metrics_handler() -> String {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

### opentelemetry (Distributed Tracing)

```toml
[dependencies]
opentelemetry = "0.24"
opentelemetry-otlp = "0.17"
tracing-opentelemetry = "0.25"
```

**Usage**:

```rust
use opentelemetry::{global, trace::Tracer};
use tracing_subscriber::layer::SubscriberExt;

fn init_telemetry() -> Result<()> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer);
    
    let subscriber = tracing_subscriber::registry()
        .with(telemetry);
    
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

---

## Testing

### tokio-test

```toml
[dev-dependencies]
tokio-test = "0.4"
```

**Usage**:

```rust
use tokio_test::{assert_ready, assert_pending, task};

#[tokio::test]
async fn test_write_cache() {
    let cache = WriteCache::new(1024);
    
    let record = Record::new(b"key", b"value");
    let offset = cache.append(record).await.unwrap();
    
    assert_eq!(offset, LogOffset::new(0));
}
```

### proptest (Property-Based Testing)

```toml
[dev-dependencies]
proptest = "1.5"
```

**Usage**:

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
    
    #[test]
    fn test_partition_hash(key in "\\PC*") {
        let partition = hash_partition(&key, 16);
        assert!(partition < 16);
    }
}
```

### criterion (Benchmarking)

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }

[[bench]]
name = "log_benchmark"
harness = false
```

**Usage**:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("write");
    
    for size in [256, 1024, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                b.iter(|| {
                    let record = Record::new(
                        Some(vec![0u8; 32]),
                        vec![0u8; size],
                    );
                    black_box(record);
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_write);
criterion_main!(benches);
```

### mockall (Mocking)

```toml
[dev-dependencies]
mockall = "0.13"
```

**Usage**:

```rust
use mockall::{automock, predicate::*};

#[automock]
#[async_trait]
pub trait Storage {
    async fn append(&self, record: Record) -> Result<LogOffset>;
    async fn read(&self, offset: LogOffset) -> Result<Record>;
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockStorage::new();
    
    mock.expect_append()
        .with(eq(record.clone()))
        .times(1)
        .returning(|_| Ok(LogOffset::new(100)));
    
    let offset = mock.append(record).await.unwrap();
    assert_eq!(offset, LogOffset::new(100));
}
```

---

## Performance & Profiling

### pprof (CPU Profiling)

```toml
[dev-dependencies]
pprof = { version = "0.13", features = ["flamegraph", "criterion"] }
```

**Usage with Criterion**:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

fn bench(c: &mut Criterion) {
    // benchmarks...
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = bench
}
criterion_main!(benches);
```

### jemalloc (Memory Allocator)

```toml
[dependencies]
tikv-jemallocator = "0.6"
```

**Usage**:

```rust
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() {
    // Your program now uses jemalloc
}
```

**Why jemalloc?**
- Better performance for multi-threaded apps
- Lower memory fragmentation
- Production-ready (used by Redis, Firefox)

---

## Utilities

### clap (CLI Parsing)

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
```

**Usage**:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "pyralog")]
#[command(about = "Pyralog distributed log server")]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    
    #[arg(short, long, default_value = "9092")]
    port: u16,
    
    #[arg(short, long, default_value = "/var/lib/pyralog")]
    data_dir: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    Start,
    Stop,
    Status,
}

fn main() {
    let cli = Cli::parse();
    println!("Config: {}", cli.config);
}
```

### config (Configuration Management)

```toml
[dependencies]
config = "0.14"
```

**Usage**:

```rust
use config::{Config, File, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub replication: ReplicationConfig,
}

impl Settings {
    pub fn new() -> Result<Self> {
        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("PYRALOG"))
            .build()?;
        
        s.try_deserialize()
    }
}
```

### chrono (Date/Time)

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

**Usage**:

```rust
use chrono::{DateTime, Utc, Duration};

pub struct Record {
    pub timestamp: DateTime<Utc>,
    // ...
}

impl Record {
    pub fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            timestamp: Utc::now(),
            key,
            value,
        }
    }
    
    pub fn age(&self) -> Duration {
        Utc::now() - self.timestamp
    }
}
```

### humantime (Human-Readable Durations)

```toml
[dependencies]
humantime = "2.1"
```

**Usage**:

```rust
use humantime::parse_duration;

let timeout = parse_duration("5m30s")?; // 5 minutes 30 seconds
let retention = parse_duration("7d")?;  // 7 days
```

---

## Complete Cargo.toml Example

```toml
[package]
name = "pyralog"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
serde_json = "1.0"

# Networking
tonic = "0.12"
prost = "0.13"
hyper = { version = "1.5", features = ["full"] }

# Storage
memmap2 = "0.9"
rocksdb = "0.22"

# Data structures
dashmap = "6.1"
parking_lot = "0.12"
crossbeam = "0.8"
bytes = "1.7"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
prometheus = "0.13"
opentelemetry = "0.24"

# Consensus
raft = "0.7"
protobuf = "3.5"

# Utilities
clap = { version = "4.5", features = ["derive"] }
config = "0.14"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.10", features = ["v4", "serde"] }
lazy_static = "1.5"

# Allocator
tikv-jemallocator = "0.6"

[dev-dependencies]
# Testing
tokio-test = "0.4"
proptest = "1.5"
mockall = "0.13"

# Benchmarking
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }
pprof = { version = "0.13", features = ["flamegraph", "criterion"] }

[build-dependencies]
tonic-build = "0.12"
prost-build = "0.13"

[[bench]]
name = "log_benchmark"
harness = false

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.bench]
debug = true
```

---

## Library Selection Guidelines

### Performance-Critical Paths

For **hot paths** (write/read paths):
- ✅ Use `bincode` for serialization (fastest)
- ✅ Use `memmap2` for zero-copy reads
- ✅ Use `parking_lot` instead of `std::sync`
- ✅ Use `bytes::Bytes` for zero-copy buffer sharing
- ✅ Use `tokio-uring` on Linux for maximum I/O

### Reliability-Critical Paths

For **consensus and metadata**:
- ✅ Use battle-tested `raft` crate
- ✅ Use `rocksdb` for persistent state
- ✅ Use `thiserror` for structured errors
- ✅ Heavy instrumentation with `tracing`

### Developer Experience

For **maintainability**:
- ✅ Use `serde` derive macros
- ✅ Use `async-trait` for trait async methods
- ✅ Use `tracing` spans for debugging
- ✅ Use `mockall` for testing

---

## Version Pinning Strategy

### Conservative (Production)

```toml
# Pin exact versions
tokio = "=1.40.0"
serde = "=1.0.210"
```

### Liberal (Development)

```toml
# Use compatible versions
tokio = "1.40"
serde = "1.0"
```

### Recommendation

```toml
# Pin major + minor, allow patches
tokio = "~1.40"  # Allows 1.40.x
serde = "~1.0"   # Allows 1.0.x
```

---

## Dependency Audit

### cargo-audit (Security)

```bash
cargo install cargo-audit
cargo audit
```

### cargo-outdated (Updates)

```bash
cargo install cargo-outdated
cargo outdated
```

### cargo-deny (Policy Enforcement)

```bash
cargo install cargo-deny
cargo deny check
```

**Example `deny.toml`**:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "MIT-0",
    "Apache-2.0",
    "BSD-3-Clause",
    "CC0-1.0",
]

[bans]
multiple-versions = "warn"
```

---

## Common Patterns

### Pattern 1: Zero-Copy Data Flow

```rust
use bytes::Bytes;
use tokio::io::AsyncWriteExt;

pub async fn write_record(socket: &mut TcpStream, data: Bytes) -> Result<()> {
    // data is Bytes, can be cloned cheaply (reference counted)
    socket.write_all(&data).await?;
    
    // data can still be used after write
    cache.store(data.clone()).await?;
    
    Ok(())
}
```

### Pattern 2: Structured Concurrency

```rust
use tokio::task::JoinSet;

pub async fn replicate_to_all(&self, record: Record) -> Result<()> {
    let mut set = JoinSet::new();
    
    for node in &self.replicas {
        let record = record.clone();
        let node = node.clone();
        
        set.spawn(async move {
            node.send(record).await
        });
    }
    
    let mut success = 0;
    while let Some(result) = set.join_next().await {
        if result?.is_ok() {
            success += 1;
            if success >= self.write_quorum {
                return Ok(());
            }
        }
    }
    
    Err(PyralogError::QuorumNotAvailable)
}
```

### Pattern 3: Graceful Shutdown

```rust
use tokio::signal;
use tokio::sync::broadcast;

pub async fn run_server(server: PyralogServer) -> Result<()> {
    let (shutdown_tx, _) = broadcast::channel(1);
    let shutdown_rx = shutdown_tx.subscribe();
    
    // Spawn shutdown signal handler
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl-c");
        shutdown_tx.send(()).unwrap();
    });
    
    // Run server with shutdown signal
    server.serve(shutdown_rx).await
}
```

---

## Summary

### Must-Have Crates

| Category | Library | Why |
|----------|---------|-----|
| Async | `tokio` | Industry standard, best ecosystem |
| Serialization | `bincode` | Fastest binary format |
| Networking | `tonic` | High-performance gRPC |
| Storage | `memmap2` | Zero-copy file I/O |
| Errors | `thiserror` | Structured error types |
| Logging | `tracing` | Structured, async-aware |
| Metrics | `prometheus` | Industry standard |
| Testing | `criterion` | Statistical benchmarking |

### Optimization Priorities

1. **Hot path**: `bincode` + `memmap2` + `parking_lot`
2. **Networking**: `tonic` + `hyper`
3. **Observability**: `tracing` + `prometheus` + `opentelemetry`
4. **Testing**: `tokio-test` + `proptest` + `criterion`
5. **Memory**: `jemalloc` allocator

### Resources

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Are We Async Yet?](https://areweasyncyet.rs/)
- [Blessed.rs](https://blessed.rs/) - Unofficial guide to Rust ecosystem

---

**Related Documentation:**
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guide

