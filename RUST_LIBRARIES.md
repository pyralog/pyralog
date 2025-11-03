# 🦀 Rust Libraries for Pyralog

> **Production-Ready Rust Ecosystem for Distributed Systems**

Battle-tested libraries for building high-performance, distributed log systems.

---

## 📖 Table of Contents

### Foundation
- [Quick Reference](#-quick-reference) - Essential crates at a glance
- [Core Stack](#-core-stack) - Must-have dependencies
- [Complete Cargo.toml](#-production-cargotoml) - Ready-to-use configuration

### By Category
- [Async Runtime](#-async-runtime) - Tokio, async-std
- [Serialization](#-serialization) - bincode, JSON
- [Networking](#-networking) - JSON-RPC/WebSocket, Arrow Flight, HTTP
- [Storage & I/O](#️-storage--io) - mmap, io_uring, RocksDB
- [Concurrency](#-concurrency--data-structures) - DashMap, parking_lot, crossbeam
- [Observability](#-observability) - tracing, prometheus, OpenTelemetry
- [Testing & Benchmarks](#-testing--benchmarks) - criterion, proptest, mockall

### Advanced
- [Consensus](#-consensus--replication) - Raft implementations
- [Performance](#-performance--profiling) - Profiling, jemalloc
- [Patterns](#-common-patterns) - Zero-copy, shutdown, replication

---

## ⚡ Quick Reference

**Essential crates** for Pyralog:

| Purpose | Library | Version | Why |
|---------|---------|---------|-----|
| **Async** | `tokio` | 1.40 | Industry standard, best ecosystem |
| **Serialization** | `bincode` | 1.3 | Fastest binary format |
| **Networking** | `tokio-tungstenite` | 0.21 | JSON-RPC/WebSocket (<5ms latency) |
| **Storage** | `memmap2` | 0.9 | Zero-copy file I/O |
| **Errors** | `thiserror` | 1.0 | Structured errors for libraries |
| **Logging** | `tracing` | 0.1 | Async-aware structured logging |
| **Metrics** | `prometheus` | 0.13 | Industry standard metrics |
| **Testing** | `criterion` | 0.5 | Statistical benchmarking |
| **Concurrency** | `parking_lot` | 0.12 | Faster than std::sync |
| **Consensus** | `raft` | 0.7 | Battle-tested Raft |

---

## 🏗️ Core Stack

**Minimal production-ready dependencies**:

```toml
[dependencies]
# Async runtime
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "sync", "fs", "net"] }

# Serialization  
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Networking
tokio-tungstenite = "0.21"  # JSON-RPC/WebSocket (primary)
arrow-flight = "53.0"       # Zero-copy data transport
hyper = "1.5"               # HTTP/REST

# Storage
memmap2 = "0.9"

# Concurrency
parking_lot = "0.12"
dashmap = "6.1"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Observability
tracing = "0.1"
prometheus = "0.13"

[dev-dependencies]
tokio-test = "0.4"
criterion = { version = "0.5", features = ["async_tokio"] }
```

---

## 🔄 Async Runtime

### Tokio (Recommended)

**Why**: Industry standard, best ecosystem, excellent performance

```toml
tokio = { version = "1.40", features = ["full"] }
# Or minimal:
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "sync", "fs", "net"] }
```

**Quick Start**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let file = tokio::fs::read("log.dat").await?;
    Ok(())
}
```

**Key Features**:
| Feature | Cargo Feature | Use For |
|---------|---------------|---------|
| Runtime | `rt-multi-thread` | Multi-threaded executor |
| Macros | `macros` | `#[tokio::main]`, `#[tokio::test]` |
| Sync | `sync` | Channels, mutexes, semaphores |
| File I/O | `fs` | Async file operations |
| Network | `net` | TCP/UDP sockets |
| Time | `time` | Sleep, interval, timeout |

### Alternative: async-std

```toml
async-std = { version = "1.12", features = ["attributes"] }
```

**When to use**: Simpler API, academic projects, std-like interface

---

## 📦 Serialization

**Comparison Table**:

| Format | Crate | Speed | Size | Schema | Use Case |
|--------|-------|-------|------|--------|----------|
| **bincode** | `bincode = "1.3"` | ⚡️⚡️⚡️ | Tiny | No | **Storage** (fastest) |
| **Protocol Buffers** | `prost = "0.13"` | ⚡️⚡️ | Small | Yes | **RPC** (cross-language) |
| **MessagePack** | `rmp-serde = "1.3"` | ⚡️⚡️ | Small | Partial | Interop, schema evolution |
| **JSON** | `serde_json = "1.0"` | ⚡️ | Large | No | Config, APIs (human-readable) |

### bincode (Recommended for Storage)

```toml
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
```

```rust
#[derive(Serialize, Deserialize)]
pub struct Record {
    offset: u64,
    key: Vec<u8>,
    value: Vec<u8>,
}

// Fastest serialization
let bytes = bincode::serialize(&record)?;
let record: Record = bincode::deserialize(&bytes)?;
```

**Why**: Fastest, smallest, perfect for hot paths

### Protocol Buffers (RPC)

```toml
prost = "0.13"

[build-dependencies]
prost-build = "0.13"
```

```proto
message Record {
    uint64 offset = 1;
    bytes key = 2;
    bytes value = 3;
}
```

**Why**: Cross-language, versioned schemas, battle-tested

### JSON (Config & APIs)

```rust
let config: Config = serde_json::from_str(&json_string)?;
let json = serde_json::to_string_pretty(&config)?;
```

**Why**: Human-readable, widely supported, easy debugging

---

## 🌐 Networking

| Protocol | Crate | Version | Best For |
|----------|-------|---------|----------|
| **JSON-RPC/WS** | `tokio-tungstenite` | 0.21 | **Primary RPC** (real-time, <5ms) |
| **Arrow Flight** | `arrow-flight` | 53.0 | Zero-copy data transport |
| **HTTP** | `hyper` | 1.5 | REST APIs, admin interfaces |
| **QUIC** | `quinn` | 0.11 | Low-latency replication |
| ~~**gRPC**~~ | ~~`tonic`~~ | ~~0.12~~ | ❌ **Not needed** (JSON-RPC/WS is simpler) |

### tokio-tungstenite (JSON-RPC over WebSocket - Primary)

```toml
tokio-tungstenite = "0.21"
serde_json = "1.0"
```

```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};

async fn handle_client(ws_stream: WebSocketStream<TcpStream>) {
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if msg.is_text() {
            let request: JsonRpcRequest = serde_json::from_str(msg.to_text()?)?;
            
            // Execute query
            let result = match request.method.as_str() {
                "query.execute" => execute_query(&request.params).await?,
                "query.stream" => stream_results(&request.params).await?,
                _ => return Err("Unknown method"),
            };
            
            // Send response
            let response = JsonRpcResponse {
                jsonrpc: "2.0",
                result: Some(result),
                id: request.id,
            };
            ws_stream.send(Message::Text(serde_json::to_string(&response)?)).await?;
        }
    }
}
```

**Why**: Simpler than gRPC, lower latency (<5ms vs 5-10ms), native browser support, Arrow IPC for binary

### arrow-flight (Zero-Copy Data Transport)

```toml
arrow-flight = "53.0"
tonic = "0.12"  # Required by arrow-flight
```

```rust
use arrow_flight::{FlightService, FlightDescriptor};

#[tonic::async_trait]
impl FlightService for PyralogFlightService {
    async fn do_get(&self, request: Request<Ticket>) -> Result<Response<Self::DoGetStream>, Status> {
        let batches = self.query(request.into_inner()).await?;
        Ok(Response::new(batches))
    }
}
```

**Why**: 3× faster than gRPC/Protobuf for large datasets, zero-copy Arrow IPC format

### Hyper (HTTP)

```toml
hyper = { version = "1.5", features = ["full"] }
```

**Use for**: REST APIs, metrics endpoints, admin consoles, health checks

### ~~Tonic (gRPC - Not Recommended)~~

> ⚠️ **Pyralog does not need gRPC**. Use JSON-RPC/WS instead:
> - **Simpler**: No Protobuf schemas or code generation
> - **Faster**: <5ms vs 5-10ms latency
> - **Better binary**: Arrow IPC (zero-copy) > Protobuf (serialize/deserialize)
> - **Browser native**: WebSocket everywhere (gRPC needs grpc-web proxy)
>
> See [JSONRPC_WEBSOCKET.md](JSONRPC_WEBSOCKET.md) for details.

```toml
# Only needed for Arrow Flight (which wraps gRPC)
tonic = "0.12"      # Dependency of arrow-flight
prost = "0.13"      # Dependency of arrow-flight
```

---

## 🗄️ Storage & I/O

| Storage | Crate | Version | Best For |
|---------|-------|---------|----------|
| **mmap** | `memmap2` | 0.9 | Zero-copy reads (hot path) |
| **io_uring** | `tokio-uring` | 0.5 | Linux max I/O performance |
| **RocksDB** | `rocksdb` | 0.22 | Metadata, indexes, state |
| **Sled** | `sled` | 0.34 | Pure Rust, embedded KV |

### memmap2 (Zero-Copy Reads)

```toml
memmap2 = "0.9"
```

```rust
use memmap2::MmapOptions;

let file = File::open("segment.log")?;
let mmap = unsafe { MmapOptions::new().map(&file)? };
let data = &mmap[offset..offset + len];  // Zero-copy!
```

**Why**: OS-managed caching, fastest reads, perfect for segments

### RocksDB (Metadata Storage)

```toml
rocksdb = "0.22"
```

```rust
let mut opts = Options::default();
opts.create_if_missing(true);
let db = DB::open(&opts, "metadata")?;

db.put(b"key", b"value")?;
let value = db.get(b"key")?;
```

**Why**: Battle-tested, ACID, excellent for metadata

### tokio-uring (Linux Only)

```toml
tokio-uring = "0.5"
```

**When**: Maximum I/O throughput on Linux (io_uring kernel support)

---

## ⚖️ Consensus & Replication

| Implementation | Crate | Version | Notes |
|----------------|-------|---------|-------|
| **Raft** | `raft` | 0.7 | Battle-tested, TiKV uses this |
| **Openraft** | `openraft` | 0.9 | Modern API, better async |

```toml
raft = "0.7"
protobuf = "3.5"
```

```rust
let config = Config { id, election_tick: 10, heartbeat_tick: 3, ..Default::default() };
let node = RawNode::new(&config, storage, &Default::default())?;
node.propose(vec![], data)?;
```

**Recommendation**: Use `raft` for production (proven), `openraft` for new projects

---

## 🔒 Concurrency & Data Structures

| Purpose | Crate | Version | Replaces |
|---------|-------|---------|----------|
| Concurrent HashMap | `dashmap` | 6.1 | `RwLock<HashMap>` |
| Fast locks | `parking_lot` | 0.12 | `std::sync::Mutex/RwLock` |
| Lock-free channels | `crossbeam` | 0.8 | `std::sync::mpsc` |

### DashMap (Concurrent HashMap)

```toml
dashmap = "6.1"
```

```rust
let partitions: Arc<DashMap<PartitionId, Partition>> = Arc::new(DashMap::new());
partitions.insert(id, partition);
let partition = partitions.get(&id);
```

**Why**: Lock-free reads, fast writes, perfect for shared state

### parking_lot (Faster Locks)

```toml
parking_lot = "0.12"
```

```rust
use parking_lot::{RwLock, Mutex};

let data = RwLock::new(vec![]);
data.read();   // No .unwrap() needed!
data.write();  // Cleaner API than std
```

**Why**: 2-3x faster than std, smaller footprint, drop-in replacement

### crossbeam (Lock-Free)

```toml
crossbeam = "0.8"
```

```rust
let (tx, rx) = crossbeam::channel::unbounded();
tx.send(msg)?;  // Lock-free MPMC channel
```

---

## ⚠️ Error Handling

| Crate | Use For | Example |
|-------|---------|---------|
| `thiserror` | **Library code** | Structured enum errors |
| `anyhow` | **Applications** | Quick error handling with context |

### thiserror (Libraries)

```rust
#[derive(Error, Debug)]
pub enum PyralogError {
    #[error("invalid offset: {0}")]
    InvalidOffset(u64),
    
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PyralogError>;
```

### anyhow (Applications)

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config("config.toml")
        .context("Failed to load config")?;
    Ok(())
}
```

**Rule**: `thiserror` for libs (public API), `anyhow` for bins (main.rs, tests)

---

## 📊 Observability

| Tool | Crate | Version | Purpose |
|------|-------|---------|---------|
| **Logging** | `tracing` | 0.1 | Structured, async-aware logs |
| **Metrics** | `prometheus` | 0.13 | Industry standard metrics |
| **Traces** | `opentelemetry` | 0.24 | Distributed tracing |

### tracing (Logging)

```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

```rust
#[instrument(skip(self))]
pub async fn append(&self, record: Record) -> Result<LogOffset> {
    info!(offset = %offset, "Record appended");
    Ok(offset)
}

// Init
tracing_subscriber::fmt().json().init();
```

**Why**: Async-aware, structured, zero-cost when disabled

### prometheus (Metrics)

```toml
prometheus = "0.13"
```

```rust
lazy_static! {
    static ref RECORDS_WRITTEN: Counter = 
        Counter::new("pyralog_records_written_total", "...").unwrap();
}

RECORDS_WRITTEN.inc();
```

**Why**: Industry standard, excellent Grafana integration

### OpenTelemetry (Optional)

```toml
opentelemetry = "0.24"
tracing-opentelemetry = "0.25"
```

**Use for**: Distributed tracing across services (advanced)

---

## ✅ Testing & Benchmarks

| Type | Crate | Version | Use For |
|------|-------|---------|---------|
| **Async tests** | `tokio-test` | 0.4 | `#[tokio::test]` |
| **Property tests** | `proptest` | 1.5 | Randomized testing |
| **Benchmarks** | `criterion` | 0.5 | Statistical benchmarking |
| **Mocking** | `mockall` | 0.13 | Mock traits |

### Quick Examples

**Async Test**:
```rust
#[tokio::test]
async fn test_append() {
    let cache = WriteCache::new(1024);
    let offset = cache.append(record).await.unwrap();
    assert_eq!(offset, LogOffset::new(0));
}
```

**Property Test**:
```rust
proptest! {
    #[test]
    fn test_serialization(offset in 0u64..1000000) {
        let bytes = bincode::serialize(&LogOffset::new(offset))?;
        let deserialized: LogOffset = bincode::deserialize(&bytes)?;
        assert_eq!(LogOffset::new(offset), deserialized);
    }
}
```

**Benchmark**:
```toml
[[bench]]
name = "log_benchmark"
harness = false
```

```rust
fn bench_write(c: &mut Criterion) {
    c.bench_function("write_1kb", |b| {
        b.iter(|| black_box(Record::new(vec![0u8; 1024])));
    });
}

criterion_group!(benches, bench_write);
criterion_main!(benches);
```

**Mock**:
```rust
#[automock]
trait Storage {
    async fn append(&self, record: Record) -> Result<LogOffset>;
}

let mut mock = MockStorage::new();
mock.expect_append().returning(|_| Ok(LogOffset::new(100)));
```

---

## ⚡ Performance & Profiling

| Tool | Crate | Version | Purpose |
|------|-------|---------|---------|
| **Profiling** | `pprof` | 0.13 | CPU profiling, flamegraphs |
| **Allocator** | `tikv-jemallocator` | 0.6 | Better malloc for multi-threaded |

### jemalloc (Recommended)

```rust
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

**Why**: 2-10% faster, less fragmentation, used by Redis/Firefox

### pprof (Optional)

```toml
pprof = { version = "0.13", features = ["flamegraph"] }
```

**Use with criterion** for flamegraph generation

---

## 🛠️ Utilities

| Purpose | Crate | Version | Example |
|---------|-------|---------|---------|
| **CLI** | `clap` | 4.5 | Arg parsing with derive macros |
| **Config** | `config` | 0.14 | Load from files + env vars |
| **DateTime** | `chrono` | 0.4 | Timestamps, durations |
| **Bytes** | `bytes` | 1.7 | Zero-copy buffer sharing |
| **UUID** | `uuid` | 1.10 | Unique identifiers |

### Quick Examples

**CLI**:
```rust
#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: String,
}

let cli = Cli::parse();
```

**Config**:
```rust
let s = Config::builder()
    .add_source(File::with_name("config/default"))
    .add_source(Environment::with_prefix("PYRALOG"))
    .build()?;
```

**DateTime**:
```rust
let timestamp = Utc::now();
let duration = timestamp - earlier;
```

---

## 📋 Production Cargo.toml

**Complete production-ready configuration**:

```toml
[package]
name = "pyralog"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "sync", "fs", "net"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"

# Networking
tokio-tungstenite = "0.21"  # JSON-RPC/WebSocket (primary)
arrow-flight = "53.0"       # Zero-copy data transport (includes tonic)
hyper = "1.5"               # HTTP/REST

# Storage
memmap2 = "0.9"
rocksdb = "0.22"

# Concurrency
dashmap = "6.1"
parking_lot = "0.12"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
prometheus = "0.13"

# Consensus
raft = "0.7"

# Allocator
tikv-jemallocator = "0.6"

[dev-dependencies]
tokio-test = "0.4"
proptest = "1.5"
criterion = { version = "0.5", features = ["async_tokio"] }

[build-dependencies]
# (none needed - JSON-RPC/WS has no code generation)

[[bench]]
name = "benchmarks"
harness = false

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 🎯 Selection Guidelines

### Hot Path (Performance-Critical)
- ✅ `bincode` for serialization
- ✅ `memmap2` for zero-copy reads
- ✅ `parking_lot` over `std::sync`
- ✅ `bytes::Bytes` for buffer sharing
- ✅ `tokio-uring` on Linux

### Cold Path (Reliability-Critical)
- ✅ `raft` for consensus (battle-tested)
- ✅ `rocksdb` for persistent state
- ✅ `thiserror` for structured errors
- ✅ Heavy `tracing` instrumentation

### Version Strategy
```toml
# Recommended: Pin major+minor, allow patches
tokio = "~1.40"  # Allows 1.40.x
serde = "~1.0"   # Allows 1.0.x
```

### Security Auditing
```bash
cargo install cargo-audit cargo-deny
cargo audit        # Check for vulnerabilities
cargo deny check   # Enforce license/ban policies
```

---

## 🔧 Common Patterns

### Zero-Copy Data Flow
```rust
let data = Bytes::from(vec![0u8; 1024]);
socket.write_all(&data).await?;
cache.store(data.clone()).await?;  // Cheap clone (refcounted)
```

### Structured Concurrency (Quorum)
```rust
let mut set = JoinSet::new();
for node in &replicas {
    set.spawn(async move { node.send(record.clone()).await });
}

let mut success = 0;
while let Some(result) = set.join_next().await {
    if result?.is_ok() && {success += 1; success >= quorum} {
        return Ok(());
    }
}
```

### Graceful Shutdown
```rust
let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
tokio::spawn(async move {
    signal::ctrl_c().await.unwrap();
    shutdown_tx.send(()).unwrap();
});
server.serve(shutdown_rx).await
```

---

## 📚 Resources

**Learning**:
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Blessed.rs](https://blessed.rs/) - Ecosystem guide

**Related Docs**:
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [PERFORMANCE.md](PERFORMANCE.md) - Tuning guide
- [CONTRIBUTING.md](CONTRIBUTING.md) - Development guide

---

**License**: CC0-1.0 (Public Domain)

