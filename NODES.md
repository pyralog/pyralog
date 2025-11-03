# Pyralog Node Architecture

**Understanding the two-tier node hierarchy within a Pyralog Cluster**

> **Note**: This document describes the node architecture **within a single Pyralog Cluster** (one datacenter/region).  
> For information about multiple Pyralog Clusters forming a Pyralog Network, see [CLUSTER_VS_NETWORK.md](CLUSTER_VS_NETWORK.md).

---

## Overview

Within a **Pyralog Cluster**, there is a **two-tier architecture** with distinct node types serving different roles:

```
┌─────────────────────────────────────────────────────────┐
│              Application Layer                           │
│                   (Clients)                              │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│           🔺 Pyramid Nodes (Pyralog Cluster)            │
│         Storage, Consensus & Compute Layer              │
│  • Store data (LSM trees, segments, indexes)            │
│  • Run consensus (Raft per partition)                   │
│  • Serve queries (read/write operations)                │
│  • Execute Batuta programs                              │
│  • Run actor systems                                    │
└────────────────────────┬────────────────────────────────┘
                         │
                         │ Coordination requests
                         │
┌────────────────────────▼────────────────────────────────┐
│        🗿 Obelisk Nodes (☀️ Pharaoh Network)            │
│              Coordination Layer                          │
│  • Generate Scarab IDs (unique identifiers)             │
│  • Maintain crash-safe atomic counters                  │
│  • Provide distributed coordination                     │
│  • No data storage (lightweight)                        │
└─────────────────────────────────────────────────────────┘
```

---

## 🔺 Pyramid Nodes

**Pyramid nodes** are the main **Pyralog cluster nodes** that store data, run consensus, and serve client requests.

### Responsibilities

1. **Data Storage**
   - LSM tree segments (1GB each)
   - PPHM indexes (O(1) lookups)
   - Write-ahead logs (WAL)
   - Tiered storage (local SSD → S3/GCS)

2. **Query Execution**
   - Read requests (offset-based or indexed)
   - Write requests (append operations)
   - SQL queries (DataFusion)
   - Batuta program execution

3. **Consensus & Replication**
   - Raft consensus per partition
   - Chain replication (Ouroboros Circle)
   - ISR (In-Sync Replica) tracking
   - Leader election and failover

4. **Actor System**
   - Location-transparent actors
   - Supervision trees
   - Message routing
   - Topology-level reactivity

### Architecture

```rust
pub struct PyramidNode {
    /// Node ID (unique in cluster)
    node_id: NodeId,
    
    /// Partitions owned by this node
    partitions: HashMap<PartitionId, Partition>,
    
    /// LSM storage engine
    storage: LsmStorage,
    
    /// Raft consensus (per partition)
    raft: HashMap<PartitionId, RaftNode>,
    
    /// Connection to Pharaoh Network
    pharaoh_client: PharaohClient,
    
    /// Actor system
    actor_system: ActorSystem,
    
    /// Shen Ring coordinator
    shen_ring: ShenRing,
}
```

### Configuration

```toml
[pyramid]
node_id = "pyramid-1"
listen_addr = "0.0.0.0:9092"
data_dir = "/var/lib/pyralog/data"

# Storage
segment_size = "1GB"
segment_cache_size = "16GB"

# Pharaoh Network connection
pharaoh_endpoints = [
    "obelisk-1:7070",
    "obelisk-2:7070",
    "obelisk-3:7070"
]

# Consensus
raft_election_timeout_ms = 300
raft_heartbeat_interval_ms = 100

# Replication
replication_factor = 3
min_in_sync_replicas = 2
```

---

## 🗿 Obelisk Nodes

**Obelisk nodes** form the **☀️ Pharaoh Network**, a lightweight coordination layer that provides distributed primitives without storing user data.

### Responsibilities

1. **Scarab ID Generation**
   - Generate globally unique, time-ordered 64-bit IDs
   - ~1-2μs latency per ID
   - Crash-safe atomic counters using sparse files
   - No coordination required across nodes

2. **Distributed Coordination**
   - Session ID allocation for exactly-once semantics
   - Epoch management for partition leadership
   - Transaction ID generation
   - Timestamp service (distributed TSO)

3. **Lightweight Consensus**
   - Raft for counter allocation (small metadata only)
   - Only stores coordination metadata
   - No data segments or indexes
   - Fast failover (<100ms)

### Architecture

```rust
pub struct ObeliskNode {
    /// Node ID (unique in Pharaoh Network)
    node_id: NodeId,
    
    /// Crash-safe atomic counters (sparse files)
    counters: HashMap<CounterId, AtomicCounter>,
    
    /// Raft consensus for counter allocation
    raft: RaftNode,
    
    /// Counter cache (hot counters)
    cache: LruCache<CounterId, Counter>,
    
    /// Sparse file storage
    storage_dir: PathBuf,
}

pub struct AtomicCounter {
    /// Counter ID
    id: CounterId,
    
    /// Current value (memory-mapped from sparse file)
    value: AtomicU64,
    
    /// Sparse file handle
    file: File,
    
    /// Memory mapping
    mmap: MmapMut,
}
```

