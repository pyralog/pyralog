# 🔺 Pyralog Architecture

**A platform for secure, parallel, distributed, and decentralized computing**

> "Built to Last Millennia" - Inspired by Ancient Egyptian Engineering

---

## Table of Contents

### Foundation & Identity
1. [Overview](#overview)
2. [System Hierarchy](#system-hierarchy)
3. [Novel Coordination Primitives](#-novel-coordination-primitives)

### Core Architecture
4. [Two-Tier Node Architecture](#two-tier-node-architecture)
5. [Consensus Protocol: Dual Raft](#consensus-protocol-dual-raft)
6. [Storage Engine](#storage-engine)

### Multi-Model & Query
8. [Multi-Model Database](#multi-model-database)
9. [Query & Programming Languages](#query--programming-languages)
10. [Actor Model](#actor-model)

### Advanced Features
11. [Tensor Database](#tensor-database)
12. [Cryptographic Verification](#cryptographic-verification)
13. [Deduplication](#multi-layer-deduplication)

### Decentralization & Networking
14. [Decentralized Network](#decentralized-network)
15. [Network Protocol](#network-protocol)

### Performance & Scalability
16. [Replication System](#replication-system)
17. [Performance Optimizations](#performance-optimizations)
18. [Scalability](#scalability)

### Operations & Monitoring
19. [Monitoring & Observability](#monitoring--observability)
20. [Failure Scenarios](#failure-scenarios)

### Conclusion
21. [Architectural Philosophy](#architectural-philosophy)

---

## Overview

Pyralog is a **theoretically-founded, multi-model, actor-based, decentralized database platform** designed for:

- **High throughput**: 10M+ writes/sec (10 nodes, 100 partitions)
- **Low latency**: Sub-millisecond write latencies
- **Strong durability**: Configurable replication and persistence
- **High availability**: Tolerates node failures with fast failover
- **Horizontal scalability**: Add nodes to increase capacity linearly
- **Multi-model flexibility**: 6 data models in unified storage
- **Actor-first execution**: Distributed queries with supervision trees
- **Cryptographic safety**: Merkle trees, BLAKE3, zero-trust architecture
- **Decentralized network**: PoW, PoS, zk-proofs for global scale

### Platform Vision

Pyralog draws inspiration from **ancient Egyptian civilization** - a culture that perfected engineering excellence, mathematical precision, and distributed coordination. Like the pyramids that have stood for 4,500+ years, Pyralog is built for **permanence, precision, and power**.

### Key Innovations at a Glance

| Innovation | Type | Benefit |
|------------|------|---------|
| **🗿 Obelisk Sequencer** | Novel | Persistent atomic counter (file size = value), 28B ops/sec |
| **☀️ Pharaoh Network** | Novel | Lightweight coordination layer (Obelisk nodes) |
| **🪲 Scarab IDs** | Novel | Crash-safe globally unique 64-bit IDs |
| **🎼 Batuta Language** | Novel | Category Theory + Functional Relational Algebra |
| **Dual Raft Clusters** | Synthesized | Parallel failover (1000 partitions in 10ms) |
| **CopySet Replication** | Synthesized | Maximum cluster utilization (90%+) |
| **Multi-Model Database** | Synthesized | 6 data models, Arrow storage, Category Theory |
| **Actor-First Execution** | Synthesized | Supervision trees, location transparency |
| **Tensor Database** | Synthesized | Native ML/AI, Safetensors, DLPack |

### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| **Write throughput** | 10M+ records/sec | 10 nodes, 100 partitions |
| **Read throughput** | 30M+ records/sec | With RF=3 |
| **Write latency (p99)** | < 1ms | With write cache |
| **Read latency (p99)** | < 0.5ms | With memory-mapped I/O |
| **Leader election** | < 10ms | Per-partition Raft |
| **Replication lag** | < 100ms | With fast network |
| **Failover time** | < 10ms | Parallel per-partition |

### Egyptian Theme Explained

**Why Egyptian?** These architectural values directly mirror Pyralog's design:

| Egyptian Engineering | Pyralog Technology |
|---------------------|-------------------|
| Stone monuments (permanent) | Crash-safe primitives (Obelisk Sequencer) |
| Pharaohs (distributed authority) | Decentralized coordination (Pharaoh Network) |
| Scarab seals (unique identity) | Globally unique IDs (Scarab IDs) |
| Hieroglyphics (immutable records) | Append-only logs |
| Pyramids (layered architecture) | Two-tier nodes (Obelisk vs Pyramid) |

---

## System Hierarchy

Pyralog has two levels of architectural organization:

### Level 1: Cluster vs Network

#### 🔺 Pyralog Cluster (Single Datacenter)

A **single distributed computing cluster** within one datacenter or region:

- **Strong consistency**: Raft consensus per partition
- **Low latency**: Sub-millisecond writes, microsecond reads
- **High throughput**: 10M+ writes/sec per cluster
- **Use case**: Traditional distributed database applications

**Example**: E-commerce platform in US-East datacenter

```
Pyralog Cluster (US-East)
  ├─ Pharaoh Network Layer
  │    └─ Obelisk Nodes (ID generation, coordination)
  ├─ Pyralog Cluster Layer  
  │    └─ Pyramid Nodes (storage, consensus, compute)
  └─ Consistency: Strong (Raft)
```

#### 🌐 Pyralog Network (Multiple Clusters)

**Multiple Pyralog Clusters** forming a **Decentralized Autonomous Database**:

- **Global distribution**: Clusters across multiple datacenters/regions/continents
- **Eventual consistency**: Byzantine fault tolerance
- **Decentralized consensus**: PoW, PoS, zk-proofs
- **Use case**: Global-scale, trustless, decentralized applications

**Example**: Decentralized social network across 5 continents

```
Pyralog Network (Global)
  ├─ Pyralog Cluster (US-East)
  ├─ Pyralog Cluster (EU-West)
  ├─ Pyralog Cluster (Asia-Pacific)
  ├─ Pyralog Cluster (South America)
  └─ Pyralog Cluster (Africa)
  
  Consistency: Eventual (Byzantine fault tolerant)
  Consensus: PoW / PoS / zk-proofs
```

**See also**: [DECENTRALIZED.md](DECENTRALIZED.md) for cluster vs network details, [DADBS.md](DADBS.md) for Decentralized Autonomous Database Systems.

### Level 2: Two-Tier Node Architecture (within a cluster)

Within a single Pyralog Cluster, there are two types of nodes:

#### ☀️ Pharaoh Network (Coordination Layer)

**Lightweight coordinators** running **Obelisk Nodes**:

- **Purpose**: Scarab ID generation, sequencing, lightweight coordination
- **State**: Stateless or minimal state (sparse files for counters)
- **Consensus**: None (coordination-free)
- **Throughput**: Millions of ops/sec per node
- **Storage**: Sparse files (counter = file size)

**Example**: ID generation service

```
Obelisk Node
  ├─ Scarab ID Generator
  ├─ Obelisk Sequencers (sparse file counters)
  ├─ No Raft consensus
  └─ Throughput: 28B ops/sec theoretical
```

#### 🔺 Pyralog Cluster (Storage/Consensus/Compute Layer)

**Heavy storage and compute nodes** running **Pyramid Nodes**:

- **Purpose**: Data storage, consensus, query execution, multi-model operations
- **State**: Full stateful (LSM-Tree, partitions, indexes)
- **Consensus**: Raft per partition
- **Throughput**: 100K+ writes/sec per partition
- **Storage**: LSM-Tree + hybrid (file references)

**Example**: Data storage and query execution

```
Pyramid Node
  ├─ LSM-Tree Storage Engine
  ├─ Raft Consensus (per partition)
  ├─ Multi-Model Data (6 models)
  ├─ Actor-Based Query Execution
  ├─ Tensor Operations (ML/AI)
  └─ Cryptographic Verification (Merkle trees)
```

**Visual Hierarchy**:

```
┌─────────────────────────────────────────────────┐
│   Pyralog Platform Hierarchy                    │
├─────────────────────────────────────────────────┤
│                                                 │
│  Level 1: Deployment Topology                   │
│  ───────────────────────────────                │
│    • Cluster: Single datacenter (Raft)          │
│    • Network: Multiple clusters (Byzantine FT)  │
│                                                 │
│  Level 2: Node Architecture (within cluster)    │
│  ─────────────────────────────────────          │
│    ☀️ Pharaoh Network                            │
│      └─ Obelisk Nodes (coordination)            │
│         • Scarab ID generation                  │
│         • Lightweight, stateless                │
│         • No Raft consensus                     │
│                                                 │
│    🔺 Pyralog Cluster                            │
│      └─ Pyramid Nodes (storage/consensus)       │
│         • LSM-Tree storage                      │
│         • Raft per partition                    │
│         • Multi-model data                      │
│         • Actor-based queries                   │
│                                                 │
└─────────────────────────────────────────────────┘
```

**See also**: [NODES.md](NODES.md) for detailed node architecture, [BRANDING.md](BRANDING.md) for platform hierarchy.

---

## 🎯 Novel Coordination Primitives

Pyralog introduces three **original innovations** for distributed coordination:

### 🗿 Obelisk Sequencer

**The key innovation**: A persistent atomic counter where **the file size IS the counter value**.

#### Concept

Traditional atomic counters are in-memory and lost on crash. Obelisk Sequencer uses **sparse files** on disk where the file size represents the counter value:

```rust
pub struct ObeliskSequencer {
    file: File,  // Sparse file on disk
    // The file size IS the counter value!
    // No need for mmap, no SIGBUS risk
}

impl ObeliskSequencer {
    pub fn increment(&mut self, delta: u64) -> Result<u64> {
        // Read current size (counter value)
        let current = self.file.metadata()?.len();
        
        // Increment by extending file (atomic operation!)
        let new_value = current + delta;
        self.file.set_len(new_value)?;  // truncate() syscall
        
        Ok(new_value)
    }
    
    pub fn get(&self) -> Result<u64> {
        // Counter value = file size
        Ok(self.file.metadata()?.len())
    }
}
```

#### Key Properties

| Property | Benefit |
|----------|---------|
| **Crash-safe** | File system guarantees atomic size updates |
| **Coordination-free** | No Raft consensus needed between nodes |
| **High throughput** | 4+ billion ops/sec per coordinator type |
| **Simple** | Just truncate() system call |
| **No mmap** | Avoids SIGBUS on disk full |
| **Persistent** | Counter survives crashes |

#### Performance

```
Sequential counter increment:
  • Sparse file truncate(): 36 ns/op
  • 4+ billion ops/sec per coordinator type
  • Actual: ~1-2 million ops/sec (with fsync)
  • Batch mode: ~10 million ops/sec (async flush)
```

#### Use Cases

1. **Scarab ID Generation**: Monotonic sequence numbers for distributed IDs
2. **Schema Versioning**: Track schema changes with persistent counter
3. **Consumer Group Generations**: Rebalance tracking
4. **Partition Epochs**: Track partition leadership changes
5. **Exactly-Once Sessions**: Deduplication session IDs

#### Comparison with Alternatives

| Approach | Crash-Safe | Coordination-Free | Throughput | Complexity |
|----------|------------|-------------------|------------|------------|
| **Obelisk Sequencer** | ✅ Yes | ✅ Yes | 🔥 28B/sec | ✅ Simple |
| In-memory AtomicU64 | ❌ No | ✅ Yes | 🔥 1B/sec | ✅ Simple |
| Memory-mapped file | ⚠️ Risky (SIGBUS) | ✅ Yes | 🔥 500M/sec | ⚠️ Medium |
| Raft counter | ✅ Yes | ❌ No (consensus) | ⚠️ 10K/sec | ❌ Complex |

**Original innovation**: Not found in Kafka, LogDevice, TiKV, or other distributed logs.

**See also**: [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for Obelisk Sequencer details, blog posts [02](blog/02-obelisk-sequencer.md), [04](blog/04-28-billion-ops.md) (performance deep-dive).

### ☀️ Pharaoh Network

**Distributed coordination without centralized bottlenecks** using lightweight Obelisk nodes.

#### Architecture

```
┌──────────────────────────────────────────────────┐
│   Pharaoh Network (Coordination Layer)          │
├──────────────────────────────────────────────────┤
│                                                  │
│  Obelisk Node 1    Obelisk Node 2    Obelisk Node 3 │
│    ├─ Counter A       ├─ Counter D       ├─ Counter G │
│    ├─ Counter B       ├─ Counter E       ├─ Counter H │
│    └─ Counter C       └─ Counter F       └─ Counter I │
│                                                  │
│  Each node:                                      │
│    • Stateless or minimal state                  │
│    • Independent counter allocation              │
│    • No inter-node coordination                  │
│    • Millions of ops/sec                         │
│                                                  │
│  Clients connect directly:                       │
│    Client A → Node 1 (get Scarab ID)            │
│    Client B → Node 2 (get Scarab ID)            │
│    Client C → Node 3 (get Scarab ID)            │
│                                                  │
└──────────────────────────────────────────────────┘
```

#### Key Characteristics

| Characteristic | Description |
|----------------|-------------|
| **Lightweight** | Minimal CPU, memory, storage |
| **Stateless** | Only persistent counters (sparse files) |
| **Coordination-free** | No Raft, no consensus between nodes |
| **High availability** | Any node can serve requests |
| **Horizontal scaling** | Add more nodes for capacity |

#### Responsibilities

1. **Scarab ID Generation**: Monotonic IDs for distributed systems
2. **Sequence Number Allocation**: Batch allocation for efficiency
3. **Epoch Tracking**: Partition leadership generations
4. **Session Management**: Exactly-once deduplication sessions

#### Pharaoh Network vs Pyramid Cluster

| Aspect | Pharaoh Network (Obelisk) | Pyralog Cluster (Pyramid) |
|--------|--------------------------|---------------------------|
| **Purpose** | Coordination, ID generation | Storage, consensus, compute |
| **State** | Minimal (sparse files) | Full (LSM-Tree) |
| **Consensus** | None (coordination-free) | Raft per partition |
| **Throughput** | Millions/sec per node | 100K/sec per partition |
| **Storage** | Sparse files (~MB) | LSM-Tree (~TB) |
| **Complexity** | Low | High |

**Original innovation**: Two-tier separation of coordination and storage.

**See also**: [NODES.md](NODES.md) for node architecture, blog post [03](blog/03-pharaoh-network.md).

### 🪲 Scarab IDs

**Globally unique, time-ordered 64-bit identifiers** inspired by Twitter's Snowflake algorithm, enhanced with Obelisk Sequencers for crash-safety.

#### Structure

```
64-bit Scarab ID:
┌─────────────────┬──────────────┬───────────────┐
│  Timestamp      │  Machine ID  │  Sequence     │
│  (41 bits)      │  (10 bits)   │  (13 bits)    │
├─────────────────┼──────────────┼───────────────┤
│  Milliseconds   │  Node ID     │  Per-ms counter│
│  since epoch    │  (0-1023)    │  (0-8191)      │
└─────────────────┴──────────────┴───────────────┘

Example:
  Timestamp: 2025-11-03 12:34:56.789
  Machine: 123
  Sequence: 4567
  
  Result: 1730638496789 << 23 | 123 << 13 | 4567
        = 7123456789012345678
```

#### Key Properties

| Property | Benefit |
|----------|---------|
| **Time-ordered** | Sortable by creation time |
| **Globally unique** | 1024 nodes × 8192 IDs/ms = 8.3M IDs/ms |
| **Crash-safe** | Obelisk Sequencer for sequence counter |
| **Coordination-free** | No consensus between ID generators |
| **64-bit** | Fits in database integer columns |

#### Generation Process

```rust
pub struct ScarabIdGenerator {
    machine_id: u16,           // 0-1023
    sequencer: ObeliskSequencer, // Crash-safe counter
}

impl ScarabIdGenerator {
    pub fn generate(&mut self) -> Result<u64> {
        // 1. Get current timestamp (milliseconds since epoch)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        // 2. Get sequence number from Obelisk Sequencer
        //    (crash-safe persistent counter!)
        let sequence = self.sequencer.increment(1)? as u16;
        
        // 3. Combine: timestamp (41 bits) + machine (10 bits) + sequence (13 bits)
        let scarab_id = (timestamp << 23) | ((self.machine_id as u64) << 13) | (sequence as u64);
        
        Ok(scarab_id)
    }
}
```

#### Crash-Safety Guarantee

**Traditional Snowflake** (Twitter):
- In-memory sequence counter
- ❌ Counter resets to 0 on crash
- ❌ Risk of duplicate IDs after restart

**Scarab IDs** (Pyralog):
- Obelisk Sequencer (persistent counter)
- ✅ Counter persists across crashes
- ✅ No risk of duplicates after restart

#### Use Cases

1. **Distributed Event IDs**: Unique identifiers for events
2. **Message IDs**: Discord-style message IDs
3. **Transaction IDs**: Database transaction tracking
4. **Order IDs**: E-commerce order identifiers
5. **User IDs**: Globally unique user identifiers
6. **Log Record IDs**: Distributed log entries

#### Performance

```
Scarab ID generation:
  • 1-2 million IDs/sec (with Obelisk fsync)
  • 10 million IDs/sec (with async flush)
  • 8.3 million IDs/ms/node (theoretical max)
  • 8.5 billion IDs/sec (1024 nodes theoretical)
```

**Original innovation**: Snowflake algorithm + Obelisk Sequencer = crash-safe distributed IDs.

**See also**: [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for Scarab ID use cases, blog post [04](blog/04-28-billion-ops.md) (performance deep-dive).

---

## Two-Tier Node Architecture

Pyralog separates **coordination** (lightweight) from **storage/consensus/compute** (heavyweight) in a two-tier architecture.

### Obelisk Nodes (Pharaoh Network - Coordination Layer)

**Lightweight coordinators** for ID generation and sequencing.

#### Responsibilities

1. **Scarab ID Generation**: Globally unique identifiers
2. **Sequence Number Allocation**: For epochs, sessions, schemas
3. **Obelisk Counters**: Persistent atomic counters
4. **Lightweight Coordination**: Stateless or minimal state

#### Architecture

```rust
pub struct ObeliskNode {
    node_id: NodeId,
    
    // Scarab ID generator (with Obelisk Sequencer)
    scarab_generator: ScarabIdGenerator,
    
    // Obelisk counters for various use cases
    counters: HashMap<CounterId, ObeliskSequencer>,
    
    // Minimal metadata (cached from Pyramid nodes)
    cluster_metadata: Arc<RwLock<ClusterMetadata>>,
}

impl ObeliskNode {
    pub async fn handle_id_request(&mut self) -> Result<u64> {
        // Generate Scarab ID (crash-safe, no consensus)
        let scarab_id = self.scarab_generator.generate()?;
        Ok(scarab_id)
    }
    
    pub async fn allocate_sequence(&mut self, counter_id: CounterId, count: u64) -> Result<u64> {
        // Allocate sequence numbers from Obelisk Sequencer
        let sequencer = self.counters.get_mut(&counter_id)
            .ok_or(Error::CounterNotFound)?;
        
        let base = sequencer.increment(count)?;
        Ok(base)
    }
}
```

#### Storage

Obelisk nodes use **sparse files** for persistent counters:

```
/var/lib/pyralog/obelisk/
├── counters/
│   ├── scarab_sequence_123.sparse      # Scarab ID sequence (node 123)
│   ├── schema_version.sparse            # Schema versioning
│   ├── consumer_generation.sparse       # Consumer group generations
│   └── session_ids.sparse               # Exactly-once session IDs
└── metadata/
    └── cluster_cache.json               # Cached cluster metadata
```

**File size = counter value** (no content, just file metadata).

#### Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| **Throughput** | 1-2M IDs/sec | With fsync |
| **Throughput (async)** | 10M IDs/sec | Batched flush |
| **Latency (p99)** | < 1ms | Local counter increment |
| **Storage** | ~1 MB/counter | Sparse file metadata |
| **CPU** | < 5% | Minimal processing |
| **Memory** | < 100 MB | Lightweight state |

#### Failure Handling

- **Node failure**: Clients connect to another Obelisk node
- **Counter lost**: Recreate from last known value (monotonic guarantee)
- **No Raft**: Each node operates independently

**See also**: [NODES.md](NODES.md) for detailed Obelisk node design.

### Pyramid Nodes (Pyralog Cluster - Storage/Consensus/Compute Layer)

**Heavy storage and compute nodes** for data persistence and query execution.

#### Responsibilities

1. **Data Storage**: LSM-Tree storage engine
2. **Consensus**: Raft consensus per partition
3. **Multi-Model Data**: 6 data models (relational, document, graph, RDF, tensor, key-value)
4. **Query Execution**: Actor-based distributed queries
5. **Replication**: CopySet replication for durability
6. **Tensor Operations**: ML/AI workloads
7. **Cryptographic Verification**: Merkle trees, BLAKE3

#### Architecture

```rust
pub struct PyramidNode {
    node_id: NodeId,
    
    // Storage layer (LSM-Tree)
    storage: Arc<LogStorage>,
    
    // Raft consensus (per partition)
    raft_groups: HashMap<PartitionId, Arc<RaftNode>>,
    
    // Multi-model data
    arrow_storage: Arc<ArrowStore>,
    relational: Arc<RelationalEngine>,
    document: Arc<DocumentEngine>,
    graph: Arc<GraphEngine>,
    rdf: Arc<RDFEngine>,
    tensor: Arc<TensorEngine>,
    kv: Arc<KVEngine>,
    
    // Actor system for query execution
    actor_system: Arc<ActorSystem>,
    
    // Cryptographic verification
    merkle_trees: Arc<MerkleTreeManager>,
    
    // Replication
    replicator: Arc<Replicator>,
}

impl PyramidNode {
    pub async fn write(&self, record: Record) -> Result<LogOffset> {
        // 1. Write to LSM-Tree (local storage)
        let offset = self.storage.append(record.clone()).await?;
        
        // 2. Replicate to followers (CopySet)
        self.replicator.replicate(record, offset).await?;
        
        // 3. Update Merkle tree (cryptographic verification)
        self.merkle_trees.append_leaf(offset, &record).await?;
        
        Ok(offset)
    }
    
    pub async fn query(&self, query: Query) -> Result<QueryResult> {
        // Execute query as distributed actors
        let actor = QueryActor::new(query, self.actor_system.clone());
        actor.execute().await
    }
}
```

#### Storage Hierarchy

```
/var/lib/pyralog/pyramid/
├── raft/
│   ├── partition_0/               # Raft log for partition 0
│   ├── partition_1/               # Raft log for partition 1
│   └── ...
├── storage/
│   ├── partition_0/
│   │   ├── 00000000000000.log     # LSM-Tree segments
│   │   ├── 00000000000000.index
│   │   ├── 00000000001000.log
│   │   └── 00000000001000.index
│   └── ...
├── arrow/                         # Multi-model Arrow storage
│   ├── relational/
│   ├── document/
│   ├── graph/
│   ├── rdf/
│   ├── tensor/
│   └── kv/
├── merkle/                        # Merkle trees
│   ├── partition_0.merkle
│   └── ...
└── metadata/
    └── cluster_metadata.db        # RocksDB for metadata
```

#### Performance Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| **Write throughput** | 100K/sec/partition | Single partition |
| **Read throughput** | 300K/sec/partition | With RF=3 |
| **Write latency (p99)** | < 1ms | With cache |
| **Read latency (p99)** | < 0.5ms | With mmap |
| **Storage** | ~1 TB/node | LSM-Tree + Arrow |
| **CPU** | 50-80% | Query execution |
| **Memory** | 16-64 GB | Cache + indexes |

#### Failure Handling

- **Leader failure**: Per-partition Raft election (< 10ms)
- **Follower failure**: ISR tracking, continue with quorum
- **Disk failure**: Redirect to replicas, rebuild from object storage

**See also**: [NODES.md](NODES.md) for detailed Pyramid node design, [STORAGE.md](STORAGE.md) for storage layer.

### Interaction Between Tiers

```
┌──────────────────────────────────────────────────────┐
│   Client Request Flow                                │
├──────────────────────────────────────────────────────┤
│                                                      │
│  1. Client writes to Pyralog                         │
│     ├─ Needs unique ID for record                    │
│     └─ Connects to Obelisk Node (Pharaoh Network)   │
│                                                      │
│  2. Obelisk Node generates Scarab ID                 │
│     ├─ Uses Obelisk Sequencer (crash-safe)          │
│     ├─ Returns ID to client (< 1ms)                 │
│     └─ No consensus, no coordination                 │
│                                                      │
│  3. Client writes record with Scarab ID              │
│     ├─ Connects to Pyramid Node (Pyralog Cluster)   │
│     ├─ Record: { id: scarab_id, data: ... }         │
│     └─ Pyramid Node handles storage + replication   │
│                                                      │
│  4. Pyramid Node persists data                       │
│     ├─ Writes to LSM-Tree (local storage)           │
│     ├─ Replicates via CopySet                       │
│     ├─ Updates Merkle tree                          │
│     └─ Returns offset to client (< 1ms)             │
│                                                      │
│  Result: Fast, crash-safe, coordinated write! ✅    │
│                                                      │
└──────────────────────────────────────────────────────┘
```

**Benefits of Two-Tier Architecture**:

1. **Separation of Concerns**: ID generation ≠ data storage
2. **Independent Scaling**: Add Obelisk or Pyramid nodes independently
3. **Lightweight Coordination**: Obelisk nodes are simple, fast
4. **Heavy Computation**: Pyramid nodes handle complex queries
5. **Fault Isolation**: Obelisk failure doesn't affect storage

**See also**: [NODES.md](NODES.md), [BRANDING.md](BRANDING.md) for two-tier architecture details, diagrams [system-architecture.mmd](diagrams/system-architecture.mmd), [component-relationships.mmd](diagrams/component-relationships.mmd).

---

## Consensus Protocol: Dual Raft

Pyralog uses **two separate Raft clusters** for scalability: Global Raft (cluster-wide metadata) and Per-Partition Raft (partition-specific operations).

### Why Dual Raft?

**The Problem**: Single global Raft doesn't scale:

```
Single Global Raft Only:
  ✅ Simple architecture
  ✅ One consensus group to manage
  ❌ Bottleneck: All epoch changes through one log
  ❌ Contention: 10K partitions = 10K ops/sec to global Raft
  ❌ Slow failover: Global Raft must process all partitions

Dual Raft (Global + Per-Partition):
  ✅ Parallel: Partition failovers are independent
  ✅ Fast: Epoch change only needs partition replicas
  ✅ Scalable: 10K partitions = 10K independent Raft groups
  ⚠️  Complex: Each node in multiple Raft groups
  ⚠️  Overhead: More Raft state to manage
```

### Global Raft Cluster

**All nodes participate** in a single global Raft group for cluster-wide operations.

#### Responsibilities

1. **Cluster membership changes**: Add/remove nodes
2. **Partition creation/deletion**: Lifecycle management
3. **CopySet assignments**: Initial replica placement (per-partition mode)
4. **Configuration changes**: Cluster-wide settings

#### Example

```
Cluster: [N1, N2, N3, N4, N5]

Global Raft Group: All 5 nodes participate

Operations:
  • Add Node N6 → Propose to Global Raft → Committed → All nodes updated
  • Create Partition 10 → Propose to Global Raft → Assigns CopySet [N2, N4, N5]
  • Update Config → Propose to Global Raft → All nodes receive new config
```

#### Implementation

```rust
pub struct GlobalRaft {
    raft_node: Arc<RaftNode>,
    state_machine: Arc<GlobalStateMachine>,
}

#[derive(Debug, Clone)]
pub struct GlobalStateMachine {
    cluster_members: HashMap<NodeId, NodeMetadata>,
    partitions: HashMap<PartitionId, PartitionMetadata>,
    copysets: HashMap<PartitionId, Vec<NodeId>>,
}

impl GlobalRaft {
    pub async fn add_node(&self, node: NodeId, address: SocketAddr) -> Result<()> {
        // Propose membership change to Global Raft
        let proposal = Proposal::AddNode { node, address };
        self.raft_node.propose(proposal).await?;
        Ok(())
    }
    
    pub async fn create_partition(&self, partition: PartitionId) -> Result<Vec<NodeId>> {
        // Select CopySet for new partition
        let copyset = self.select_copyset(partition)?;
        
        // Propose partition creation to Global Raft
        let proposal = Proposal::CreatePartition { partition, copyset: copyset.clone() };
        self.raft_node.propose(proposal).await?;
        
        Ok(copyset)
    }
}
```

### Per-Partition Raft Clusters

**Only partition replicas participate** in each partition's Raft group for partition-specific operations.

#### Responsibilities

1. **Epoch activation**: Leader election for partition
2. **Epoch sealing**: Leadership transfer
3. **Partition-level failover**: Fast recovery without global coordination

#### Example

```
Cluster: [N1, N2, N3, N4, N5]
Partition 0 CopySet: [N1, N2, N3]
Partition 1 CopySet: [N2, N3, N4]
Partition 2 CopySet: [N3, N4, N5]

Per-Partition Raft Groups:
  • Partition 0: [N1, N2, N3] → Raft group for P0 only
  • Partition 1: [N2, N3, N4] → Raft group for P1 only
  • Partition 2: [N3, N4, N5] → Raft group for P2 only

Node N2 is a member of:
  • Global Raft: [N1, N2, N3, N4, N5]
  • Partition 0 Raft: [N1, N2, N3]
  • Partition 1 Raft: [N2, N3, N4]
  Total: 1 global + 2 partition Raft groups = 3 Raft instances
```

#### Implementation

```rust
pub struct PerPartitionRaft {
    partition_id: PartitionId,
    raft_node: Arc<RaftNode>,
    state_machine: Arc<PartitionStateMachine>,
}

#[derive(Debug, Clone)]
pub struct PartitionStateMachine {
    current_epoch: Epoch,
    leader: NodeId,
    last_committed_offset: LogOffset,
}

impl PerPartitionRaft {
    pub async fn activate_epoch(&self, epoch: Epoch) -> Result<()> {
        // Propose epoch activation to Partition Raft (only replicas vote!)
        let proposal = Proposal::ActivateEpoch { epoch };
        self.raft_node.propose(proposal).await?;
        Ok(())
    }
    
    pub async fn handle_leader_failure(&self) -> Result<NodeId> {
        // Fast election among partition replicas only
        self.raft_node.trigger_election().await?;
        
        // Wait for new leader
        let new_leader = self.raft_node.wait_for_leader().await?;
        
        // Activate new epoch
        let new_epoch = self.state_machine.current_epoch.next();
        self.activate_epoch(new_epoch).await?;
        
        Ok(new_leader)
    }
}
```

### Node Membership in Raft Clusters

Each node is a member of **1 global + N partition Raft clusters**:

```rust
pub struct NodeRaftMembership {
    node_id: NodeId,
    
    // 1. Global cluster (always)
    global_raft: Arc<GlobalRaft>,
    
    // 2. Per-partition clusters (for partitions this node replicates)
    partition_rafts: HashMap<PartitionId, Arc<PerPartitionRaft>>,
}

// Example: Node 2 in 5-node cluster
// Node 2 is a member of:
//   - Global Raft: [N1, N2, N3, N4, N5]
//   - Partition 0 Raft: [N1, N2, N3]  (N2 is replica)
//   - Partition 1 Raft: [N2, N3, N4]  (N2 is replica)
//   - Partition 5 Raft: [N2, N4, N5]  (N2 is replica)
//
// Total: 1 global + 3 partition Raft groups = 4 Raft instances
```

### Benefits of Dual Raft

#### Parallel Failover

```
1000 partitions fail over simultaneously:
  
  Single Global Raft:
    1000 epoch changes → Global Raft log
    Sequential processing → Slow!
    Latency: 1000 × 10ms = 10 seconds ❌
  
  Per-Partition Raft:
    1000 independent Raft groups
    Parallel processing → Fast!
    Latency: 10ms (same as one partition) ✅
```

#### Node Failure Example

```
Initial State:
  Global Raft: [N1, N2, N3, N4, N5]
  Partition 0: [N1, N2, N3] → Leader: N1
  Partition 1: [N2, N3, N4] → Leader: N2

N1 Fails:
──────────────────────────────────────────────────
1. Global Raft: Detects N1 down (heartbeat timeout)
   → Updates cluster membership (optional, for permanent failure)
   → No impact on other partitions!

2. Partition 0 Raft: [N1, N2, N3] detects leader failure
   → Election among [N2, N3] (N1 doesn't participate)
   → N2 becomes leader for Partition 0
   → N2 proposes new epoch to Partition 0 Raft
   → Committed when N2 + N3 agree (majority of 2/3)
   
   This is FAST because:
   ✅ Only 3 nodes involved (not all 5)
   ✅ Parallel with other partitions
   ✅ No global bottleneck

3. Partition 1: Unaffected! 
   → N2 is still leader
   → Continues normal operation
```

### Resource Management: Multiple Raft Groups

**Challenge**: Each node participates in 1 global + N partition Raft groups.

**Resource Usage Per Raft Group:**

```rust
pub struct RaftGroupResources {
    // Memory
    log_entries: Vec<LogEntry>,      // ~100 bytes per entry
    state: NodeState,                 // ~1 KB
    
    // Disk
    raft_log_file: File,              // Persistent log
    snapshot: Option<File>,           // Periodic snapshots
    
    // Network
    heartbeat_interval: Duration,     // 100ms default
    election_timeout: Duration,       // 300ms default
}
```

**Overhead Calculation:**

```
Example: 5-node cluster, RF=3, 1000 partitions

Per Node:
  Global Raft: 1 group
  Partition Rafts: ~600 groups (each node in ~60% of partitions)
  Total: ~601 Raft groups per node

Memory per Raft group: ~10 KB (state + recent entries)
Total memory: 601 × 10 KB = ~6 MB ✅ (negligible!)

Disk per Raft group: ~1 MB (log + snapshots)
Total disk: 601 × 1 MB = ~600 MB ✅ (acceptable)

Network: 601 × 2 heartbeats/sec = ~1200 msgs/sec ✅ (fine)
```

**Optimization: Raft Multiplexing**

```rust
// Batch heartbeats for multiple Raft groups
pub struct MultiRaftManager {
    groups: HashMap<PartitionId, Arc<PerPartitionRaft>>,
    
    pub async fn tick(&self) {
        // Single timer for all groups
        for (partition_id, raft) in &self.groups {
            raft.tick();
        }
    }
    
    pub async fn send_batched_heartbeats(&self, peer: NodeId) {
        // Batch heartbeats for all Raft groups to same peer
        let mut batch = Vec::new();
        for (partition_id, raft) in &self.groups {
            if raft.is_leader() {
                batch.push(HeartbeatMsg {
                    partition_id: *partition_id,
                    term: raft.current_term(),
                    commit_index: raft.commit_index(),
                });
            }
        }
        
        // Send one message instead of 600!
        self.send_to_peer(peer, batch).await;
    }
}
```

**Result**: 1000 partitions managed efficiently with minimal overhead! ✅

**See also**: [EPOCHS.md](EPOCHS.md) for epoch-based coordination, diagram [consensus.mmd](diagrams/consensus.mmd).

---

## Storage Engine

Pyralog uses a **hybrid storage architecture** combining native LSM-Tree for hot data with file references for cold data.

### LSM-Tree Architecture

**Log-Structured Merge Tree** for write-optimized storage.

#### Multi-Level Organization

```
┌──────────────────────────────────────────────────┐
│   LSM-Tree Structure                             │
├──────────────────────────────────────────────────┤
│                                                  │
│  Level 0: MemTable (Memory)                      │
│    • In-memory sorted tree                       │
│    • 16-64 MB size                               │
│    • Flush to Level 1 when full                  │
│                                                  │
│  Level 1: Immutable MemTable → SSTable           │
│    • Write to disk as sorted string table        │
│    • One SSTable = one segment                   │
│    • Max 10-20 SSTables                          │
│                                                  │
│  Level 2-N: Compacted SSTables                   │
│    • Merge overlapping SSTables                  │
│    • Each level 10× larger than previous         │
│    • Level 2: ~100-200 MB                        │
│    • Level 3: ~1-2 GB                            │
│    • Level 4: ~10-20 GB                          │
│                                                  │
└──────────────────────────────────────────────────┘
```

#### Write Path

```rust
pub struct LSMTree {
    memtable: Arc<RwLock<MemTable>>,
    immutable: Arc<RwLock<Vec<MemTable>>>,
    sstables: Arc<RwLock<Vec<SSTable>>>,
    wal: Arc<WriteAheadLog>,
}

impl LSMTree {
    pub async fn write(&self, key: &[u8], value: &[u8]) -> Result<()> {
        // 1. Write to WAL (crash safety)
        self.wal.append(key, value).await?;
        
        // 2. Write to MemTable (in-memory)
        let mut memtable = self.memtable.write();
        memtable.insert(key.to_vec(), value.to_vec());
        
        // 3. Check if MemTable is full
        if memtable.size() > 64 * 1024 * 1024 {  // 64 MB
            // Flush to disk as SSTable
            self.flush_memtable().await?;
        }
        
        Ok(())
    }
    
    async fn flush_memtable(&self) -> Result<()> {
        // 1. Freeze current MemTable
        let frozen = {
            let mut memtable = self.memtable.write();
            let frozen = std::mem::replace(&mut *memtable, MemTable::new());
            frozen
        };
        
        // 2. Add to immutable list
        self.immutable.write().push(frozen.clone());
        
        // 3. Write to disk as SSTable (async)
        let sstable = self.write_sstable(frozen).await?;
        
        // 4. Add to SSTable list (Level 1)
        self.sstables.write().push(sstable);
        
        // 5. Trigger compaction if needed
        self.maybe_compact().await?;
        
        Ok(())
    }
}
```

#### Read Path

```rust
impl LSMTree {
    pub async fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // 1. Check MemTable (most recent)
        if let Some(value) = self.memtable.read().get(key) {
            return Ok(Some(value.clone()));
        }
        
        // 2. Check immutable MemTables (recent, not yet flushed)
        for memtable in self.immutable.read().iter().rev() {
            if let Some(value) = memtable.get(key) {
                return Ok(Some(value.clone()));
            }
        }
        
        // 3. Check SSTables (disk, most recent first)
        for sstable in self.sstables.read().iter().rev() {
            // Use Bloom filter to skip SSTables that don't have the key
            if !sstable.bloom_filter.may_contain(key) {
                continue;
            }
            
            // Use sparse index to find approximate position
            if let Some(offset) = sstable.index.lookup(key) {
                // Read from disk (memory-mapped for zero-copy)
                if let Some(value) = sstable.read_at(offset, key).await? {
                    return Ok(Some(value));
                }
            }
        }
        
        // 4. Not found
        Ok(None)
    }
}
```

#### Compaction Strategies

1. **Size-Tiered Compaction**: Merge SSTables of similar size
2. **Leveled Compaction**: Merge overlapping SSTables across levels
3. **Deduplication Compaction**: Remove duplicates and tombstones

```rust
pub enum CompactionStrategy {
    SizeTiered,
    Leveled,
    Deduplication { mode: DeduplicationMode },
}

#[derive(Debug, Clone)]
pub enum DeduplicationMode {
    LastWriteWins,     // Keep most recent value
    Tombstone,         // Keep delete markers
    MVCC,              // Keep all versions
}

impl LSMTree {
    async fn compact(&self, strategy: CompactionStrategy) -> Result<()> {
        match strategy {
            CompactionStrategy::Leveled => {
                // Merge overlapping SSTables from Level N to Level N+1
                self.leveled_compaction().await?;
            }
            CompactionStrategy::Deduplication { mode } => {
                // Remove duplicates based on mode
                self.deduplication_compaction(mode).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

**See also**: [STORAGE.md](STORAGE.md) for detailed storage layer, diagram [lsm-storage.mmd](diagrams/lsm-storage.mmd).

### Hybrid Storage Architecture

**Combine native LSM-Tree (hot data) with file references (cold data)** for cost-effective storage.

#### Decision Matrix

| Data Type | Hot (LSM-Tree) | Cold (File Reference) |
|-----------|----------------|----------------------|
| **Recent records** | ✅ Fast random access | ❌ Too slow |
| **Old records** | ⚠️ Wastes space | ✅ Cost-effective |
| **Analytics tables** | ❌ Too large | ✅ Parquet files |
| **ML models** | ❌ Too large | ✅ Safetensors files |
| **Tensors** | ❌ Too large | ✅ Zarr files |
| **Documents** | ✅ For recent | ✅ For archival |

#### File References

Instead of storing large blobs in LSM-Tree, store **file paths** and memory-map files directly:

```rust
pub enum StorageValue {
    // Hot data: Stored in LSM-Tree
    Inline(Vec<u8>),
    
    // Cold data: File reference (memory-mapped on access)
    FileReference {
        path: PathBuf,         // /mnt/cold/analytics/table123.parquet
        offset: u64,           // Byte offset within file
        length: u64,           // Byte length
        format: ExternalFormat,  // Parquet, Safetensors, Zarr
    },
}

impl LSMTree {
    pub async fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let value = self.read_raw(key).await?;
        
        match value {
            StorageValue::Inline(data) => Ok(Some(data)),
            
            StorageValue::FileReference { path, offset, length, format } => {
                // Memory-map external file (zero-copy!)
                let mmap = unsafe { Mmap::map(&File::open(path)?)? };
                let data = &mmap[offset as usize..(offset + length) as usize];
                Ok(Some(data.to_vec()))
            }
        }
    }
}
```

#### Benefits

| Benefit | Description |
|---------|-------------|
| **Zero-copy** | Memory-map files directly, no data duplication |
| **Cost-effective** | 70-90% cost savings for cold data |
| **Native formats** | Use Parquet, Safetensors, Zarr directly |
| **Flexibility** | Hot data in LSM, cold data as files |

**Example**: Store 100 GB ML model as Safetensors file, reference it in LSM-Tree (1 KB metadata), access via memory-mapping (zero-copy).

**See also**: [STORAGE.md](STORAGE.md) for hybrid storage details, [DATA_FORMATS.md](DATA_FORMATS.md) for external formats, blog post [20](blog/20-lsm-arrow.md).

### Memory-Only Mode

**Ultra-fast ephemeral storage** for testing, caching, and real-time workloads.

#### Configuration

```rust
pub struct StorageConfig {
    pub mode: StorageMode,
}

pub enum StorageMode {
    Persistent {
        data_dir: PathBuf,
        wal_enabled: bool,
        fsync_mode: FsyncMode,
    },
    MemoryOnly {
        max_size: usize,         // Max memory usage
        eviction: EvictionPolicy,  // LRU, LFU, TTL
    },
}
```

#### Performance Characteristics

| Metric | Persistent Mode | Memory-Only Mode |
|--------|----------------|------------------|
| **Write throughput** | 100K/sec | 10M+/sec (100× faster) |
| **Write latency** | 1-10ms | 10-100μs (100× faster) |
| **Read latency** | 0.5-5ms | 0.1-1μs (10× faster) |
| **Durability** | ✅ Crash-safe | ❌ Lost on restart |
| **Storage** | Disk-based | RAM-based |

#### Use Cases

1. **Testing**: Fast test databases (no disk I/O)
2. **Caching**: Hot data cache (TTL-based eviction)
3. **Real-time**: Sub-millisecond latencies
4. **Ephemeral**: Session storage, temporary results
5. **Development**: Fast iteration cycles

**See also**: [MEMORY_ONLY_MODE.md](MEMORY_ONLY_MODE.md) for details, blog post [15](blog/15-memory-only.md).

---

## Multi-Model Database

Pyralog unifies **6 data models** in a single storage engine using Apache Arrow.

### The Six Data Models

#### 1. Relational (SQL Tables)

**Traditional SQL tables** with schemas, indexes, and ACID transactions.

**Storage**: Arrow RecordBatch (columnar)

```rust
pub struct RelationalEngine {
    arrow_store: Arc<ArrowStore>,
    datafusion: Arc<DataFusion>,  // SQL engine
}

// Example: Users table
// Schema: (id: Int64, name: Utf8, email: Utf8, created_at: Timestamp)

let schema = Schema::new(vec![
    Field::new("id", DataType::Int64, false),
    Field::new("name", DataType::Utf8, false),
    Field::new("email", DataType::Utf8, false),
    Field::new("created_at", DataType::Timestamp(TimeUnit::Millisecond, None), false),
]);

// Write as Arrow RecordBatch
let batch = RecordBatch::try_new(
    Arc::new(schema),
    vec![
        Arc::new(Int64Array::from(vec![1, 2, 3])),
        Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"])),
        Arc::new(StringArray::from(vec!["alice@example.com", "bob@example.com", "charlie@example.com"])),
        Arc::new(TimestampMillisecondArray::from(vec![1730000000000, 1730000001000, 1730000002000])),
    ],
)?;

relational.write_batch(batch).await?;

// Query with SQL
let result = relational.query("SELECT name, email FROM users WHERE id > 1").await?;
```

#### 2. Document (JSON/XML Hierarchies)

**Hierarchical documents** with nested structures.

**Storage**: Arrow Struct (nested columnar)

```rust
pub struct DocumentEngine {
    arrow_store: Arc<ArrowStore>,
}

// Example: Blog post document
// Schema: { id: Int64, title: Utf8, author: Struct(name, email), tags: List(Utf8), content: Utf8 }

let doc = json!({
    "id": 123,
    "title": "Pyralog Architecture",
    "author": {
        "name": "Alice",
        "email": "alice@pyralog.io"
    },
    "tags": ["database", "distributed", "rust"],
    "content": "..."
});

// Store as Arrow Struct
let batch = document.json_to_arrow(&doc).await?;
document.write_batch(batch).await?;

// Query with JSONPath
let result = document.query("$.author.name").await?;
```

#### 3. Property Graph (Cypher Queries)

**Nodes and edges** with properties for graph traversals.

**Storage**: Arrow Table with adjacency list

```rust
pub struct GraphEngine {
    arrow_store: Arc<ArrowStore>,
}

// Example: Social network graph
// Nodes: (id: Int64, label: Utf8, properties: Struct)
// Edges: (from: Int64, to: Int64, label: Utf8, properties: Struct)

// Create nodes
graph.create_node(Node { id: 1, label: "Person", properties: { name: "Alice" } }).await?;
graph.create_node(Node { id: 2, label: "Person", properties: { name: "Bob" } }).await?;

// Create edge
graph.create_edge(Edge { from: 1, to: 2, label: "FOLLOWS", properties: {} }).await?;

// Query with Cypher
let result = graph.query("MATCH (a:Person)-[:FOLLOWS]->(b:Person) RETURN a.name, b.name").await?;
```

#### 4. RDF Graph (SPARQL Queries)

**Semantic triples** (subject, predicate, object) for semantic web.

**Storage**: Arrow Triple Table

```rust
pub struct RDFEngine {
    arrow_store: Arc<ArrowStore>,
}

// Example: Knowledge graph
// Schema: (subject: Utf8, predicate: Utf8, object: Utf8)

// Add triples
rdf.add_triple("<pyralog>", "<type>", "<database>").await?;
rdf.add_triple("<pyralog>", "<written_in>", "<rust>").await?;
rdf.add_triple("<pyralog>", "<supports>", "<multi_model>").await?;

// Query with SPARQL
let result = rdf.query("SELECT ?o WHERE { <pyralog> <supports> ?o }").await?;
```

#### 5. Tensor (Multi-Dimensional Arrays)

**ML/AI tensors** with native operations.

**Storage**: Arrow FixedSizeList or File Reference (Safetensors, Zarr)

```rust
pub struct TensorEngine {
    arrow_store: Arc<ArrowStore>,
}

// Example: ML model embeddings
// Schema: (id: Int64, embedding: FixedSizeList(Float32, 768))

let embedding = vec![0.1, 0.2, ..., 0.768];  // 768-dimensional vector

// Store as Arrow FixedSizeList (for analytics)
let batch = tensor.vector_to_arrow(&embedding).await?;
tensor.write_batch(batch).await?;

// Or store as Safetensors file (for large models)
let model_path = PathBuf::from("/mnt/models/bert-base.safetensors");
tensor.store_file_reference(model_path).await?;

// Query with tensor operations
let result = tensor.cosine_similarity(embedding1, embedding2).await?;
```

#### 6. Key-Value (High-Speed Lookups)

**Simple key-value pairs** for fast dictionary storage.

**Storage**: Arrow Dictionary (columnar dictionary encoding)

```rust
pub struct KVEngine {
    arrow_store: Arc<ArrowStore>,
}

// Example: User sessions
// Schema: (key: Utf8, value: Binary)

kv.put("session:123", b"user_data...").await?;
let value = kv.get("session:123").await?;
```

### Unified Storage: Apache Arrow

**Columnar in-memory format** for zero-copy data interchange.

#### Why Arrow?

| Benefit | Description |
|---------|-------------|
| **Zero-copy** | No serialization between models |
| **Columnar** | SIMD vectorization (8-16× speedup) |
| **Cross-model joins** | 10-50× faster than ETL |
| **DataFusion/Polars** | Best-in-class SQL engine |
| **Industry standard** | Interop with Pandas, Spark, etc. |

#### Cross-Model Queries

**Example**: Join relational table with graph data

```sql
-- SQL query joining relational users with graph relationships
SELECT u.name, COUNT(e.to) AS follower_count
FROM users AS u
LEFT JOIN graph_edges AS e ON e.from = u.id
WHERE e.label = 'FOLLOWS'
GROUP BY u.name;
```

**Performance**: 10-50× faster than ETL (no data copying!)

**See also**: [MULTI_MODEL_DATABASE.md](MULTI_MODEL_DATABASE.md) for details, [ARROW.md](ARROW.md) for Arrow integration, blog post [07](blog/07-multi-model-database.md).

### Category Theory Foundation

**Schema as category, instance as functor** for proven correctness of transformations.

#### Core Concepts

1. **Schema**: Category with types as objects, transformations as morphisms
2. **Instance**: Functor mapping schema to data
3. **Query**: Natural transformation between functors
4. **Correctness**: Commutative diagrams ensure query correctness

**See also**: [FUNCTIONAL_RELATIONAL_ALGEBRA.md](FUNCTIONAL_RELATIONAL_ALGEBRA.md), blog post [18](blog/18-category-theory.md).

---

## Query & Programming Languages

Pyralog offers **4 query interfaces** with different theoretical rigor levels:

### Theoretical Rigor Spectrum

```
SQL (none) < PRQL (pragmatic) < GraphQL (API) < **Batuta (Category Theory)**
```

### 🎼 Batuta Language (Theoretically Founded)

**A full programming language** with Category Theory foundations and Functional Relational Algebra.

#### Core Principles

1. **Category Theory**: Functors, monads, natural transformations
2. **Functional Relational Algebra**: Proven query optimizations
3. **Sulise Foundation**: Complete language theory (grammar, type systems, semantics)
4. **Actor-First**: Queries execute as distributed actors
5. **Lisp Macros**: Full metaprogramming capabilities

#### Two Execution Modes

**Client-Side (Application-Embedded)**:
```rust
// Batuta runs in application process
let batuta = BatutaRuntime::new_client();
let result = batuta.eval("
  (from users
    (filter (> age 18))
    (select name email))
").await?;
```

**Server-Side (Database-Embedded)**:
```rust
// Batuta runs in Pyramid node
let batuta = BatutaRuntime::new_server(pyramid_node);
let result = batuta.eval("
  (from users
    (join orders (on (= users.id orders.user_id)))
    (aggregate (count *)))
").await?;
```

#### Category Theory Example

```clojure
;; Define a category for database schema
(defcategory UserSchema
  (objects User Order Product)
  (morphisms
    (has-orders User → [Order])
    (contains Order → [Product])))

;; Define a functor mapping schema to data
(deffunctor UserData [UserSchema → Set]
  (map User #{alice bob charlie})
  (map Order #{order1 order2})
  (map has-orders {alice [order1] bob [order2]}))

;; Query as natural transformation
(defnatural-transformation
  active-users
  [UserData → UserData]
  (from User u
    (where (> (count (has-orders u)) 0))
    (select u)))
```

**Benefits**:
- **Proven correctness**: Category Theory guarantees
- **Type-safe schema evolution**: Commutative diagrams
- **Automatic optimization**: Functional Relational Algebra

**See also**: [BATUTA.md](BATUTA.md), [FUNCTIONAL_RELATIONAL_ALGEBRA.md](FUNCTIONAL_RELATIONAL_ALGEBRA.md), blog posts [08](blog/08-batuta-language.md), [17](blog/17-batuta-modes.md), [18](blog/18-category-theory.md).

### PRQL (Pragmatic Query Language)

**Functional pipelines** that compile to SQL for readable queries.

```prql
from users
filter age > 18
join orders (users.id == orders.user_id)
aggregate (count *)
```

Compiles to:
```sql
SELECT COUNT(*)
FROM users
JOIN orders ON users.id = orders.user_id
WHERE users.age > 18;
```

**Benefits**:
- **10× more readable**: Pipelines > nested SQL
- **Compiles to SQL**: Zero runtime overhead
- **Type-safe**: Catches errors at compile time

**See also**: [PRQL.md](PRQL.md), blog post [16](blog/16-five-interfaces.md).

### GraphQL (Flexible API)

**Client-driven queries** for fetching exactly the data needed.

```graphql
query {
  users(age_gt: 18) {
    name
    email
    orders {
      id
      total
      products {
        name
        price
      }
    }
  }
}
```

**Benefits**:
- **Client-driven**: Request exactly what you need
- **Type-safe**: Strong type system
- **Real-time**: Subscriptions for live updates
- **Multi-model**: Query across relational, document, graph

**See also**: [GRAPHQL.md](GRAPHQL.md), blog post [16](blog/16-five-interfaces.md).

### JSON-RPC/WebSocket (Lightweight RPC)

**Low-latency, bidirectional RPC** for real-time applications.

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "query",
  "params": {
    "sql": "SELECT * FROM users WHERE age > 18",
    "format": "arrow"
  }
}

// Response (with Arrow IPC binary)
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "format": "arrow-ipc",
    "data": "<binary Arrow IPC stream>"
  }
}
```

**Benefits**:
- **Low-latency**: <5ms overhead
- **Bidirectional**: Server can push updates
- **Binary support**: Arrow IPC for zero-copy
- **Simpler than gRPC**: No protobuf compilation
- **Browser-native**: WebSocket in all browsers

**Why JSON-RPC/WS Replaces gRPC**:
- ✅ 30-50% faster (no HTTP/2 framing overhead)
- ✅ Simpler (no protobuf, no code generation)
- ✅ Browser-native (WebSocket everywhere)
- ✅ Better binary format (Arrow IPC vs protobuf)

**See also**: [JSONRPC_WEBSOCKET.md](JSONRPC_WEBSOCKET.md), blog post [16](blog/16-five-interfaces.md).

---

## Actor Model

Pyralog uses **actors** for distributed query execution with fault tolerance.

### Location-Transparent Actors

**Queries execute as distributed actors** that can run anywhere in the cluster.

```rust
pub struct QueryActor {
    query: Query,
    actor_system: Arc<ActorSystem>,
}

impl QueryActor {
    pub async fn execute(&self) -> Result<QueryResult> {
        // 1. Spawn child actors for each partition
        let partition_actors: Vec<_> = self.query.partitions.iter()
            .map(|&p| self.actor_system.spawn(PartitionQueryActor::new(p, self.query.clone())))
            .collect();
        
        // 2. Execute queries in parallel across partitions
        let partition_results = futures::future::join_all(
            partition_actors.iter().map(|a| a.send(Execute))
        ).await?;
        
        // 3. Aggregate results
        let result = self.aggregate(partition_results)?;
        
        Ok(result)
    }
}
```

**Benefits**:
- **Location transparency**: Actor can run on any node
- **Automatic parallelism**: Partitions processed concurrently
- **Fault tolerance**: Actors can restart on failure

### Supervision Trees

**Self-healing hierarchies** for fault tolerance ("let it crash" philosophy).

```rust
pub struct QuerySupervisor {
    children: Vec<ActorRef<QueryActor>>,
    strategy: SupervisionStrategy,
}

#[derive(Debug, Clone)]
pub enum SupervisionStrategy {
    OneForOne,  // Restart only failed child
    OneForAll,  // Restart all children if one fails
    RestForOne, // Restart failed child and all younger siblings
}

impl QuerySupervisor {
    pub async fn handle_failure(&self, failed_actor: ActorRef<QueryActor>, error: Error) {
        match self.strategy {
            SupervisionStrategy::OneForOne => {
                // Restart only the failed actor
                self.restart_actor(failed_actor).await;
            }
            SupervisionStrategy::OneForAll => {
                // Restart all child actors
                for child in &self.children {
                    self.restart_actor(child.clone()).await;
                }
            }
            SupervisionStrategy::RestForOne => {
                // Restart failed actor and all younger siblings
                let failed_idx = self.children.iter().position(|c| c == &failed_actor).unwrap();
                for child in &self.children[failed_idx..] {
                    self.restart_actor(child.clone()).await;
                }
            }
        }
    }
}
```

**Benefits**:
- **Self-healing**: Automatic recovery from failures
- **Fault isolation**: Failures don't propagate
- **Configurable**: Choose supervision strategy per use case

### Topology-Level Reactivity

**Flocks and deploy-* operators** for peer discovery and coordination.

```rust
// Flock: Auto-discover and coordinate with peers
let flock = Flock::new("query-workers");
flock.join("pyralog-cluster").await?;

// deploy-map: Distribute work across flock members
let results = flock.deploy_map(|node| {
    node.execute_query(query.clone())
}).await?;

// deploy-reduce: Aggregate results from all nodes
let final_result = flock.deploy_reduce(results, |a, b| {
    merge_query_results(a, b)
}).await?;
```

**Benefits**:
- **Auto-discovery**: Nodes find each other via mDNS/gossip
- **Dynamic topology**: Add/remove nodes without reconfiguration
- **Declarative coordination**: deploy-* operators abstract complexity

**See also**: [ACTOR_MODEL.md](ACTOR_MODEL.md), blog post [09](blog/09-actor-concurrency.md).

### Formal Semantics

1. **π-calculus**: Process communication and concurrency
2. **Session types**: Protocol safety and correctness
3. **Category theory**: Actor composition

**See also**: [FUNCTIONAL_RELATIONAL_ALGEBRA.md](FUNCTIONAL_RELATIONAL_ALGEBRA.md).

---

## Tensor Database

Pyralog provides **native tensor storage and operations** for ML/AI workloads.

### Storage Strategy

**Two-layer architecture**:

1. **Persistent Storage**: Safetensors files (100× faster than pickle)
2. **Runtime Exchange**: DLPack (zero-copy between frameworks)

```rust
pub struct TensorEngine {
    arrow_store: Arc<ArrowStore>,
    tensor_cache: Arc<TensorCache>,
}

impl TensorEngine {
    pub async fn store_model(&self, model_id: &str, tensors: &HashMap<String, Tensor>) -> Result<()> {
        // 1. Serialize to Safetensors format
        let safetensors_bytes = serialize_safetensors(tensors)?;
        
        // 2. Write to disk
        let path = format!("/mnt/models/{}.safetensors", model_id);
        fs::write(&path, safetensors_bytes).await?;
        
        // 3. Store file reference in LSM-Tree
        self.arrow_store.put(
            model_id.as_bytes(),
            StorageValue::FileReference {
                path: PathBuf::from(path),
                offset: 0,
                length: safetensors_bytes.len() as u64,
                format: ExternalFormat::Safetensors,
            }
        ).await?;
        
        Ok(())
    }
    
    pub async fn load_model(&self, model_id: &str) -> Result<HashMap<String, Tensor>> {
        // 1. Get file reference from LSM-Tree
        let file_ref = self.arrow_store.get(model_id.as_bytes()).await?;
        
        // 2. Memory-map Safetensors file (zero-copy!)
        let tensors = match file_ref {
            StorageValue::FileReference { path, .. } => {
                deserialize_safetensors(&path)?
            }
            _ => return Err(Error::InvalidStorageValue),
        };
        
        Ok(tensors)
    }
    
    pub fn to_dlpack(&self, tensor: &Tensor) -> DLPackTensor {
        // Zero-copy export to DLPack for PyTorch/TensorFlow/JAX
        DLPackTensor::from_tensor(tensor)
    }
}
```

### Use Cases

#### 1. Vector Embeddings

```rust
// Store embeddings for similarity search
let embedding = vec![0.1, 0.2, ..., 0.768];  // 768-dim BERT embedding

tensor_engine.store_embedding(
    "doc_123",
    embedding.clone(),
).await?;

// ANN search for similar embeddings
let results = tensor_engine.ann_search(
    embedding,
    top_k: 10,
    metric: CosineS imilarity,
).await?;
```

#### 2. ML Feature Store

```rust
// Store versioned features for ML training
let features = FeatureSet {
    user_id: 123,
    features: vec![
        ("age", 25.0),
        ("clicks", 150.0),
        ("purchases", 5.0),
    ],
    timestamp: SystemTime::now(),
    version: 1,
};

tensor_engine.store_features(features).await?;

// Load features for training
let training_data = tensor_engine.load_features(
    user_ids: vec![123, 456, 789],
    version: 1,
).await?;
```

#### 3. Model Registry

```rust
// Store trained model with metadata
let model = TrainedModel {
    id: "bert-base-v2",
    weights: model_tensors,
    metadata: ModelMetadata {
        framework: "PyTorch",
        accuracy: 0.95,
        training_date: "2025-11-03",
    },
};

tensor_engine.register_model(model).await?;

// Load model for inference
let model = tensor_engine.load_model("bert-base-v2").await?;
let predictions = model.predict(inputs).await?;
```

#### 4. Hugging Face Integration

```rust
// Download model from Hugging Face and store in Pyralog
let model = tensor_engine.download_from_huggingface(
    "bert-base-uncased",
).await?;

// Model is now stored as Safetensors file with file reference
// Access it via zero-copy memory-mapping
```

### Performance

| Operation | Traditional (pickle) | Pyralog (Safetensors + DLPack) |
|-----------|---------------------|-------------------------------|
| **Model save** | ~10 sec | ~100 ms (100× faster) |
| **Model load** | ~5 sec | ~50 ms (100× faster) |
| **Framework exchange** | Copy | Zero-copy (DLPack) |
| **Safety** | ⚠️ Arbitrary code execution | ✅ Memory-safe |

**See also**: [TENSOR_DATABASE.md](TENSOR_DATABASE.md), [DATA_FORMATS.md](DATA_FORMATS.md), blog post [19](blog/19-tensor-database.md).

---

## Cryptographic Verification

Pyralog provides **cryptographic guarantees** for data integrity and zero-trust architectures.

### Merkle Trees

**Hierarchical hash trees** for efficient verification.

```rust
pub struct MerkleTreeManager {
    trees: HashMap<PartitionId, MerkleTree>,
}

impl MerkleTreeManager {
    pub async fn append_leaf(&self, partition: PartitionId, offset: LogOffset, record: &Record) -> Result<()> {
        // 1. Hash the record using BLAKE3 (10× faster than SHA256)
        let leaf_hash = blake3::hash(&record.serialize()?);
        
        // 2. Get Merkle tree for partition
        let mut tree = self.trees.get_mut(&partition).unwrap();
        
        // 3. Append leaf and update tree
        tree.append(leaf_hash);
        
        // 4. Store updated root hash
        self.store_root_hash(partition, tree.root()).await?;
        
        Ok(())
    }
    
    pub fn generate_proof(&self, partition: PartitionId, offset: LogOffset) -> Result<MerkleProof> {
        let tree = self.trees.get(&partition).unwrap();
        let proof = tree.generate_proof(offset)?;
        Ok(proof)
    }
    
    pub fn verify_proof(&self, root: Hash, proof: &MerkleProof, leaf: Hash) -> bool {
        proof.verify(root, leaf)
    }
}
```

**Benefits**:
- **Efficient**: O(log N) proof size
- **Fast**: BLAKE3 is 10× faster than SHA256
- **Tamper-evident**: Any modification changes root hash

### Zero-Trust Architecture

**Client-side verification** without trusting the server.

```rust
// Client verifies data integrity without trusting server
let record = client.read(partition, offset).await?;
let proof = client.request_merkle_proof(partition, offset).await?;
let root = client.get_root_hash(partition).await?;

// Verify locally
let leaf_hash = blake3::hash(&record.serialize()?);
if !proof.verify(root, leaf_hash) {
    return Err(Error::TamperedData);
}
```

### Notarization API

**Cryptographic timestamps** for legal/copyright protection.

```rust
pub struct NotarizationService {
    merkle_trees: Arc<MerkleTreeManager>,
}

impl NotarizationService {
    pub async fn notarize(&self, data: &[u8]) -> Result<NotarizationReceipt> {
        // 1. Hash the data
        let data_hash = blake3::hash(data);
        
        // 2. Append to Merkle tree
        let partition = self.select_notarization_partition();
        let offset = self.merkle_trees.append_leaf(partition, data_hash).await?;
        
        // 3. Generate receipt
        let receipt = NotarizationReceipt {
            data_hash,
            partition,
            offset,
            timestamp: SystemTime::now(),
            merkle_root: self.merkle_trees.get_root(partition)?,
        };
        
        Ok(receipt)
    }
    
    pub fn verify_receipt(&self, data: &[u8], receipt: &NotarizationReceipt) -> Result<bool> {
        // Verify data hasn't been tampered with
        let data_hash = blake3::hash(data);
        Ok(data_hash == receipt.data_hash)
    }
}
```

### Auditor Mode

**Independent verification** for regulatory compliance.

```rust
// Auditor runs independent read-only node
let auditor = AuditorNode::new();

// Continuously verify data integrity
loop {
    for partition in partitions {
        let root = auditor.read_root_hash(partition).await?;
        let verified = auditor.verify_partition(partition, root).await?;
        
        if !verified {
            auditor.report_tampering(partition).await?;
        }
    }
    
    tokio::time::sleep(Duration::from_secs(60)).await;
}
```

**Use cases**:
- SEC, HIPAA, SOC2 compliance
- Tamper detection
- Forensic auditing

**See also**: [CRYPTOGRAPHIC_VERIFICATION.md](CRYPTOGRAPHIC_VERIFICATION.md), blog post [06](blog/06-cryptographic-verification.md).

---

## Multi-Layer Deduplication

Pyralog applies **5 deduplication strategies** at different layers.

### The Five Layers

#### 1. Storage-Level Deduplication (LSM Compaction)

```rust
pub enum DeduplicationMode {
    LastWriteWins,     // Keep most recent value
    Tombstone,         // Keep delete markers
    MVCC,              // Keep all versions
}

// Example: Last-Write-Wins compaction
impl LSMTree {
    async fn compact_lww(&self) -> Result<()> {
        let mut seen = HashMap::new();
        let mut output = Vec::new();
        
        // Scan SSTables in reverse order (most recent first)
        for sstable in self.sstables.read().iter().rev() {
            for (key, value) in sstable.iter() {
                if !seen.contains_key(&key) {
                    seen.insert(key.clone(), true);
                    output.push((key, value));
                }
            }
        }
        
        // Write deduplicated SSTable
        self.write_sstable(output).await?;
        Ok(())
    }
}
```

#### 2. PPHM-Level Deduplication (Index Merging)

**6 deduplication strategies** when merging perfect hash maps:

1. **First-Wins**: Keep first occurrence
2. **Last-Wins**: Keep last occurrence
3. **Min-Value**: Keep minimum value
4. **Max-Value**: Keep maximum value
5. **Concatenate**: Merge values
6. **Custom**: User-defined merge function

**See also**: [PPHM.md](PPHM.md), blog post [13](blog/13-perfect-hash-maps.md).

#### 3. Exactly-Once Semantics

```rust
pub struct ExactlyOnceDeduplicator {
    sessions: HashMap<SessionId, SessionState>,
}

impl ExactlyOnceDeduplicator {
    pub async fn deduplicate_write(&self, session_id: SessionId, sequence: u64, record: Record) -> Result<bool> {
        let session = self.sessions.get(&session_id).unwrap();
        
        // Check if already written
        if session.written_sequences.contains(&sequence) {
            return Ok(false);  // Duplicate, skip
        }
        
        // Write and mark as written
        self.storage.write(record).await?;
        session.written_sequences.insert(sequence);
        
        Ok(true)  // New write
    }
}
```

#### 4. Content-Addressable Storage

```rust
pub struct ContentAddressableStore {
    chunks: HashMap<Blake3Hash, Vec<u8>>,
}

impl ContentAddressableStore {
    pub async fn store(&self, data: &[u8]) -> Result<Blake3Hash> {
        // 1. Hash content
        let hash = blake3::hash(data);
        
        // 2. Check if already stored
        if self.chunks.contains_key(&hash) {
            return Ok(hash);  // Deduplicated!
        }
        
        // 3. Store new chunk
        self.chunks.insert(hash, data.to_vec());
        Ok(hash)
    }
}
```

#### 5. Application-Level Deduplication

```rust
// Semantic deduplication: Detect similar documents
let doc1_embedding = tensor_engine.embed("Document 1 content").await?;
let doc2_embedding = tensor_engine.embed("Document 2 content").await?;

let similarity = cosine_similarity(&doc1_embedding, &doc2_embedding);
if similarity > 0.95 {
    // Documents are duplicates (95% similar)
    deduplicate_documents(doc1, doc2).await?;
}
```

**See also**: [DEDUPLICATION.md](DEDUPLICATION.md), diagram [deduplication-layers.mmd](diagrams/deduplication-layers.mmd), blog post [14](blog/14-deduplication.md).

---

## Decentralized Network

Pyralog supports **two deployment models**: Cluster (single datacenter) and Network (multiple clusters).

### Pyralog Cluster (Single Datacenter)

**Strong consistency** with Raft consensus:

- **Use case**: Traditional distributed database
- **Consistency**: Strong (Raft per partition)
- **Latency**: Sub-millisecond
- **Fault tolerance**: Crash fault tolerant (CFT)
- **Scale**: Single region/datacenter

### Pyralog Network (Multiple Clusters)

**Decentralized Autonomous Database** with Byzantine fault tolerance:

- **Use case**: Global-scale, trustless applications
- **Consistency**: Eventual (configurable)
- **Latency**: Milliseconds to seconds (cross-region)
- **Fault tolerance**: Byzantine fault tolerant (BFT)
- **Scale**: Multi-region/continent

### Consensus Mechanisms

#### 1. Raft (Default for Clusters)

**Crash fault tolerant** consensus for trusted environments:

```rust
pub struct RaftConsensus {
    raft_node: Arc<RaftNode>,
}

impl RaftConsensus {
    pub async fn propose(&self, command: Command) -> Result<()> {
        // Propose to Raft
        self.raft_node.propose(command).await?;
        Ok(())
    }
}
```

**Properties**:
- **Fast**: < 10ms consensus
- **Simple**: Easier to implement than Paxos
- **Trusted**: Assumes non-Byzantine faults

#### 2. Proof of Work (PoW)

**Useful computation** for anti-spam and rate limiting:

```rust
pub struct ProofOfWork {
    difficulty: u32,
}

impl ProofOfWork {
    pub fn compute(&self, data: &[u8]) -> Result<u64> {
        // Find nonce such that hash(data || nonce) has `difficulty` leading zeros
        let mut nonce = 0u64;
        loop {
            let hash = blake3::hash(&[data, &nonce.to_le_bytes()].concat());
            if Self::check_difficulty(&hash, self.difficulty) {
                return Ok(nonce);
            }
            nonce += 1;
        }
    }
}
```

**Use cases** (not just mining!):
- **Anti-spam**: Require PoW for writes
- **Rate limiting**: Natural throttle mechanism
- **Sybil resistance**: Make fake identities expensive
- **Priority queues**: Higher PoW = higher priority
- **Time-lock puzzles**: Delay access until computation complete

**See also**: Blog post [23](blog/23-pow-useful.md).

#### 3. Proof of Stake (PoS)

**Energy-efficient staking** for economic incentives:

```rust
pub struct ProofOfStake {
    stakes: HashMap<NodeId, u64>,
    total_stake: u64,
}

impl ProofOfStake {
    pub fn select_proposer(&self, round: u64) -> NodeId {
        // Weighted random selection based on stake
        let mut rng = ChaCha20Rng::seed_from_u64(round);
        let target = rng.gen_range(0..self.total_stake);
        
        let mut cumulative = 0;
        for (&node, &stake) in &self.stakes {
            cumulative += stake;
            if cumulative > target {
                return node;
            }
        }
        
        unreachable!()
    }
}
```

**Properties**:
- **Energy-efficient**: No wasteful computation
- **Fast finality**: Seconds instead of minutes
- **Economic security**: Slashing for misbehavior

#### 4. zk-SNARKs (Privacy-Preserving)

**Zero-knowledge proofs** for private transactions:

```rust
pub struct ZKSnark {
    proving_key: ProvingKey,
    verifying_key: VerifyingKey,
}

impl ZKSnark {
    pub fn prove(&self, witness: &Witness) -> Result<Proof> {
        // Generate proof (slow: seconds)
        let proof = groth16::create_proof(&self.proving_key, witness)?;
        Ok(proof)
    }
    
    pub fn verify(&self, proof: &Proof, public_inputs: &[Fr]) -> bool {
        // Verify proof (fast: 1-5ms)
        groth16::verify_proof(&self.verifying_key, proof, public_inputs).is_ok()
    }
}
```

**Properties**:
- **Small proofs**: 200-500 bytes
- **Fast verification**: 1-5ms
- **Slow generation**: Seconds to minutes
- **Trusted setup**: Requires ceremony

**Use cases**:
- Private transactions (amounts hidden)
- Verifiable computation (prove correctness without revealing inputs)
- Batch verification (aggregate multiple proofs)

#### 5. zk-STARKs (No Trusted Setup)

**Transparent zero-knowledge proofs**:

```rust
pub struct ZKStark;

impl ZKStark {
    pub fn prove(&self, witness: &Witness) -> Result<Proof> {
        // Generate STARK proof (faster for large computations)
        let proof = stark::prove(witness)?;
        Ok(proof)
    }
    
    pub fn verify(&self, proof: &Proof) -> bool {
        // Verify (slower than SNARKs: 10-50ms)
        stark::verify(proof).is_ok()
    }
}
```

**Properties**:
- **No trusted setup**: Transparent
- **Post-quantum secure**: Resistant to quantum attacks
- **Larger proofs**: 100-200 KB (vs 200-500 bytes for SNARKs)
- **Slower verification**: 10-50ms (vs 1-5ms for SNARKs)

**See also**: [DECENTRALIZED.md](DECENTRALIZED.md), [DADBS.md](DADBS.md), blog posts [21](blog/21-decentralized.md), [22](blog/22-zk-proofs.md).

---

## Network Protocol

Pyralog uses **JSON-RPC/WebSocket** as the primary RPC protocol with **Arrow Flight** for zero-copy data transport.

### JSON-RPC/WebSocket (Primary RPC)

**Low-latency, bidirectional RPC** for real-time applications:

```rust
pub struct JsonRpcServer {
    pyramid_node: Arc<PyramidNode>,
}

impl JsonRpcServer {
    pub async fn handle_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        match request.method.as_str() {
            "query" => {
                // Execute SQL/PRQL/GraphQL query
                let result = self.pyramid_node.query(&request.params).await?;
                
                // Return as Arrow IPC (zero-copy)
                Ok(JsonRpcResponse {
                    id: request.id,
                    result: ArrowIpcStream::new(result),
                })
            }
            "write" => {
                // Write records
                let offset = self.pyramid_node.write(&request.params).await?;
                Ok(JsonRpcResponse {
                    id: request.id,
                    result: json!({ "offset": offset }),
                })
            }
            _ => Err(Error::MethodNotFound),
        }
    }
}
```

**Benefits**:
- **Low-latency**: <5ms overhead
- **Bidirectional**: Server push via WebSocket
- **Binary support**: Arrow IPC streams
- **Simpler than gRPC**: No protobuf, no code generation
- **Browser-native**: WebSocket everywhere

**See also**: [JSONRPC_WEBSOCKET.md](JSONRPC_WEBSOCKET.md).

### Arrow Flight (Zero-Copy Data Transport)

**High-performance data streams**:

```rust
pub struct ArrowFlightServer {
    arrow_store: Arc<ArrowStore>,
}

impl ArrowFlightServer {
    pub async fn do_get(&self, ticket: FlightTicket) -> Result<FlightData> {
        // 1. Read Arrow RecordBatch from storage
        let batch = self.arrow_store.read_batch(&ticket.key).await?;
        
        // 2. Convert to FlightData (zero-copy!)
        let flight_data = FlightData::from_batch(&batch)?;
        
        Ok(flight_data)
    }
}
```

**Benefits**:
- **3× faster**: Than gRPC/Protobuf
- **Zero-copy**: Direct Arrow IPC
- **Columnar**: SIMD-friendly data layout
- **Streaming**: Large datasets

### WireGuard + Rosenpass (Quantum-Resistant)

**Secure, low-latency networking**:

```rust
pub struct WireGuardTunnel {
    interface: WgInterface,
    rosenpass: RosenpassHandshake,
}

impl WireGuardTunnel {
    pub async fn establish_tunnel(&self, peer: SocketAddr) -> Result<()> {
        // 1. Perform Rosenpass handshake (post-quantum key exchange)
        let shared_secret = self.rosenpass.handshake(peer).await?;
        
        // 2. Configure WireGuard tunnel
        self.interface.set_peer(peer, &shared_secret)?;
        
        Ok(())
    }
}
```

**Benefits**:
- **10× less complexity**: Than TLS handshake
- **Post-quantum**: Kyber1024 + Dilithium
- **DPI resistance**: Traffic obfuscation
- **Cryptokey routing**: No IP-based trust

**See also**: [WIREGUARD_PROTOCOL.md](WIREGUARD_PROTOCOL.md), [JSONRPC_WEBSOCKET.md](JSONRPC_WEBSOCKET.md).

### Smart Client Architecture

**Direct connections** to partition leaders:

```rust
pub struct SmartClient {
    metadata_cache: Arc<RwLock<ClusterMetadata>>,
}

impl SmartClient {
    pub async fn write(&self, key: &[u8], value: &[u8]) -> Result<LogOffset> {
        // 1. Determine partition from key
        let partition = self.hash_partition(key);
        
        // 2. Get leader from cached metadata
        let leader = self.metadata_cache.read().get_leader(partition)?;
        
        // 3. Connect directly to leader (no proxy!)
        let offset = self.send_to_node(leader, key, value).await?;
        
        Ok(offset)
    }
    
    async fn refresh_metadata(&self) -> Result<()> {
        // Refresh metadata from any node (infrequent)
        let metadata = self.fetch_metadata().await?;
        *self.metadata_cache.write() = metadata;
        Ok(())
    }
}
```

**Benefits**:
- **1 hop**: Client → Leader (vs 2 hops with proxy)
- **Client-side load balancing**: No central bottleneck
- **Metadata caching**: ~5 min TTL, low overhead

**See also**: [DATA_PATH.md](DATA_PATH.md).

---

## Replication System

Pyralog supports **two CopySet strategies**: Per-Partition (simple) and Per-Record (maximum distribution).

### CopySet Selection Strategies

#### Strategy 1: Per-Partition CopySet (Kafka-Style)

**Fixed replicas per partition**:

```rust
pub struct PartitionCopySet {
    partition_id: PartitionId,
    nodes: Vec<NodeId>,  // Fixed: [N1, N2, N3]
}
```

**Benefits**:
- ✅ Simpler to implement
- ✅ Faster lookups (cached)
- ✅ Good for small clusters

**Use case**: Clusters < 10 nodes

#### Strategy 2: Per-Record CopySet (LogDevice-Style)

**Dynamic replicas per record**:

```rust
pub struct RecordCopySetSelector {
    nodes: Vec<NodeId>,
    replication_factor: usize,
    seed: u64,
}

impl RecordCopySetSelector {
    pub fn select(&self, lsn: u64) -> Vec<NodeId> {
        // Deterministic selection based on LSN
        let mut hasher = DefaultHasher::new();
        lsn.hash(&mut hasher);
        self.seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Select RF unique nodes
        let mut selected = Vec::new();
        let mut offset = hash as usize;
        
        while selected.len() < self.replication_factor {
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

**Key Innovation: Leader as Coordinator**

Leader doesn't need to store data:

```
Per-Record CopySet (Coordinator Mode):
  Client → Leader → Calculate CopySet [N3, N7, N9]
                  → Send directly to storage nodes
  
  Leader: Metadata only (~1 MB)
  Storage nodes: Actual data
  
  Result: Leader can handle 20×-50× more partitions! 🚀
```

**Benefits**:
- ✅ Maximum load distribution
- ✅ Leader disk-free (99%+ less I/O)
- ✅ 20×-50× more partitions per leader
- ✅ 90%+ cluster utilization

**Use case**: Clusters > 50 nodes

### Flexible Quorums

**Configure consistency vs availability**:

```rust
pub struct QuorumConfig {
    pub replication_factor: usize,  // R
    pub write_quorum: usize,         // W
    pub read_quorum: usize,          // Rd
}

// Constraint: W + Rd > R (ensures overlap)
```

**Examples**:

| Config | R | W | Rd | Use Case |
|--------|---|---|-----|----------|
| **Strong** | 3 | 3 | 3 | Maximum durability |
| **Balanced** | 3 | 2 | 2 | Standard config |
| **Write-heavy** | 3 | 1 | 3 | Low write latency |
| **Read-heavy** | 3 | 3 | 1 | Low read latency |

### ISR (In-Sync Replicas)

**Track which replicas are up-to-date**:

```rust
pub struct ISRTracker {
    isr: HashMap<PartitionId, Vec<NodeId>>,
    max_lag: Duration,
}

impl ISRTracker {
    pub fn update_isr(&mut self, partition: PartitionId, leader_offset: LogOffset, follower_offsets: &HashMap<NodeId, LogOffset>) {
        let mut in_sync = vec![];
        
        for (&node, &offset) in follower_offsets {
            let lag = leader_offset.as_u64() - offset.as_u64();
            if lag < 1000 {  // < 1000 records lag
                in_sync.push(node);
            }
        }
        
        self.isr.insert(partition, in_sync);
    }
}
```

**See also**: [ARCHITECTURE.md sections in original](#replication-system).

---

## Performance Optimizations

Pyralog applies **multiple optimizations** for sub-millisecond latencies.

### 1. Zero-Copy Data Flow

**Four layers of zero-copy**:

1. **Memory-mapped files**: 30-50% faster reads
2. **Arrow IPC**: Zero-copy serialization
3. **File references**: No blob duplication
4. **DMA transfers**: Direct memory access

```rust
// Zero-copy read from disk
let mmap = unsafe { Mmap::map(&file)? };
let data = &mmap[offset..offset+length];  // No copy!

// Zero-copy Arrow serialization
let batch = RecordBatch::from(data);  // No copy!

// Zero-copy network send
socket.send_vectored(&[batch.as_bytes()])?;  // No copy!
```

**See also**: Blog post [11](blog/11-zero-copy-data-flow.md).

### 2. Batch Processing

**Amortize overhead**:

```rust
// Batch multiple records
let batch = RecordBatch::new(vec![rec1, rec2, rec3]);

// Single write operation
storage.append_batch(batch).await?;
```

### 3. Write Caching

**In-memory buffering**:

```rust
pub struct WriteCache {
    buffer: VecDeque<Record>,
    max_size: usize,        // 16 MB
    max_time: Duration,     // 10 ms
}

impl WriteCache {
    pub async fn push(&mut self, record: Record) -> Result<()> {
        self.buffer.push_back(record);
        
        if self.should_flush() {
            self.drain_and_write().await?;
        }
        
        Ok(())
    }
    
    fn should_flush(&self) -> bool {
        self.buffer.len() * 1024 > self.max_size  // Size
            || self.last_flush.elapsed() > self.max_time  // Time
    }
}
```

### 4. Async I/O

**Concurrent operations**:

```rust
// Execute queries in parallel
let (r1, r2, r3) = tokio::join!(
    query1.execute(),
    query2.execute(),
    query3.execute(),
);
```

**See also**: [PERFORMANCE.md](PERFORMANCE.md).

---

## Scalability

Pyralog achieves **horizontal scalability** through partitioning, CopySet replication, and distributed leadership.

### Distributed Leadership

**Partitions spread across nodes**:

```
16 Partitions, 4 Nodes:
  Node 1 leads: [P0, P4, P8, P12]
  Node 2 leads: [P1, P5, P9, P13]
  Node 3 leads: [P2, P6, P10, P14]
  Node 4 leads: [P3, P7, P11, P15]

Write throughput:
  Single partition: 100K/sec
  16 partitions: 1.6M/sec (16× scaling)
```

### CopySet Distribution

**Replication load spread evenly**:

```
Without CopySet (bottleneck):
  All partitions: [N1, N2, N3]
  ❌ Nodes 1-3: Overloaded
  ❌ Nodes 4-6: Underutilized

With CopySet (balanced):
  P0: [N1, N2, N4]
  P1: [N2, N3, N5]
  P2: [N3, N4, N6]
  ✅ All nodes: Evenly loaded
```

### Read Scaling

**Reads from any replica**:

```
Write throughput: 10M/sec
Read throughput: 30M/sec (with RF=3)

Scaling: 3× read capacity per partition
```

### Dynamic Partitions

**Auto-split/merge for load balancing**:

```toml
[partition_policy]
mode = "dynamic"
max_size = 10_000_000_000  # 10 GB
max_write_rate = 100_000.0  # 100K/sec

# Hot partition auto-splits
# Cold partitions auto-merge
```

**See also**: [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md), [SCALABILITY section in original](#scalability).

### Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| **Write throughput** | 10M+/sec | 10 nodes, 100 partitions |
| **Read throughput** | 30M+/sec | With RF=3 |
| **Write latency (p99)** | < 1ms | With cache |
| **Read latency (p99)** | < 0.5ms | With mmap |
| **Scalability** | Linear | Add nodes/partitions |

---

## Monitoring & Observability

Pyralog provides **comprehensive metrics and tracing**.

### Key Metrics

```rust
pub struct PyralogMetrics {
    // Throughput
    write_rate: Counter,
    read_rate: Counter,
    
    // Latency
    write_latency: Histogram,
    read_latency: Histogram,
    
    // Replication
    replication_lag: Gauge,
    isr_count: Gauge,
    
    // Storage
    disk_usage: Gauge,
    segment_count: Gauge,
}
```

**Prometheus integration**:

```rust
// Expose metrics endpoint
let metrics = prometheus::default_registry();
let exporter = PrometheusExporter::new(metrics);
exporter.listen("0.0.0.0:9090").await?;
```

### Distributed Tracing

**OpenTelemetry integration**:

```rust
#[tracing::instrument]
pub async fn write(&self, record: Record) -> Result<LogOffset> {
    // Automatic trace context propagation
    let span = tracing::info_span!("write", record.key = ?record.key);
    let _guard = span.enter();
    
    // ... write logic ...
    
    Ok(offset)
}
```

### Grafana Dashboards

**Pre-built dashboards** for:
- Write/read throughput
- Latency percentiles (p50, p99, p999)
- Replication lag
- Disk usage
- Partition health

**See also**: [OPERATIONS.md](OPERATIONS.md), blog post [24](blog/24-operations.md).

---

## Failure Scenarios

Pyralog handles **multiple failure modes** with automatic recovery.

### Node Failure

```
Per-Partition Raft Election:
  1. Followers detect missing heartbeats
  2. Election timeout triggers (300ms)
  3. Candidate wins election (majority votes)
  4. New leader activates epoch
  5. System continues operating

Recovery time: < 10ms per partition
```

### Network Partition

```
Split-brain prevention:
  [N1] | [N2, N3, N4, N5]
  
  N1: Cannot form quorum (1 < 3) → Blocks writes
  N2-N5: Can form quorum (4 > 3) → Continues operation
  
  Result: Availability on majority side
```

### Disk Failure

```
Recovery process:
  1. Detect disk failure (I/O errors)
  2. Mark node as degraded
  3. Redirect reads to replicas
  4. Background recovery from object storage
  5. Rebuild local copy

Recovery time: Minutes to hours (data size dependent)
```

### Data Corruption

```
Detection & recovery:
  1. CRC checksum validation
  2. Merkle tree verification
  3. Request from healthy replica
  4. Automatic rebuild

Prevention: BLAKE3 checksums, Merkle trees
```

**See also**: [OPERATIONS.md](OPERATIONS.md), blog post [24](blog/24-operations.md).

---

## Architectural Philosophy

Pyralog's architecture embodies **four core principles**.

### 1. Optimize the Hot Path

**Write path optimizations**:
- Epochs avoid Raft (100× throughput gain)
- Cache avoids fsync (98% latency reduction)
- Smart clients avoid proxy (50% latency reduction)

**Read path optimizations**:
- Memory-mapped I/O for zero-copy
- ISR tracking for flexibility
- Metadata caching for direct routing

### 2. Eliminate Bottlenecks at Every Level

| Bottleneck | Solution |
|------------|----------|
| **Global consensus** | Dual Raft (separate domains) |
| **Single leader** | Distributed leadership (partitioning) |
| **Follower overload** | CopySet (distributed replication) |
| **Proxy overhead** | Smart clients (direct routing) |
| **Consensus per record** | Epochs (batch consensus) |
| **Sequential failover** | Per-partition Raft (parallel) |

### 3. Make Trade-offs Configurable

**CAP spectrum**:
- Strong consistency: W=3, R=3
- High availability: W=1, R=3
- Balanced: W=2, R=2

**Read policy**:
- Leader only (strong consistency)
- Any replica (low latency)
- Quorum (balanced)
- Nearest (geo-distributed)

**Quorum sizes**:
- Balance durability vs latency
- Tune per use case

### 4. Horizontal Scalability

**Linear scaling**:
- Add nodes → Add capacity
- Add partitions → Add throughput
- No fundamental limitations

**Dynamic rebalancing**:
- Auto-split hot partitions
- Auto-merge cold partitions
- Continuous load balancing

## Key Innovations Summary

### Novel (Original to Pyralog) ⭐

1. **🗿 Obelisk Sequencer** - File size as persistent atomic counter
2. **☀️ Pharaoh Network** - Two-tier architecture (coordination vs storage)
3. **🪲 Scarab IDs** - Crash-safe globally unique IDs
4. **🎼 Batuta Language** - Category Theory + Functional Relational Algebra

### Synthesized (Best of Breed)

6. **Dual Raft Clusters** - Parallel failover (from TiKV)
7. **CopySet Replication** - Maximum utilization (from LogDevice)
8. **Smart Client Pattern** - Direct routing (from Kafka)
9. **Write Caching** - Sub-ms latencies (from Redpanda)
10. **Multi-Model Database** - 6 data models (from ArangoDB + theory)

## Learning from the Best

Pyralog synthesizes innovations from:
- **LogDevice** (Facebook): Epochs, CopySet, flexible quorums
- **Kafka** (LinkedIn): Smart clients, partitioning, ISR
- **Redpanda** (Vectorized): Write caching, zero-copy I/O
- **TiKV** (PingCAP): Multi-Raft architecture
- **Raft** (Stanford): Proven consensus algorithm

**Plus our own innovations**: Obelisk, Pharaoh, Scarab, Batuta

## The Big Picture

```
┌─────────────────────────────────────────────────────┐
│   Why Pyralog's Architecture Succeeds              │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Traditional Distributed Log:                       │
│    ❌ Leader bottleneck                             │
│    ❌ Consensus per record                          │
│    ❌ Proxy overhead                                │
│    ❌ Follower bottleneck                           │
│    ❌ Fixed consistency model                       │
│                                                     │
│  Pyralog's Solution:                                │
│    ✅ Distributed leadership (partitioning)         │
│    ✅ Consensus per epoch (100× faster)             │
│    ✅ Smart clients (direct routing)                │
│    ✅ Distributed replication (CopySet)             │
│    ✅ Flexible quorums (configurable)               │
│                                                     │
│  Result: 10M+ writes/sec, sub-ms latency,          │
│          horizontal scaling, no bottlenecks         │
│                                                     │
└─────────────────────────────────────────────────────┘
```

## Welcome to Pyralog 🔺

**Built to last millennia. Built for the next generation of distributed systems.**

This architecture combines:
- **Novel primitives** (Obelisk, Pharaoh, Scarab)
- **Proven techniques** (Raft, CopySet, LSM-Tree, Arrow)
- **Theoretical rigor** (Category Theory, Functional Relational Algebra)
- **Practical performance** (10M+ writes/sec, sub-ms latencies)

**The modular design allows for easy extension while maintaining strong guarantees about data durability and consistency. Whether you need strong consistency for financial transactions or high availability for analytics, Pyralog's architecture can be configured to meet your requirements.**

---

*Pyralog Architecture - Complete*  
*Last Updated: 2025-11-03*  
*Version: 2.0 (Complete Rewrite)*
