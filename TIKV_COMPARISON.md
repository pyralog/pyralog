# Pyralog vs TiKV: Architectural Comparison

A detailed comparison between Pyralog (distributed log) and TiKV (distributed key-value store).

## Table of Contents

1. [Overview](#overview)
2. [Architecture Similarities](#architecture-similarities)
3. [Key Differences](#key-differences)
4. [Multi-Raft Comparison](#multi-raft-comparison)
5. [Use Cases](#use-cases)
6. [Performance Characteristics](#performance-characteristics)
7. [Design Philosophy](#design-philosophy)
8. [When to Use Which](#when-to-use-which)

---

## Overview

### Pyralog

**Type**: Distributed log / Streaming platform  
**Language**: Rust  
**Inspired by**: LogDevice, Kafka, Redpanda  
**Primary Use**: Event streaming, message queues, change data capture  

### TiKV

**Type**: Distributed transactional key-value store  
**Language**: Rust  
**Part of**: TiDB (distributed SQL database)  
**Primary Use**: Database storage engine, distributed transactions  

---

## Architecture Similarities

Both systems share fundamental distributed systems patterns:

### 1. Multi-Raft Architecture ✅

**Both use dual Raft clusters!**

```
┌─────────────────────────────────────────────────────────┐
│   Pyralog Multi-Raft                                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Global Raft: [N1, N2, N3, N4, N5]                     │
│    → Cluster membership, partition metadata            │
│                                                         │
│  Per-Partition Raft:                                    │
│    Partition 0: [N1, N2, N3]                           │
│    Partition 1: [N2, N3, N4]                           │
│    → Epoch changes, partition leadership               │
│                                                         │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│   TiKV Multi-Raft                                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  PD (Placement Driver): Centralized metadata           │
│    → Uses Raft for its own HA                          │
│    → Manages cluster metadata, region assignments      │
│                                                         │
│  Per-Region Raft:                                       │
│    Region 0: [N1, N2, N3]                              │
│    Region 1: [N2, N3, N4]                              │
│    → Data replication, leadership for key ranges       │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Key Insight**: Both avoid global Raft bottleneck by using per-shard Raft!

### 2. Sharding Strategy

**Pyralog: Partitions**
```rust
// Partition by key hash
partition_id = hash(record.key) % partition_count

// Or by explicit partition
client.produce_to_partition(log_id, partition_id, record)
```

**TiKV: Regions (Dynamic)**
```rust
// Region = contiguous key range
Region 1: [key_a, key_m)
Region 2: [key_m, key_z)

// Regions split automatically when they grow too large
if region.size > threshold {
    split_region(region);
}
```

**Similarity**: Both distribute data across independent shards with separate Raft groups.

**Difference**: TiKV's regions split/merge dynamically; Pyralog's partitions are static (pre-allocated).

### 3. Rust Implementation

Both are implemented in Rust for:
- ✅ Memory safety without GC pauses
- ✅ Zero-cost abstractions
- ✅ Fearless concurrency
- ✅ High performance

### 4. Flexible Replication

**Pyralog:**
```toml
[replication]
replication_factor = 3
write_quorum = 2
read_quorum = 2
```

**TiKV:**
```
Region replicas: 3 (default)
Raft quorum: Majority (2/3)
```

Both use quorum-based replication for fault tolerance.

---

## Key Differences

### 1. Data Model

```
┌─────────────────────────────────────────────────────────┐
│   Pyralog: Append-Only Log                                 │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Operations:                                            │
│    ✓ Append (write to end)                             │
│    ✓ Read by offset/time                               │
│    ✓ Scan (sequential reads)                           │
│    ✗ Update (not supported)                            │
│    ✗ Delete (only via compaction)                      │
│                                                         │
│  Storage:                                               │
│    Offset 0: Record A                                  │
│    Offset 1: Record B                                  │
│    Offset 2: Record C                                  │
│    → Immutable, sequential                             │
│                                                         │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│   TiKV: Key-Value Store (LSM Tree)                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Operations:                                            │
│    ✓ Get (read by key)                                 │
│    ✓ Put (insert/update)                               │
│    ✓ Delete                                            │
│    ✓ Scan (range queries)                              │
│    ✓ Transactions (ACID)                               │
│                                                         │
│  Storage (RocksDB):                                     │
│    key_a → value_1                                     │
│    key_b → value_2                                     │
│    key_c → value_3                                     │
│    → Mutable, random access                            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 2. Transaction Support

**Pyralog:**
- ❌ No built-in ACID transactions (by design)
- ✅ Atomic appends within a partition
- ✅ Exactly-once semantics (idempotent producers)
- ✅ Cross-partition atomicity (future: 2PC for multi-partition writes)

**TiKV:**
- ✅ Full ACID transactions (Percolator model)
- ✅ Distributed transactions across regions
- ✅ Snapshot isolation
- ✅ Optimistic/Pessimistic locking

**Use case difference:**
- Pyralog: Event streaming, where order matters more than updates
- TiKV: Database storage, where updates and transactions are essential

### 3. Consistency Model

**Pyralog:**
```
Per-Partition Sequential Consistency:
  - Records within a partition are totally ordered
  - Across partitions: no ordering guarantees
  - Reads can be stale (configurable with quorums)

Epoch-based consistency:
  - Epoch = generation of partition leader
  - All records in epoch are ordered
  - Epoch change = failover boundary
```

**TiKV:**
```
Linearizable Reads/Writes (optional):
  - Read Index from leader
  - Ensures reads see latest committed writes
  
Snapshot Isolation:
  - MVCC timestamps
  - Transactions see consistent snapshot
  - Causality preserved across regions
```

### 4. Storage Engine

**Pyralog:**
```rust
// Log-structured, append-only
pub struct LogStorage {
    segments: Vec<Segment>,     // Sequential files
    active_segment: Segment,    // Current write target
    index: SparseIndex,         // Offset → file position
}

// Optimized for:
✓ Sequential writes (1M+ writes/sec)
✓ Sequential reads (scanning)
✓ Time-series data
✗ Random updates
```

**TiKV:**
```rust
// RocksDB (LSM Tree)
pub struct RocksDBEngine {
    db: Arc<DB>,
    cf_handles: HashMap<String, ColumnFamily>,
}

// Optimized for:
✓ Random reads (point queries)
✓ Range scans
✓ Updates and deletes
✓ Compaction (merging SSTables)
```

### 5. Metadata Management

```
┌─────────────────────────────────────────────────────────┐
│   Pyralog: Distributed Metadata                            │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Global Raft cluster (embedded):                        │
│    - All nodes participate                             │
│    - Metadata replicated to all nodes                  │
│    - No separate metadata service                      │
│                                                         │
│  Advantages:                                            │
│    ✓ Simple deployment (single binary)                 │
│    ✓ No external dependencies                          │
│    ✗ Metadata on all nodes (memory overhead)           │
│                                                         │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│   TiKV: Centralized Metadata (PD)                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Placement Driver (separate service):                   │
│    - Dedicated metadata cluster (3-5 nodes)            │
│    - Global view of cluster state                      │
│    - Schedules region moves, splits, merges            │
│                                                         │
│  Advantages:                                            │
│    ✓ Global optimization (load balancing)              │
│    ✓ Advanced scheduling policies                      │
│    ✗ Additional component to manage                    │
│    ✗ PD becomes critical path                          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 6. Client Protocol

**Pyralog: Smart Clients (Kafka-style)**
```rust
// Client fetches metadata, routes directly
let metadata = client.fetch_metadata(log_id).await?;
let leader = metadata.get_leader(partition_id);
client.send_to_leader(leader, record).await?;

// Benefits:
✓ No proxy overhead
✓ Direct routing (1 hop)
✓ Client-side load balancing
```

**TiKV: Region Cache + PD**
```rust
// Client caches region info, queries PD on miss
let region = client.get_region_by_key(key).await?;
if region.leader_changed {
    region = client.query_pd(key).await?;
}
client.send_to_region(region, kv_request).await?;

// Benefits:
✓ Dynamic region routing
✓ PD handles complexity
✗ PD query on cache miss
```

---

## Multi-Raft Comparison

### Resource Usage

**Pyralog (5 nodes, 1000 partitions, RF=3):**
```
Per Node:
  Global Raft: 1 group
  Partition Rafts: ~600 groups (60% of partitions)
  Total: ~601 Raft groups

Memory: 601 × 10 KB ≈ 6 MB
Disk: 601 × 1 MB ≈ 600 MB
Network: ~1200 heartbeats/sec
```

**TiKV (5 nodes, 1000 regions, RF=3):**
```
Per Node:
  Region Rafts: ~600 groups (60% of regions)
  PD client: Metadata cache
  Total: ~600 Raft groups

Memory: 600 × 10 KB ≈ 6 MB (similar!)
Disk: 600 × 1 MB ≈ 600 MB (RocksDB WAL)
Network: ~1200 heartbeats/sec + PD reports
```

**Conclusion**: Multi-Raft overhead is similar in both systems!

### Failure Handling

**Pyralog Partition Leader Failure:**
```
1. Per-Partition Raft detects leader failure
   → Partition 0: [N1, N2, N3], N1 fails
   
2. Election among partition replicas
   → N2 or N3 elected (10-300ms)
   
3. New leader activates new epoch
   → Proposes to Partition 0 Raft
   → Committed when majority of [N2, N3] ack
   
4. Clients redirect to new leader
   → Metadata updated via Global Raft
   
Time: ~300ms total
```

**TiKV Region Leader Failure:**
```
1. Per-Region Raft detects leader failure
   → Region 0: [N1, N2, N3], N1 fails
   
2. Election among region replicas
   → N2 or N3 elected (10-300ms)
   
3. New leader reports to PD
   → PD updates region metadata
   → Broadcasts to all TiKV nodes
   
4. Clients fetch new metadata from PD
   → Cache updated
   
Time: ~300ms total (similar!)
```

### Scalability

**Pyralog:**
```
Static partitions:
  - Partition count set at creation
  - Rebalance on node add/remove
  - No automatic splitting
  
Scaling strategy:
  1. Over-provision partitions (100-1000)
  2. Distribute across nodes
  3. Rebalance when adding nodes
  
Best for:
  ✓ Predictable workloads
  ✓ Known partition count
  ✗ Highly uneven key distributions
```

**TiKV:**
```
Dynamic regions:
  - Regions split when they grow
  - Regions merge when they shrink
  - PD schedules region moves
  
Scaling strategy:
  1. Start with few regions
  2. Automatic splitting as data grows
  3. PD rebalances automatically
  
Best for:
  ✓ Unpredictable growth
  ✓ Uneven key distributions
  ✓ Automatic rebalancing
```

---

## Use Cases

### Pyralog Excels At

**1. Event Streaming**
```rust
// High-throughput event ingestion
for event in events {
    dlog.append(event).await?;
}

// Sequential consumption
let records = dlog.read_from_offset(offset, 1000).await?;
```

**2. Message Queues**
```rust
// Multiple consumers, each partition
consumer_group.consume(log_id, |record| {
    process(record);
}).await?;
```

**3. Change Data Capture**
```rust
// Database changes → Pyralog → downstream systems
db.on_change(|change| {
    dlog.append(change).await;
});
```

**4. Time-Series Data**
```rust
// Metrics, logs, traces
metrics.on_sample(|sample| {
    dlog.append(sample).await;
});
```

### TiKV Excels At

**1. Database Storage**
```rust
// SQL queries via TiDB
SELECT * FROM users WHERE id = 123;
// → TiKV get(key="users:123")

UPDATE users SET name = 'Alice' WHERE id = 123;
// → TiKV put(key="users:123", value=...)
```

**2. Distributed Transactions**
```rust
// Multi-key transaction
txn.begin();
txn.put("account:1", balance - 100);
txn.put("account:2", balance + 100);
txn.commit()?; // ACID guaranteed
```

**3. Key-Value Workloads**
```rust
// Session storage, caching, etc.
tikv.put("session:abc", session_data).await?;
let session = tikv.get("session:abc").await?;
```

**4. Metadata Storage**
```rust
// Application metadata, configuration
tikv.put("config:feature_flags", flags).await?;
```

---

## Performance Characteristics

### Throughput

```
┌──────────────────────────────────────────────────────────┐
│   Write Throughput (10 nodes, RF=3)                      │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Pyralog:                                                    │
│    Sequential writes: 10M+ records/sec                   │
│    Per partition: 1M+ records/sec                        │
│    Batch size: 1000 records                              │
│                                                          │
│  TiKV:                                                    │
│    Random writes: 200K+ ops/sec                          │
│    Per region: ~20K ops/sec                              │
│    Transaction overhead: ~2-3ms per txn                  │
│                                                          │
│  Winner: Pyralog (50x higher for sequential writes)         │
│                                                          │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│   Read Throughput (10 nodes, RF=3)                       │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Pyralog:                                                    │
│    Sequential scans: 30M+ records/sec                    │
│    Random reads: 1M+ reads/sec                           │
│    (mmap + zero-copy)                                    │
│                                                          │
│  TiKV:                                                    │
│    Point reads: 500K+ ops/sec                            │
│    Range scans: 100K+ ops/sec                            │
│    (RocksDB block cache)                                 │
│                                                          │
│  Winner: Depends on access pattern                       │
│    - Sequential: Pyralog                                    │
│    - Random: TiKV                                        │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Latency

```
┌──────────────────────────────────────────────────────────┐
│   Write Latency (p99)                                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Pyralog:                                                    │
│    With write cache: < 1ms                               │
│    Durable (fsync): ~5ms                                 │
│    Replication: ~10ms (quorum)                           │
│                                                          │
│  TiKV:                                                    │
│    Single put: ~10ms                                     │
│    Transaction: ~20ms                                    │
│    (2PC + Raft commit)                                   │
│                                                          │
│  Winner: Pyralog (10x lower latency)                        │
│                                                          │
└──────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│   Read Latency (p99)                                      │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Pyralog:                                                    │
│    Sequential (hot): < 0.5ms (mmap)                      │
│    Random (hot): ~1ms                                    │
│    Cold: ~10ms (disk seek)                               │
│                                                          │
│  TiKV:                                                    │
│    Point read (hot): ~1ms                                │
│    Point read (cold): ~5ms                               │
│    Range scan: ~10ms                                     │
│                                                          │
│  Winner: Similar for hot data, Pyralog for scans            │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Scalability

Both scale linearly with nodes:

```
Pyralog:
  10 nodes: 10M writes/sec
  20 nodes: 20M writes/sec
  Limit: Network bandwidth

TiKV:
  10 nodes: 200K writes/sec
  20 nodes: 400K writes/sec
  Limit: Transaction coordination
```

---

## Design Philosophy

### Pyralog: Optimized for Streaming

**Priorities:**
1. ⭐ Sequential write throughput
2. ⭐ Low-latency appends
3. ⭐ Efficient sequential scans
4. ✅ Immutability (simplifies replication)
5. ✅ Simple deployment

**Trade-offs:**
- ❌ No random updates
- ❌ No transactions
- ❌ Static partitioning

### TiKV: Optimized for Databases

**Priorities:**
1. ⭐ ACID transactions
2. ⭐ Random access performance
3. ⭐ Dynamic load balancing
4. ✅ Mutable data
5. ✅ Strong consistency

**Trade-offs:**
- ⚠️  Transaction overhead (2PC)
- ⚠️  Complex deployment (PD + TiKV)
- ⚠️  Lower write throughput

---

## When to Use Which

### Use Pyralog When

✅ **Event streaming is the primary use case**
```
Examples:
  - Click streams
  - Application logs
  - IoT sensor data
  - CDC from databases
  - Message queues
```

✅ **Sequential access dominates**
```
Workload:
  - Append at end
  - Read from beginning
  - Process in order
```

✅ **High throughput > transactions**
```
Requirements:
  - 1M+ events/sec
  - Sub-millisecond latency
  - No need for updates/deletes
```

✅ **Simpler operations preferred**
```
Deployment:
  - Single binary
  - No external dependencies
  - Kafka-like semantics
```

### Use TiKV When

✅ **Need a distributed database**
```
Examples:
  - SQL database (via TiDB)
  - Key-value store
  - Distributed cache
  - Metadata storage
```

✅ **Transactions are essential**
```
Workload:
  - Multi-key updates
  - ACID guarantees
  - Snapshot isolation
```

✅ **Random access dominates**
```
Access pattern:
  - Point queries by key
  - Updates and deletes
  - Range scans
```

✅ **Dynamic scaling needed**
```
Requirements:
  - Unpredictable growth
  - Automatic rebalancing
  - Hot key handling
```

---

## Hybrid Approaches

### Can They Work Together?

**YES!** Common patterns:

**Pattern 1: Pyralog → TiKV**
```
Application → Pyralog (event stream)
              ↓
           Consumer → TiKV (materialized views)
```

Example:
```rust
// Write events to Pyralog (high throughput)
dlog.append(Event {
    user_id: 123,
    action: "purchase",
    amount: 99.99,
}).await?;

// Consume and materialize to TiKV
consumer.process(|event| {
    // Update user stats in TiKV
    tikv.transaction(|txn| {
        let stats = txn.get("user_stats:123")?;
        stats.total_spent += event.amount;
        txn.put("user_stats:123", stats)?;
        txn.commit()
    }).await?;
}).await?;
```

**Pattern 2: Database CDC**
```
TiDB → Pyralog (change stream)
        ↓
     Downstream consumers
```

Example:
```rust
// Capture TiDB changes to Pyralog
tidb.on_change(|change| {
    dlog.append(change).await;
});

// Multiple consumers process changes
search_indexer.consume_from(dlog);
analytics.consume_from(dlog);
cache_invalidator.consume_from(dlog);
```

---

## Conclusion

### Similarities

Both Pyralog and TiKV:
- ✅ Use multi-Raft for scalability
- ✅ Written in Rust for performance
- ✅ Support 1000+ shards per node
- ✅ Provide strong consistency
- ✅ Scale linearly with nodes

### Key Differences

| Feature | Pyralog | TiKV |
|---------|------|------|
| **Data Model** | Append-only log | Mutable key-value |
| **Access Pattern** | Sequential | Random |
| **Transactions** | No (by design) | Yes (ACID) |
| **Write Throughput** | 10M+ ops/sec | 200K+ ops/sec |
| **Latency** | < 1ms | ~10ms |
| **Deployment** | Single binary | Multi-component (PD + TiKV) |
| **Use Case** | Streaming, logs | Database, storage |

### The Big Picture

```
┌─────────────────────────────────────────────────────────┐
│   Storage Spectrum                                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Append-Only ←──────────────────────→ Mutable          │
│                                                         │
│  Pyralog            Kafka          TiKV        PostgreSQL │
│  ↓                ↓              ↓              ↓       │
│  Pure log    Event stream   KV store    Relational DB │
│                                                         │
│  High throughput ←───────────────────→ Rich features   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Bottom Line:**
- **Pyralog**: Best-in-class distributed log for streaming workloads
- **TiKV**: Best-in-class distributed KV store for database workloads
- **Together**: Complementary, can be used in the same architecture!

Both systems prove that **multi-Raft is the key to scaling distributed systems** beyond single-cluster limitations! 🚀