### Configuration

```toml
[obelisk]
node_id = "obelisk-1"
listen_addr = "0.0.0.0:7070"
data_dir = "/var/lib/pharaoh/counters"

# Sparse file configuration
counter_file_size = "1TB"  # Sparse, doesn't use actual space
fsync_on_increment = false  # Trust OS page cache

# Raft (small cluster)
raft_election_timeout_ms = 100
raft_heartbeat_interval_ms = 30

# No data segments - only counters!
```

---

## The Pharaoh Network (☀️)

The **Pharaoh Network** is the collective name for the cluster of 🗿 Obelisk nodes.

### Purpose

Provides **distributed coordination primitives** without being a bottleneck:

1. **Decentralized ID Generation**
   - Each Pyramid node can request IDs from any Obelisk node
   - No single point of failure
   - Linear scalability (add more Obelisk nodes)

2. **Crash-Safe Counters**
   - Sparse file storage survives node crashes
   - Instant recovery (memory-map existing file)
   - No replay needed

3. **High Throughput**
   - 4B+ operations/sec across network
   - ~1-2μs per counter increment
   - Minimal latency overhead

### Network Topology

```
Pyramid Cluster (100s of nodes):
┌─────────┐  ┌─────────┐  ┌─────────┐
│Pyramid 1│  │Pyramid 2│  │Pyramid N│
└────┬────┘  └────┬────┘  └────┬────┘
     │            │            │
     └────────────┼────────────┘
                  │
         ╔════════▼════════╗
         ║  Pharaoh Network ║
         ╚════════╤════════╝
                  │
     ┌────────────┼────────────┐
     │            │            │
┌────▼────┐  ┌───▼─────┐  ┌───▼─────┐
│Obelisk 1│  │Obelisk 2│  │Obelisk 3│
└─────────┘  └─────────┘  └─────────┘

Key:
- Obelisk nodes: Scales horizontally (lightweight, coordination)
- Pyramid nodes: Scales horizontally (heavy, storage/consensus/compute)
```

### Separation of Concerns

| Concern | Obelisk Nodes | Pyramid Nodes |
|---------|---------------|---------------|
| **Data Storage** | ❌ No | ✅ Yes (LSM segments) |
| **ID Generation** | ✅ Yes (Scarab IDs) | ❌ Request from Obelisk |
| **Query Execution** | ❌ No | ✅ Yes (reads/writes) |
| **Consensus** | ✅ Small Raft (metadata only) | ✅ Large Raft (per partition) |
| **Actor System** | ❌ No | ✅ Yes |
| **Resource Usage** | Low (CPU, memory) | High (CPU, memory, disk) |
| **Scaling** | Horizontal (lightweight) | Horizontal (heavy) |

---

## Interaction Patterns

### Pattern 1: Write Request

```
1. Client → Pyramid Node
   "Write record to partition X"

2. Pyramid → Obelisk (Pharaoh Network)
   "Generate session_id for exactly-once"
   
3. Obelisk → Pyramid
   "session_id = 12345 (from counter #7)"

4. Pyramid → Pyramid
   "Check dedup cache (session_id, sequence)"
   
5. Pyramid → Storage
   "Append to LSM tree"
   
6. Pyramid → Client
   "Success, offset = 1000"
```

### Pattern 2: Partition Leadership Change

```
1. Pyramid Leader Fails
   (detected via Raft heartbeat timeout)

2. Pyramid Follower Promoted
   "I'm the new leader for partition X"

3. New Leader → Obelisk (Pharaoh Network)
   "Allocate new epoch for partition X"
   
4. Obelisk → New Leader
   "epoch = 42 (from counter #3)"

5. New Leader → Raft Consensus
   "Propose: Activate epoch 42"
   
6. Raft → New Leader
   "Committed: epoch 42 active"

7. New Leader
   "Ready to accept writes with epoch 42"
```

### Pattern 3: Scarab ID Generation

```
1. Pyramid Node
   "Need unique ID for record"

2. Pyramid → Obelisk (any node in Pharaoh Network)
   "Generate Scarab ID"

3. Obelisk
   "Atomic increment counter #5"
   "counter.fetch_add(1) → 99999"

4. Obelisk
   "Build Scarab ID:
    - timestamp: current_time_ms()
    - node_id: 2
    - sequence: 99999"

5. Obelisk → Pyramid
   "Scarab ID = 0x1A2B3C4D5E6F7890"

6. Pyramid
   "Store record with Scarab ID"
```

---

## Why Two Tiers?

### Without Separation (Monolithic)

```
❌ Problems:
- Coordination overhead on data nodes
- Counter state competes for memory with data
- Scaling coordination requires scaling data nodes
- No clear separation of concerns
```

### With Separation (Two-Tier)

```
✅ Benefits:

1. Independent Scaling
   - Add Pyramid nodes for storage capacity
   - Add Obelisk nodes for coordination throughput

2. Resource Efficiency
   - Obelisk nodes: Small, CPU-focused
   - Pyramid nodes: Large, storage-focused

3. Fault Isolation
   - Pharaoh Network failure doesn't affect existing data
   - Pyramid failure doesn't affect ID generation

4. Clear Contracts
   - Obelisk: "I generate IDs fast and crash-safe"
   - Pyramid: "I store data durably and serve queries"

5. Operational Simplicity
   - Upgrade Pharaoh Network without touching data
   - Upgrade Pyramid nodes without touching coordination
```

---

## Deployment Scenarios

### Small Cluster (Dev/Test)

```
3 Obelisk nodes + 3 Pyramid nodes

┌─────────────────────────────────────┐
│  Pharaoh Network (3 Obelisk nodes)  │
│  • 100MB memory each                │
│  • 1 CPU core each                  │
│  • 10GB disk (sparse files)         │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│  Pyramid Cluster (3 nodes)          │
│  • 64GB memory each                 │
│  • 16 CPU cores each                │
│  • 4TB NVMe each                    │
└─────────────────────────────────────┘
```

### Large Cluster (Production)

```
5 Obelisk nodes + 100 Pyramid nodes

┌─────────────────────────────────────┐
│  Pharaoh Network (5 Obelisk nodes)  │
│  • 500MB memory each                │
│  • 2 CPU cores each                 │
│  • 50GB disk (sparse files)         │
│  • 4B+ ops/sec total                │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│  Pyramid Cluster (100 nodes)        │
│  • 256GB memory each                │
│  • 64 CPU cores each                │
│  • 20TB NVMe each                   │
│  • 500M writes/sec total            │
└─────────────────────────────────────┘
```

---

## Naming Rationale

### 🗿 Obelisk

Ancient Egyptian **monuments** that:
- Stand tall and visible (coordination points visible to all)
- Are permanent and crash-safe (stone construction)
- Mark important locations (ID generation points)
- Lightweight but numerous (scales horizontally for coordination load)

### ☀️ Pharaoh Network

The **sun** that:
- Illuminates the entire kingdom (coordination for all)
- Is central but not a bottleneck (distributed coordination)
- Provides life-giving energy (enables the cluster to function)
- Rules over all (coordination layer above storage)

### 🔺 Pyramid

Ancient Egyptian **monuments** that:
- Store treasures (data storage)
- Are massive and numerous (many nodes)
- Built to last forever (durable storage)
- Represent the main achievement (the actual database)

---

## Migration from Single-Tier

If you're coming from a single-tier architecture:

```rust
// Old: Monolithic node
pub struct PyralogNode {
    storage: Storage,
    consensus: Consensus,
    counters: Counters,  // Everything in one!
}

// New: Two-tier separation
pub struct PyramidNode {
    storage: Storage,
    consensus: Consensus,
    pharaoh_client: PharaohClient,  // Delegates to Obelisk
}

pub struct ObeliskNode {
    counters: Counters,  // Focused role
    raft: SmallRaft,
}
```

**Migration Steps**:
1. Deploy Pharaoh Network (3 Obelisk nodes)
2. Update Pyramid nodes to use Pharaoh client
3. Migrate counter state to Obelisk nodes
4. Remove counter state from Pyramid nodes

---

## Summary

| Aspect | Obelisk Nodes | Pyramid Nodes |
|--------|---------------|---------------|
| **Layer** | Coordination (Pharaoh Network) | Storage, Consensus & Compute (Pyralog Cluster) |
| **Role** | ID generation, coordination | Data storage, consensus, query execution |
| **Count** | Scales for coordination load | Scales for storage/compute capacity |
| **Size** | Small (MB memory) | Large (GB memory, TB disk) |
| **Throughput** | 4B+ ops/sec | 500M writes/sec |
| **State** | Atomic counters (sparse files) | Data segments (LSM trees) |
| **Consensus** | Small Raft (metadata) | Large Raft (per partition) |
| **Scaling** | Independent | Independent |

**The key insight**: Separate coordination (Obelisk) from storage (Pyramid) for optimal scalability, resource efficiency, and operational simplicity.

---

🗿 **Obelisk** = Pharaoh Network Node (coordination)  
🔺 **Pyramid** = Pyralog Node (storage & compute)  
☀️ **Pharaoh Network** = Cluster of Obelisk nodes  
🔺 **Pyralog Cluster** = Cluster of Pyramid nodes

