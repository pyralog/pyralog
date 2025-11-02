# Pyralog Architecture

This document provides a deep dive into Pyralog's architecture, design decisions, and implementation details.

## Table of Contents

1. [Overview](#overview)
2. [Core Components](#core-components)
3. [Storage Engine](#storage-engine)
4. [Consensus Protocol](#consensus-protocol)
5. [Replication System](#replication-system)
6. [Network Protocol](#network-protocol)
7. [Performance Optimizations](#performance-optimizations)
8. [Failure Scenarios](#failure-scenarios)
9. [Scalability](#scalability)
10. [Monitoring and Observability](#monitoring-and-observability)
11. [Conclusion](#conclusion)

## Overview

Pyralog is a distributed log system designed for:
- **High throughput**: Millions of writes per second
- **Low latency**: Sub-millisecond write latencies
- **Strong durability**: Configurable replication and persistence
- **High availability**: Tolerates node failures
- **Horizontal scalability**: Add nodes to increase capacity

## Core Components

### 1. pyralog-core

Provides fundamental abstractions:

```
pyralog-core/
├── error.rs          # Error types
├── log.rs            # Log metadata
├── offset.rs         # Offset types
├── partition.rs      # Partition types
├── record.rs         # Record and batch types
└── traits.rs         # Core traits
```

**Key Types:**
- `LogOffset`: 64-bit offset in a log
- `Record`: Single log record with key, value, headers
- `RecordBatch`: Batch of records for efficient I/O
- `LogId`: Namespaced log identifier
- `PartitionId`: Partition identifier within a log

### 2. pyralog-storage

High-performance storage engine:

```
pyralog-storage/
├── segment.rs        # Segment file management
├── index.rs          # Offset index
├── log_storage.rs    # Main storage interface
├── write_cache.rs    # Write caching
└── tiered.rs         # Tiered storage
```

**Design Principles:**
- Log-structured storage
- Sequential writes for performance
- Sparse indexes for fast lookups
- Memory-mapped I/O for zero-copy reads

### 3. pyralog-consensus

Raft-based consensus protocol:

```
pyralog-consensus/
├── raft.rs           # Main Raft implementation
├── state.rs          # Node state management
├── rpc.rs            # RPC types
├── election.rs       # Leader election
└── log.rs            # Persistent log
```

**Responsibilities:**
- Cluster membership
- Leader election
- Metadata replication
- Configuration changes

### 4. pyralog-replication

Flexible quorum-based replication:

```
pyralog-replication/
├── quorum.rs         # Quorum configuration
├── copyset.rs        # CopySet selection
├── replicator.rs     # Replication manager
└── sync.rs           # Synchronization
```

**Features:**
- Configurable quorums
- CopySet replication
- ISR tracking
- Replication lag monitoring

### 5. pyralog-protocol

Protocol abstraction layer:

```
pyralog-protocol/
├── api.rs            # API types
├── partitioner.rs    # Partitioning strategies
├── kafka.rs          # Kafka compatibility
├── request.rs        # Request wire format
└── response.rs       # Response wire format
```

## Storage Engine

### Segment-Based Storage

Data is organized into segments:

```
log-namespace/
└── log-name/
    └── partition-0/
        ├── 00000000000000000000.log      # Segment
        ├── 00000000000000000000.index    # Index
        ├── 00000000000001000000.log      # Next segment
        └── 00000000000001000000.index    # Next index
```

**Segment Properties:**
- Fixed maximum size (default: 1GB)
- Immutable once full
- Can be memory-mapped for reads
- Atomic writes

### Index Structure

Sparse index for fast offset lookups:

```
Index Entry: [Offset (8 bytes)][Position (8 bytes)][Size (4 bytes)]
```

**Properties:**
- Not every record is indexed (sparse)
- Typically one entry per 4KB
- Entire index loaded in memory
- Binary search for lookups

### Write Cache

In-memory write buffer:

```
Write Cache
├── Buffer: VecDeque<Record>
├── Total Size: usize
├── Last Flush: Instant
└── Config
    ├── Max Size: 16MB
    └── Max Time: 10ms
```

**Benefits:**
- Reduced write latency
- Batch multiple writes
- Configurable durability/latency tradeoff

## Consensus Protocol

### Raft Cluster Topology

**Key Question**: Do we need one global Raft cluster or per-partition Raft clusters?

**Answer**: Pyralog uses **BOTH** (dual Raft clusters):

```
┌────────────────────────────────────────────────────────────┐
│   Dual Raft Cluster Architecture                           │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  1. GLOBAL RAFT CLUSTER (Metadata)                         │
│     ────────────────────────────────────                   │
│     All nodes participate                                  │
│     Purpose: Cluster-wide operations                       │
│                                                            │
│     Operations:                                            │
│     ✓ Cluster membership changes                          │
│     ✓ Partition creation/deletion                         │
│     ✓ CopySet assignments (per-partition mode)            │
│     ✓ Configuration changes                               │
│                                                            │
│     Example:                                               │
│     [N1, N2, N3, N4, N5] → One Raft group                 │
│                                                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  2. PER-PARTITION RAFT CLUSTERS (Epochs)                   │
│     ──────────────────────────────────                     │
│     Only partition replicas participate                    │
│     Purpose: Partition-specific operations                 │
│                                                            │
│     Operations:                                            │
│     ✓ Epoch activation (leader election for partition)    │
│     ✓ Epoch sealing (leadership transfer)                 │
│     ✓ Partition-level failover                            │
│                                                            │
│     Example:                                               │
│     Partition 0: [N1, N2, N3] → Raft group for P0        │
│     Partition 1: [N2, N3, N4] → Raft group for P1        │
│     Partition 2: [N3, N4, N5] → Raft group for P2        │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Why Dual Clusters?**

```
Scalability Trade-off:
──────────────────────────────────────────────────
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

### Node Membership in Raft Clusters

Each node is a member of **1 global + N partition Raft clusters**:

```rust
pub struct NodeRaftMembership {
    node_id: NodeId,
    
    // 1. Global cluster (always)
    global_raft: Arc<RaftNode>,
    
    // 2. Per-partition clusters (for partitions this node replicates)
    partition_rafts: HashMap<PartitionId, Arc<RaftNode>>,
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

### Raft State Machine

```
                    Follower
                       │
                       │ election timeout
                       ▼
                   Candidate
                       │
                       │ receives votes from majority
                       ▼
                     Leader
                       │
                       │ discovers higher term
                       ▼
                    Follower
```

### Which Raft Cluster Does What?

```
┌─────────────────────────────────────────────────────────────┐
│   Operation Flow: Global vs Per-Partition Raft              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  CLUSTER-WIDE OPERATIONS → Global Raft                      │
│  ──────────────────────────────────────────                 │
│                                                             │
│  1. Add Node to Cluster                                     │
│     Client → Any Node → Propose to Global Raft             │
│     Global Raft commits → All nodes updated                │
│                                                             │
│  2. Create New Partition                                    │
│     Admin API → Global Raft                                │
│     Assigns partition ID, initial CopySet                  │
│     Creates per-partition Raft group                       │
│                                                             │
│  3. Reassign CopySet (rebalancing)                          │
│     Admin API → Global Raft                                │
│     Updates CopySet metadata                               │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  PARTITION-SPECIFIC OPERATIONS → Per-Partition Raft         │
│  ────────────────────────────────────────────────           │
│                                                             │
│  1. Leader Election for Partition                           │
│     Partition 0's Raft: [N1, N2, N3]                       │
│     N1 fails → N2 or N3 elected leader                     │
│     Only involves partition replicas!                      │
│                                                             │
│  2. Epoch Activation                                        │
│     New leader N2 for Partition 0                          │
│     N2 proposes epoch change to P0's Raft                  │
│     Only [N1, N2, N3] vote (fast!)                         │
│                                                             │
│  3. Epoch Sealing                                           │
│     Leader N2 seals old epoch                              │
│     Proposes to P0's Raft                                  │
│     Committed when majority of [N1, N2, N3] ack            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Concrete Example: Node Failure**

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

### Benefits of Dual Raft

```
Parallelism:
──────────────────────────────────────────────────
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

### Log Replication Flow

```
Leader                      Follower
  │                            │
  │──── AppendEntries ────────►│
  │    (entries, commitIndex)  │
  │                            │
  │                            │ Apply entries
  │                            │ Update commitIndex
  │                            │
  │◄───── Success ─────────────│
  │    (matchIndex)            │
  │                            │
  │ Update commitIndex         │
  │                            │
```

### Election Process

1. **Follower timeout**: No heartbeat from leader
2. **Become candidate**: Increment term, vote for self
3. **Request votes**: Send RequestVote RPC to all peers
4. **Collect votes**: Wait for majority
5. **Become leader**: If majority votes received
6. **Send heartbeats**: Establish authority

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
    groups: HashMap<PartitionId, RaftNode>,
    
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

## Replication System

### CopySet Selection Strategies

Pyralog supports **two CopySet selection strategies**, configurable based on your needs:

#### Strategy 1: Per-Partition CopySet (Kafka-style)

**Fixed CopySet per partition:**

```rust
pub struct PartitionCopySet {
    partition_id: PartitionId,
    nodes: Vec<NodeId>,  // Fixed: [N1, N2, N3]
}

// All records in partition 0 → Always [N1, N2, N3]
// All records in partition 1 → Always [N2, N3, N4]
```

**Advantages:**
- ✅ Simpler to implement
- ✅ Easier to reason about
- ✅ Faster lookups (cached per partition)
- ✅ Good for small clusters (< 10 nodes)

**Disadvantages:**
- ❌ Less uniform load distribution
- ❌ Hot partitions still overload same nodes
- ❌ Fixed replica sets

**Best for:**
- Smaller deployments
- Simpler operations
- Kafka-compatible behavior

#### Strategy 2: Per-Record CopySet (LogDevice-style)

**Dynamic CopySet per record/batch:**

```rust
pub struct RecordCopySet {
    lsn: u64,  // Log Sequence Number (epoch + offset)
    nodes: Vec<NodeId>,  // Calculated: hash(lsn) → [N1, N5, N7]
}

// Record @ LSN 1000 → hash(1000) → [N1, N5, N7]
// Record @ LSN 1001 → hash(1001) → [N2, N3, N6]
// Record @ LSN 1002 → hash(1002) → [N1, N4, N8]
```

**Key Innovation: Leader as Coordinator**

With per-record CopySet, the **leader doesn't need to store data locally**:

```
Traditional (Leader Stores Data):
──────────────────────────────────────────
Client → Leader → Write to local disk
                → Replicate to followers
                
Leader role: Storage + Coordinator
Leader disk: Heavy I/O (ALL partition data)

Per-Record CopySet (Leader Coordinates Only):
──────────────────────────────────────────
Client → Leader → Calculate CopySet
                → Send to storage nodes
                
Leader role: Coordinator ONLY
Leader disk: Metadata only (epochs, offsets)
Reduction: 99%+ less leader I/O! 🚀
```

**Advantages:**
- ✅ Maximum load distribution
- ✅ Hot keys don't overload same nodes
- ✅ Better fault tolerance
- ✅ No metadata storage needed (deterministic)
- ✅ **Leader can be disk-free** (just coordinates)
- ✅ **Leader can handle 10x more partitions** (no storage overhead)
- ✅ **Disk failure doesn't affect leadership** (no local data)

**Disadvantages:**
- ❌ More complex implementation
- ❌ Readers must calculate CopySet
- ❌ Slightly more CPU for hash calculation
- ❌ Can't read directly from leader (unless hybrid mode)

**Best for:**
- Large clusters (10+ nodes)
- Uneven key distributions
- Maximum performance
- High partition count per node

### Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CopySetStrategy {
    /// Fixed CopySet per partition
    PerPartition,
    
    /// Dynamic CopySet per record (LogDevice-style)
    PerRecord {
        /// Seed for deterministic selection
        seed: u64,
        
        /// Should leader store data locally?
        /// false = Leader is pure coordinator (LogDevice-style)
        /// true = Leader also stores data (hybrid mode)
        leader_stores_data: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    pub replication_factor: usize,
    pub write_quorum: usize,
    pub read_quorum: usize,
    
    /// CopySet selection strategy
    pub copyset_strategy: CopySetStrategy,
}
```

**Configuration file:**

```toml
[replication]
replication_factor = 3
write_quorum = 2
read_quorum = 2

# Option 1: Per-partition (simpler)
copyset_strategy = "PerPartition"

# Option 2: Per-record with leader as coordinator (maximum performance)
[replication.copyset_strategy]
type = "PerRecord"
seed = 42
leader_stores_data = false  # Leader disk-free! 🚀

# Option 3: Per-record hybrid (leader also stores)
[replication.copyset_strategy]
type = "PerRecord"
seed = 42
leader_stores_data = true   # Leader can serve reads
```

### Implementation: Per-Partition CopySet

```rust
pub struct PartitionCopySetSelector {
    // Stored in metadata (Raft + RocksDB)
    assignments: Arc<RwLock<HashMap<PartitionId, Vec<NodeId>>>>,
    replication_factor: usize,
}

impl PartitionCopySetSelector {
    pub fn select(&self, partition: PartitionId) -> Vec<NodeId> {
        // Simple lookup - O(1)
        self.assignments
            .read()
            .get(&partition)
            .cloned()
            .unwrap_or_default()
    }
    
    pub async fn assign(&self, partition: PartitionId) -> Result<Vec<NodeId>> {
        // Round-robin or random selection
        let nodes = self.select_nodes_for_partition(partition);
        
        // Store in metadata via Raft consensus
        self.propose_assignment(partition, nodes.clone()).await?;
        
        // Cache locally
        self.assignments.write().insert(partition, nodes.clone());
        
        Ok(nodes)
    }
}
```

### Implementation: Per-Record CopySet

```rust
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
        
        // Select RF unique nodes using hash
        let mut selected = Vec::with_capacity(self.replication_factor);
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
    
    // NO storage or consensus needed!
    // Any node can calculate the same CopySet for a given LSN
}
```

### Write Path with Both Strategies

```rust
pub struct Replicator {
    strategy: CopySetStrategy,
    partition_selector: Option<PartitionCopySetSelector>,
    record_selector: Option<RecordCopySetSelector>,
    node_id: NodeId,
}

impl Replicator {
    pub async fn replicate(&self, record: Record) -> Result<()> {
        // Select CopySet based on strategy
        let (copyset, include_leader) = match &self.strategy {
            CopySetStrategy::PerPartition => {
                // Lookup partition's fixed CopySet
                let selector = self.partition_selector.as_ref().unwrap();
                let copyset = selector.select(record.partition_id);
                (copyset, true)  // Leader always stores data
            }
            
            CopySetStrategy::PerRecord { seed, leader_stores_data } => {
                // Calculate CopySet from LSN
                let selector = self.record_selector.as_ref().unwrap();
                let lsn = record.epoch_offset.as_u64();
                let mut copyset = selector.select(lsn);
                
                // If leader_stores_data=true, ensure leader is in CopySet
                if *leader_stores_data && !copyset.contains(&self.node_id) {
                    copyset.push(self.node_id);
                }
                
                (copyset, *leader_stores_data)
            }
        };
        
        // Write to local storage (if configured)
        if include_leader && copyset.contains(&self.node_id) {
            self.local_storage.append(record.clone()).await?;
        }
        
        // Send to remote nodes in CopySet
        let futures: Vec<_> = copyset.iter()
            .filter(|&&node| node != self.node_id)  // Skip self if already written
            .map(|&node| self.send_to_node(node, record.clone()))
            .collect();
        
        // Wait for write quorum
        self.wait_for_quorum(futures).await
    }
}
```

**Leader as Pure Coordinator (leader_stores_data=false):**

```rust
// Leader (Sequencer) - Lightweight, no storage!
pub struct Sequencer {
    partition_id: PartitionId,
    current_epoch: Epoch,
    next_offset: AtomicU64,
    copyset_selector: RecordCopySetSelector,
    // NO local_storage field!
}

impl Sequencer {
    pub async fn handle_write(&self, record: Record) -> Result<LogOffset> {
        // 1. Assign LSN (metadata only, no disk I/O!)
        let epoch = self.current_epoch;
        let offset = LogOffset::new(
            self.next_offset.fetch_add(1, Ordering::SeqCst)
        );
        let lsn = EpochOffset::new(epoch, offset.as_u64()).as_u64();
        
        // 2. Calculate CopySet (pure function, deterministic)
        let copyset = self.copyset_selector.select(lsn);
        // → [Node 3, Node 7, Node 9]
        
        // 3. Prepare record with LSN
        let mut record = record;
        record.epoch = epoch;
        record.offset = offset;
        
        // 4. Send directly to storage nodes (NOT local disk!)
        for node in copyset {
            self.send_to_storage_node(node, record.clone()).await?;
        }
        
        // 5. Wait for write quorum
        self.wait_for_quorum(copyset.len()).await?;
        
        // 6. Done! Leader never touched disk! ✅
        Ok(offset)
    }
}
```

**Storage Node (Receives and Stores):**

```rust
// Storage node - Stores data selected by CopySet
pub struct StorageNode {
    node_id: NodeId,
    storage: LogStorage,
}

impl StorageNode {
    pub async fn handle_write(&self, record: Record) -> Result<()> {
        // Storage node writes to disk
        self.storage.append(record).await?;
        Ok(())
    }
}
```

### Read Path with Both Strategies

```rust
impl Reader {
    pub async fn read(&self, partition: PartitionId, offset: LogOffset) -> Result<Record> {
        // Find which nodes have this record
        let copyset = match &self.strategy {
            CopySetStrategy::PerPartition => {
                // Lookup partition's fixed CopySet
                self.partition_selector.select(partition)
            }
            
            CopySetStrategy::PerRecord { seed } => {
                // Calculate from LSN (epoch + offset)
                let lsn = self.calculate_lsn(partition, offset)?;
                self.record_selector.select(lsn)
            }
        };
        
        // Try reading from any node in the CopySet
        for node in copyset {
            if let Ok(record) = self.try_read_from(node, partition, offset).await {
                return Ok(record);
            }
        }
        
        Err(PyralogError::RecordNotFound(offset))
    }
}
```

### Leader as Coordinator: Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│   Traditional: Leader Stores All Partition Data         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Node 1 (Leader for P0):                                │
│    ├─ Sequencer (assigns offsets)                       │
│    ├─ Local Storage (/segments/P0/)     ← Heavy I/O ⚠️  │
│    │  └─ All records for partition 0                    │
│    └─ Replicates to followers                           │
│                                                         │
│  Node 2, Node 3 (Followers):                            │
│    └─ Receive replicas of partition 0                   │
│                                                         │
│  Problem: Leader disk is bottleneck                     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│   Per-Record CopySet: Leader Coordinates Only           │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Node 1 (Leader for P0):                                │
│    ├─ Sequencer (assigns LSN)                           │
│    ├─ NO local storage! ✅                               │
│    └─ Routes to storage nodes                           │
│         │                                               │
│         ├──► Record @ LSN 1000 → CopySet [N3, N7, N9]  │
│         ├──► Record @ LSN 1001 → CopySet [N2, N5, N8]  │
│         └──► Record @ LSN 1002 → CopySet [N4, N6, N7]  │
│                                                         │
│  Nodes 2-10 (Storage Nodes):                            │
│    └─ Each stores subset of records                     │
│       Based on deterministic CopySet selection          │
│                                                         │
│  Result: Leader is lightweight! 🚀                      │
└─────────────────────────────────────────────────────────┘
```

**Leader Resource Usage:**

```
┌──────────────────────────────────────────────────┐
│   Leader Disk I/O Comparison                     │
├──────────────────────────────────────────────────┤
│                                                  │
│  Per-Partition CopySet:                          │
│    Disk writes: 100 GB/hour (all data)          │
│    Disk reads: 50 GB/hour (serves reads)        │
│    Total I/O: 150 GB/hour ⚠️                     │
│                                                  │
│  Per-Record CopySet (leader_stores_data=false):  │
│    Disk writes: 10 MB/hour (metadata only)      │
│    Disk reads: 5 MB/hour (metadata only)        │
│    Total I/O: 15 MB/hour ✅                      │
│                                                  │
│  Reduction: 99.99% less I/O! 🎉                  │
│                                                  │
│  Leader can handle:                              │
│    Before: 10-20 partitions                     │
│    After: 200-500 partitions                    │
│    Increase: 20x-50x! 🚀                         │
│                                                  │
└──────────────────────────────────────────────────┘
```

**Benefits for Large Deployments:**

```
Scenario: 1000 partitions, 20 nodes

Per-Partition CopySet:
  Each node leads: 50 partitions
  Each partition: 10 GB data
  Leader storage: 50 × 10 GB = 500 GB per node
  Problem: Disk-bound! 💥

Per-Record CopySet (coordinator mode):
  Each node leads: 50 partitions
  Each partition: Metadata only
  Leader storage: 50 × 1 MB = 50 MB per node
  Solution: CPU-bound (better!) ✅
  
  Actual data: Distributed across all 20 nodes
  Each node stores: ~1/3 of total data (RF=3)
  Balanced load across cluster!
```

### Load Distribution Comparison

```
Scenario: 10 nodes, 100 partitions, RF=3, 1M records

Per-Partition CopySet:
──────────────────────────────────────────
Partition 0 → [N1, N2, N3]  (10K records)
Partition 1 → [N2, N3, N4]  (10K records)
...
Partition 99 → [N7, N8, N9] (10K records)

If partition 0 is hot (100K records):
  N1, N2, N3: 100K records each ⚠️ 
  N4-N10: 10K records each
  
Imbalance: 10x difference!

Per-Record CopySet:
──────────────────────────────────────────
Record 1 → hash(1) → [N1, N3, N5]
Record 2 → hash(2) → [N2, N4, N7]
Record 3 → hash(3) → [N1, N6, N8]
...

Even if 100K records have same key:
  Each record gets different CopySet
  All nodes: ~30K records each ✅
  
Imbalance: ~1.1x (much better!)
```

### When to Use Each Strategy

```
┌─────────────────────────────────────────────────────────┐
│   Decision Matrix                                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Use Per-Partition if:                                  │
│    • Cluster size < 10 nodes                            │
│    • Keys are well-distributed                          │
│    • Simplicity is priority                             │
│    • Kafka-compatible behavior needed                   │
│    • Lower partition count (< 100 partitions/node)      │
│    • Leader can handle storage load                     │
│                                                         │
│  Use Per-Record (leader_stores_data=false) if:          │
│    • Cluster size >= 10 nodes                           │
│    • Uneven key distribution / hot partitions           │
│    • High partition count (100s per node)               │
│    • Maximum performance needed                         │
│    • Large scale (billions of records)                  │
│    • Leader disk is bottleneck                          │
│    • Want to separate coordination from storage         │
│                                                         │
│  Use Per-Record (leader_stores_data=true) if:           │
│    • Want per-record distribution benefits              │
│    • But also want leader to serve reads                │
│    • Hybrid approach for migration                      │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Rule of thumb:**
- **< 10 nodes**: Start with per-partition (simpler)
- **10-50 nodes**: Per-record with leader storage (hybrid)
- **50+ nodes**: Per-record coordinator-only (maximum scale)

### Migration Between Strategies

You can change strategies online:

```rust
// Start with per-partition
config.copyset_strategy = CopySetStrategy::PerPartition;

// Later, migrate to per-record for better distribution
config.copyset_strategy = CopySetStrategy::PerRecord { seed: 42 };

// Old records: Still use partition CopySet (backward compatible)
// New records: Use per-record CopySet
```

**Benefits:**
- ✅ Both strategies in one system
- ✅ Choose based on cluster size and workload
- ✅ Can migrate as you scale
- ✅ Best of both worlds!

### Summary: Three Configuration Modes

```
┌────────────────────────────────────────────────────────────┐
│   Mode Comparison                                          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  MODE 1: Per-Partition CopySet                             │
│  ────────────────────────────────────────────              │
│  copyset_strategy = "PerPartition"                         │
│                                                            │
│  ✅ Simplest implementation                                 │
│  ✅ Kafka-compatible                                        │
│  ✅ Leader serves reads                                     │
│  ❌ Fixed replica sets                                      │
│  ❌ Hot partitions overload nodes                           │
│  ❌ Leader stores all data                                  │
│                                                            │
│  Best for: Small clusters (< 10 nodes)                     │
│  Partitions/node: 10-20                                    │
│                                                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  MODE 2: Per-Record with Leader Storage (Hybrid)           │
│  ────────────────────────────────────────────              │
│  [replication.copyset_strategy]                            │
│  type = "PerRecord"                                        │
│  seed = 42                                                 │
│  leader_stores_data = true                                 │
│                                                            │
│  ✅ Better load distribution                                │
│  ✅ Leader serves reads                                     │
│  ✅ Hot keys don't overload                                 │
│  ⚠️  Leader still stores data                               │
│                                                            │
│  Best for: Medium clusters (10-50 nodes)                   │
│  Partitions/node: 20-100                                   │
│                                                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  MODE 3: Per-Record Coordinator-Only (Maximum Scale)       │
│  ────────────────────────────────────────────              │
│  [replication.copyset_strategy]                            │
│  type = "PerRecord"                                        │
│  seed = 42                                                 │
│  leader_stores_data = false                                │
│                                                            │
│  ✅ Maximum load distribution                               │
│  ✅ Leader disk-free (99%+ less I/O)                        │
│  ✅ Leader handles 20x-50x more partitions                  │
│  ✅ Separation of coordination and storage                  │
│  ❌ Can't read from leader                                  │
│  ❌ More complex read path                                  │
│                                                            │
│  Best for: Large clusters (50+ nodes)                      │
│  Partitions/node: 100-500                                  │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

**Scalability Progression:**

```
Start Small → Grow Large → Maximum Scale
─────────────────────────────────────────────────
Mode 1      → Mode 2     → Mode 3
Per-Partition  Hybrid       Coordinator

10 nodes    → 30 nodes   → 100 nodes
20 parts    → 100 parts  → 500 parts/node
Simple      → Balanced   → Maximum perf
```

### CopySet Storage

CopySets are **cluster metadata**, stored separately from log data:

```
┌─────────────────────────────────────────────────┐
│         CopySet Storage Architecture            │
├─────────────────────────────────────────────────┤
│                                                 │
│  1. RAFT LOG (Consensus)                        │
│     ├─ CopySet assignments proposed            │
│     ├─ Cluster votes on assignments            │
│     └─ Committed to Raft log                   │
│                                                 │
│  2. METADATA STORE (Persistence)                │
│     ├─ RocksDB or similar KV store             │
│     ├─ Key: partition_id                       │
│     └─ Value: CopySetMetadata                  │
│                                                 │
│  3. IN-MEMORY CACHE (Performance)               │
│     ├─ ClusterManager holds map                │
│     ├─ Fast lookups during writes              │
│     └─ Refreshed from metadata store           │
│                                                 │
└─────────────────────────────────────────────────┘
```

**Data Structure:**

```rust
// Stored in metadata store
pub struct CopySetMetadata {
    pub partition_id: PartitionId,
    pub nodes: Vec<NodeId>,           // [Node1, Node2, Node3]
    pub leader: NodeId,                // Current leader
    pub created_at: SystemTime,
    pub last_modified: SystemTime,
}

// In-memory representation
pub struct ClusterManager {
    // Fast lookup: partition -> copyset
    copyset_cache: Arc<RwLock<HashMap<PartitionId, CopySetMetadata>>>,
    
    // Persistent store
    metadata_store: Arc<MetadataStore>,
    
    // Raft for consensus
    raft: Arc<RaftNode>,
}
```

**File System Layout:**

```
/var/lib/pyralog/
├── raft/
│   └── raft.log              ← CopySet changes in Raft log
├── metadata/
│   └── rocksdb/
│       └── copysets/         ← Persistent CopySet storage
│           ├── partition_0   → [N1, N2, N4]
│           ├── partition_1   → [N2, N3, N5]
│           └── partition_2   → [N1, N3, N6]
└── segments/
    └── partition_0/          ← Actual log data
        ├── 00000000.log
        └── 00000000.index
```

**Key Characteristics:**

```
CopySets (Metadata):
──────────────────────────────────────
Size: O(partitions × replication_factor)
      Small - hundreds of KB

Frequency: Changed rarely
          (only on rebalancing/failures)

Storage: Raft log + RocksDB
         Must be consistent across cluster

Access: Fast in-memory lookup
        Cached by ClusterManager

vs.

Log Records (Data):
──────────────────────────────────────
Size: O(millions/billions of records)
      Large - terabytes

Frequency: Changed constantly
          (every write)

Storage: Segment files on disk
         Per-replica, eventually consistent

Access: Disk I/O or mmap
        Indexed lookups
```

**CopySet Assignment Flow:**

```
New Partition Created:
──────────────────────────────────────────
1. Leader proposes CopySet assignment
   Propose([Node1, Node3, Node5])
   
2. Raft consensus
   Majority votes → Committed
   
3. Write to metadata store
   RocksDB: partition_id → [N1, N3, N5]
   
4. Update in-memory cache
   ClusterManager.copyset_cache.insert(...)
   
5. All nodes see consistent CopySet
   Used for all writes to this partition
   
Duration: Once per partition creation
Cost: ~100ms (Raft consensus)
```

**CopySet Lookup During Write:**

```
Write arrives for Partition 0:
──────────────────────────────────────────
1. Leader checks cache (fast!)
   copyset = copyset_cache.get(partition_0)
   → [Node1, Node2, Node4]
   
2. Replicate to these nodes
   Send to Node1, Node2, Node4 in parallel
   
3. No disk I/O for CopySet lookup
   Already in memory!
   
Duration: ~1 microsecond (hash map lookup)
Cost: Negligible
```

**Why Store in Raft?**

CopySets must be **strongly consistent** across the cluster:
- All nodes must agree on which nodes hold a partition
- Prevents split-brain (different nodes thinking different CopySets)
- Raft provides linearizable consensus
- Changes are rare, so Raft overhead is acceptable

**Comparison:**

```
┌─────────────────────────────────────────────────┐
│   What Gets Stored Where                        │
├─────────────────────────────────────────────────┤
│                                                 │
│  Raft Log + Metadata Store:                     │
│    ✓ CopySet assignments                        │
│    ✓ Epoch changes                              │
│    ✓ Cluster membership                         │
│    ✓ Leader elections                           │
│    ✗ NOT log records (too many!)               │
│                                                 │
│  Segment Files (Log Storage):                   │
│    ✓ Actual records                             │
│    ✓ Record epochs (tagged on each record)     │
│    ✓ Offsets                                    │
│    ✗ NOT CopySet info (would duplicate)        │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Flexible Quorums

```
Quorum Configuration:
- Replication Factor: R
- Write Quorum: W
- Read Quorum: R

Constraint: W + R > RF (ensures overlap)
```

**Examples:**

| Config | R | W | Rd | Use Case |
|--------|---|---|-----|----------|
| Majority | 3 | 2 | 2 | Balanced |
| Write-Heavy | 3 | 1 | 3 | Low write latency |
| Read-Heavy | 3 | 3 | 1 | Low read latency |
| Strong Consistency | 3 | 3 | 3 | Maximum durability |

### ISR (In-Sync Replicas)

Track which replicas are up-to-date:

```
Partition 0:
- Leader: Node 1 (offset: 1000)
- ISR: [Node 1, Node 2, Node 3]
- Follower Offsets:
  - Node 2: 998 (in sync, lag < 1000)
  - Node 3: 995 (in sync, lag < 1000)
  - Node 4: 500 (out of sync, lag > 1000)
```

## Network Protocol

### Smart Client Architecture

Pyralog uses the **smart client pattern** where clients fetch metadata and connect directly to partition leaders, avoiding proxy hops:

```
┌────────────────────────────────────────────────┐
│         Smart Client Flow                      │
├────────────────────────────────────────────────┤
│                                                │
│  1. Bootstrap & Metadata Discovery             │
│     Client → Any Server: MetadataRequest       │
│     Server → Client: {                         │
│       partition_0: leader=Node5,               │
│       partition_1: leader=Node3,               │
│       partition_2: leader=Node1                │
│     }                                          │
│                                                │
│  2. Client Caches Topology                     │
│     partition_cache[0] = Node5                 │
│     partition_cache[1] = Node3                 │
│     partition_cache[2] = Node1                 │
│                                                │
│  3. Direct Write to Leader                     │
│     Client calculates: hash(key) % 3 = 0       │
│     Client connects directly to Node5          │
│     No proxy overhead! ✅                      │
│                                                │
└────────────────────────────────────────────────┘
```

**Benefits:**
- ✅ Direct connection (1 hop vs 2)
- ✅ Client-side load balancing
- ✅ No proxy bottleneck
- ✅ Scales with cluster size

**Metadata includes:**
- Partition → Leader mapping
- Replica locations for reads
- ISR (In-Sync Replicas) status
- Node addresses and ports

For detailed flow and implementation, see [DATA_PATH.md](DATA_PATH.md#smart-client-architecture).

### Request/Response Flow

```
Client                      Server
  │                           │
  │──── ProduceRequest ──────►│
  │    [records]              │
  │                           │
  │                           │ Write to storage
  │                           │ Replicate
  │                           │
  │◄─── ProduceResponse ──────│
  │    [offset]               │
  │                           │
```

### Wire Format

```
Message Format:
┌────────────────┬──────────────────┬─────────────┐
│ Length (4B)    │ Request ID (8B)  │ Payload     │
└────────────────┴──────────────────┴─────────────┘

Payload (bincode serialized):
- Request Type
- Request Data
```

## Performance Optimizations

### 1. Zero-Copy I/O

```rust
// Memory-mapped file
let mmap = unsafe { Mmap::map(&file)? };

// Zero-copy read
let data = &mmap[offset..offset+length];
```

### 2. Batch Processing

```rust
// Batch multiple records
let batch = RecordBatch::new(base_offset, records);

// Single write operation
storage.append_batch(batch).await?;
```

### 3. Write Caching

```rust
// Add to cache (no disk I/O)
cache.push(record)?;

// Flush when needed
if cache.should_flush() {
    cache.drain_and_write().await?;
}
```

### 4. Async I/O

```rust
// Concurrent operations
let (r1, r2, r3) = tokio::join!(
    storage.append(rec1),
    storage.append(rec2),
    storage.append(rec3),
);
```

## Failure Scenarios

### Node Failure

```
Before:
Leader: Node 1
Followers: [Node 2, Node 3]

After Node 1 fails:
1. Followers detect missing heartbeats
2. Election timeout triggers
3. Node 2 becomes candidate
4. Node 2 wins election
5. Node 2 is new leader

Recovery time: ~300ms
```

### Network Partition

```
Partition scenario:
[Node 1] | [Node 2, Node 3]

With 3 nodes, majority is 2:
- Node 1 cannot form quorum (steps down)
- Nodes 2,3 can form quorum (elect new leader)

System continues operating on majority side
```

### Data Corruption

```
Detection:
1. CRC checksum on each batch
2. Verification on read
3. Replication checksum comparison

Recovery:
1. Detect corrupted segment
2. Request from replica
3. Rebuild from healthy copy
```

### Disk Failure

```
With tiered storage:
1. Detect disk failure
2. Mark node as degraded
3. Redirect reads to replicas
4. Background recovery from object storage
5. Rebuild local copy
```

## Scalability

### The Leader Bottleneck Problem

Pyralog uses a **leader-based architecture** where all writes for a partition must go through a single leader node. This creates a potential bottleneck:

```
┌─────────────────────────────────────────────────┐
│         Single Partition Write Path             │
├─────────────────────────────────────────────────┤
│                                                 │
│  All clients writing to Partition 0             │
│        │         │         │                    │
│        ▼         ▼         ▼                    │
│    ┌──────────────────────────┐                │
│    │   Leader Node 1          │ ← BOTTLENECK!  │
│    │   (Partition 0)          │                │
│    └───────┬──────────────────┘                │
│            │ Replicate                          │
│       ┌────┼────┐                               │
│       ▼    ▼    ▼                               │
│    Node2 Node3 Node4                            │
│   (Followers)                                   │
│                                                 │
│  Limit: Leader's CPU/Network/Disk              │
└─────────────────────────────────────────────────┘
```

**Why this happens:**
- All writes must flow through the partition leader
- Leader assigns offsets (with epochs for efficiency)
- Leader coordinates replication
- Single point of serialization per partition

**But this is a deliberate trade-off** for strong consistency and ordering guarantees!

### Solution 1: Distributed Leadership via Partitioning

Pyralog distributes leadership across the cluster through partitioning:

```
┌──────────────────────────────────────────────────────┐
│         Distributed Leadership                       │
├──────────────────────────────────────────────────────┤
│                                                      │
│  16 Partitions, 4 Nodes                              │
│                                                      │
│  Node 1 leads: Partitions [0, 4, 8, 12]            │
│  Node 2 leads: Partitions [1, 5, 9, 13]            │
│  Node 3 leads: Partitions [2, 6, 10, 14]           │
│  Node 4 leads: Partitions [3, 7, 11, 15]           │
│                                                      │
│  Client A ──→ hash("key-1") % 16 = 0 ──→ Node 1    │
│  Client B ──→ hash("key-2") % 16 = 5 ──→ Node 2    │
│  Client C ──→ hash("key-3") % 16 = 10 ──→ Node 3   │
│  Client D ──→ hash("key-4") % 16 = 15 ──→ Node 4   │
│                                                      │
│  Result: Writes distributed across ALL nodes! ✅    │
└──────────────────────────────────────────────────────┘
```

**Throughput scaling:**

```
Single partition:
  Leader throughput: 100K records/sec
  Total: 100K records/sec

16 partitions (4 nodes):
  16 leaders × 100K = 1.6M records/sec
  Scaling: 16x ✅

64 partitions (8 nodes):
  64 leaders × 100K = 6.4M records/sec
  Scaling: 64x ✅
```

### Solution 2: Read Scaling via Replicas

While writes must go through the leader, **reads can come from any replica**:

```
┌──────────────────────────────────────────────────┐
│         Write vs Read Paths                      │
├──────────────────────────────────────────────────┤
│                                                  │
│  WRITE PATH (must use leader):                  │
│  Client ──→ Leader ──→ Replicate ──→ Followers  │
│             Single                               │
│                                                  │
│  READ PATH (any replica):                        │
│  Client A ──→ Node 1 (any replica) ┐            │
│  Client B ──→ Node 2 (any replica) ├─ Load      │
│  Client C ──→ Node 3 (any replica) │  balanced! │
│  Client D ──→ Node 4 (any replica) ┘            │
│                                                  │
│  Read throughput with RF=3:                      │
│    1 partition × 3 replicas = 3x reads ✅        │
└──────────────────────────────────────────────────┘
```

**Configuration:**

```rust
// Allow reads from any replica (eventual consistency)
config.read_policy = ReadPolicy::AnyReplica;

// Or require leader reads (strong consistency)
config.read_policy = ReadPolicy::LeaderOnly;

// Or require read quorum
config.read_policy = ReadPolicy::Quorum(2);
```

### Solution 3: CopySet Distribution

Pyralog uses **non-deterministic replica placement** to distribute load:

```
┌──────────────────────────────────────────────────┐
│   Traditional Replication (Bottleneck)          │
├──────────────────────────────────────────────────┤
│                                                  │
│  All partitions use same replica set:            │
│    Partition 0: [Node 1, Node 2, Node 3]        │
│    Partition 1: [Node 1, Node 2, Node 3]        │
│    Partition 2: [Node 1, Node 2, Node 3]        │
│                                                  │
│  Problem: Nodes 1,2,3 always get all traffic!   │
│           Other nodes underutilized!             │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│   CopySet Replication (Pyralog) ✅                  │
├──────────────────────────────────────────────────┤
│                                                  │
│  Each partition uses different copysets:         │
│    Partition 0: [Node 1, Node 2, Node 4]        │
│    Partition 1: [Node 2, Node 3, Node 5]        │
│    Partition 2: [Node 1, Node 3, Node 6]        │
│    Partition 3: [Node 4, Node 5, Node 6]        │
│                                                  │
│  Result: Load spread across entire cluster! ✅   │
└──────────────────────────────────────────────────┘
```

### How These Solutions Complement Each Other

The three solutions work together to eliminate bottlenecks at different levels:

```
┌─────────────────────────────────────────────────────┐
│   Problem Solved by Each Solution                   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Solution 1: Partitioning                           │
│    Distributes WRITE LEADERSHIP                     │
│    ├─ Each partition has one leader                │
│    ├─ Leaders distributed across nodes             │
│    └─ Avoids single leader bottleneck              │
│                                                     │
│  Solution 2: Read Replicas                          │
│    Distributes READ LOAD                            │
│    ├─ Clients can read from any replica            │
│    ├─ Multiplies read capacity by RF                │
│    └─ Avoids read bottleneck                       │
│                                                     │
│  Solution 3: CopySet                                │
│    Distributes REPLICATION LOAD                     │
│    ├─ Each partition uses different replicas       │
│    ├─ Spreads follower traffic across cluster      │
│    └─ Avoids always hitting same followers         │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Why Partitioning + CopySet is Powerful:**

Without CopySet (partitioning only):
```
Cluster: 6 nodes, 12 partitions, RF=3

Leadership distribution (good):
  Node 1 leads: Partitions [0, 6]
  Node 2 leads: Partitions [1, 7]
  Node 3 leads: Partitions [2, 8]
  Node 4 leads: Partitions [3, 9]
  Node 5 leads: Partitions [4, 10]
  Node 6 leads: Partitions [5, 11]
  ✅ Write load distributed evenly!

But replica placement (bottleneck):
  All partitions: Replicas=[N1, N2, N3]
  
  ❌ Problem: Nodes 1,2,3 get ALL replication traffic!
  ❌ Nodes 4,5,6 underutilized as followers
  ❌ Nodes 1,2,3 become bottleneck despite distributed leadership
```

With CopySet (partitioning + copyset):
```
Cluster: 6 nodes, 12 partitions, RF=3

Leadership distribution:
  Node 1 leads: Partitions [0, 6]
  Node 2 leads: Partitions [1, 7]
  Node 3 leads: Partitions [2, 8]
  Node 4 leads: Partitions [3, 9]
  Node 5 leads: Partitions [4, 10]
  Node 6 leads: Partitions [5, 11]
  ✅ Write load distributed!

Replica placement (with CopySet):
  Partition 0: Leader=N1, Replicas=[N1, N2, N4]
  Partition 1: Leader=N2, Replicas=[N2, N3, N5]
  Partition 2: Leader=N3, Replicas=[N3, N1, N6]
  Partition 3: Leader=N4, Replicas=[N4, N5, N1]
  Partition 4: Leader=N5, Replicas=[N5, N6, N2]
  Partition 5: Leader=N6, Replicas=[N6, N1, N3]
  Partition 6: Leader=N1, Replicas=[N1, N3, N5]
  Partition 7: Leader=N2, Replicas=[N2, N4, N6]
  Partition 8: Leader=N3, Replicas=[N3, N5, N1]
  Partition 9: Leader=N4, Replicas=[N4, N6, N2]
  Partition 10: Leader=N5, Replicas=[N5, N1, N3]
  Partition 11: Leader=N6, Replicas=[N6, N2, N4]
  
  ✅ Replication load distributed across ALL nodes!
  ✅ No node is overloaded
  ✅ Maximum cluster utilization
```

**The Combined Effect:**

```
┌─────────────────────────────────────────────────────┐
│   Partitioning ONLY                                 │
├─────────────────────────────────────────────────────┤
│  Write throughput:     ✅ High (distributed)        │
│  Replication capacity: ❌ Limited (same followers)  │
│  Cluster utilization:  ⚠️  50-70% (uneven)         │
│                                                     │
│  Bottleneck: Follower nodes overwhelmed             │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│   Partitioning + CopySet                            │
├─────────────────────────────────────────────────────┤
│  Write throughput:     ✅ High (distributed)        │
│  Replication capacity: ✅ High (distributed)        │
│  Cluster utilization:  ✅ 90%+ (even)               │
│                                                     │
│  No bottleneck: All nodes participate equally       │
└─────────────────────────────────────────────────────┘
```

**Load Distribution Comparison:**

```
Traditional (no CopySet):
─────────────────────────────────────────
Node 1: Leader for 2 partitions + Follower for 12 = OVERLOADED ⚠️
Node 2: Leader for 2 partitions + Follower for 12 = OVERLOADED ⚠️
Node 3: Leader for 2 partitions + Follower for 12 = OVERLOADED ⚠️
Node 4: Leader for 2 partitions + Follower for 0  = UNDERUTILIZED ⚠️
Node 5: Leader for 2 partitions + Follower for 0  = UNDERUTILIZED ⚠️
Node 6: Leader for 2 partitions + Follower for 0  = UNDERUTILIZED ⚠️

With CopySet:
─────────────────────────────────────────
Node 1: Leader for 2 partitions + Follower for 6  = BALANCED ✅
Node 2: Leader for 2 partitions + Follower for 6  = BALANCED ✅
Node 3: Leader for 2 partitions + Follower for 6  = BALANCED ✅
Node 4: Leader for 2 partitions + Follower for 6  = BALANCED ✅
Node 5: Leader for 2 partitions + Follower for 6  = BALANCED ✅
Node 6: Leader for 2 partitions + Follower for 6  = BALANCED ✅
```

**Real-World Impact:**

```
Scenario: 10 nodes, 100 partitions, RF=3, 1M writes/sec total

Without CopySet:
  Leaders: Evenly distributed (✅ 100K writes/node)
  Followers: Replicas always [N1, N2, N3]
    ❌ N1, N2, N3 each handle 3M writes/sec (3x load!)
    ❌ N4-N10 handle 0 replication (wasted capacity)
  
  Result: Cluster CANNOT sustain 1M writes/sec
          Nodes 1-3 are bottleneck

With CopySet:
  Leaders: Evenly distributed (✅ 100K writes/node)
  Followers: Distributed via CopySet
    ✅ Each node handles ~300K writes/sec as follower
    ✅ All nodes utilized evenly
  
  Result: Cluster EASILY sustains 1M writes/sec
          Can scale to 3M+ writes/sec
```

**Key Insight:**

Partitioning and CopySet are **complementary by design**:

1. **Partitioning** distributes the decision-making (leadership)
2. **CopySet** distributes the work (replication)
3. Together they eliminate **all major bottlenecks**

Without CopySet, partitioning only solves half the problem. With CopySet, you get **true horizontal scalability** where every node contributes equally to both leadership and replication.

This is why **LogDevice invented CopySet replication** - to complement partitioning and achieve maximum cluster utilization! 🚀

### Throughput Scaling Examples

#### Example 1: Small Cluster (3 nodes)

```
Configuration:
  Nodes: 3
  Partitions: 9
  Replication Factor: 3
  Write Quorum: 2

Leadership Distribution:
  Node 1 leads: 3 partitions
  Node 2 leads: 3 partitions
  Node 3 leads: 3 partitions

Write Throughput:
  Per-partition: 100K records/sec
  Total: 9 × 100K = 900K records/sec

Read Throughput (from any replica):
  Total: 900K × 3 = 2.7M records/sec
```

#### Example 2: Large Cluster (10 nodes)

```
Configuration:
  Nodes: 10
  Partitions: 100
  Replication Factor: 3
  Write Quorum: 2

Leadership Distribution:
  Each node leads: ~10 partitions
  Load balanced evenly across cluster

Write Throughput:
  Per-partition: 100K records/sec
  Total: 100 × 100K = 10M records/sec

Read Throughput (from any replica):
  Total: 10M × 3 = 30M records/sec
```

### The Fundamental Trade-off

Pyralog's leader-based architecture is a deliberate choice:

```
┌─────────────────────────────────────────────────────┐
│   LEADER-BASED (Pyralog, Kafka)                        │
├─────────────────────────────────────────────────────┤
│  Advantages:                                        │
│    ✅ Strong consistency per partition              │
│    ✅ Total ordering within partition               │
│    ✅ Simple programming model                      │
│    ✅ Exactly-once semantics possible               │
│    ✅ No write conflicts                            │
│                                                     │
│  Disadvantages:                                     │
│    ❌ Leader bottleneck per partition               │
│    ❌ Write latency includes network RTT            │
│    ❌ Single point of failure (until failover)      │
│                                                     │
│  Scales via: Many partitions with distributed       │
│              leadership across nodes                │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│   LEADERLESS (Cassandra, Riak)                      │
├─────────────────────────────────────────────────────┤
│  Advantages:                                        │
│    ✅ No single bottleneck                          │
│    ✅ Write to any node                             │
│    ✅ Better availability                           │
│    ✅ Simpler failure handling                      │
│                                                     │
│  Disadvantages:                                     │
│    ❌ Eventual consistency only                     │
│    ❌ Complex conflict resolution                   │
│    ❌ No total ordering                             │
│    ❌ Read-repair overhead                          │
│                                                     │
│  Scales via: All nodes equal, hash-based routing    │
└─────────────────────────────────────────────────────┘
```

**Pyralog chooses leader-based because:**
1. Distributed logs require ordering (fundamental requirement)
2. Strong consistency simplifies application logic
3. Kafka compatibility demands leader-based model
4. Scales well via partitioning in practice

### Remaining Bottleneck: Hot Partitions

Even with distributed leadership, a single hot partition can become a bottleneck:

```
Problem: Hot Key
────────────────────────────────────────
All records with key="popular-user-id"
  → Same partition (hash-based routing)
  → Same leader
  → Bottleneck on that leader

Example:
  1M requests/sec for one user
  → All to Partition 7
  → Node 2 (leader) overloaded
  → Other nodes underutilized
```

**Mitigations:**

```rust
// 1. Application-level sharding
let partition = if is_hot_key(key) {
    hash(key + random_suffix) % partition_count
} else {
    hash(key) % partition_count
};

// 2. Dynamic partition splitting (see DYNAMIC_PARTITIONS.md)
if partition_load > threshold {
    split_partition(partition_id);
    // Partition 7 → Partitions 7a, 7b
    // Automatic with dynamic partitions!
}

// 3. Read from replicas for hot reads
config.read_policy = ReadPolicy::AnyReplica;
// Spreads read load across 3 nodes
```

**Solution: Dynamic Partitions**

Pyralog supports **dynamic partition splitting and merging** (similar to TiKV's regions):

```
Static Partitions (original):
──────────────────────────────────────────
10 partitions, fixed at creation
  → Partition 7 gets hot
  → Cannot split without reconfiguration
  → Must over-provision partitions upfront

Dynamic Partitions (NEW):
──────────────────────────────────────────
Start with 5 partitions
  → Partition 3 gets hot (100K writes/sec)
  → Automatic split: P3 → P3a + P3b
  → Each gets 50K writes/sec ✅
  → No manual intervention!

See DYNAMIC_PARTITIONS.md for complete details.
```

**Benefits of dynamic partitions:**
- ✅ Automatic load balancing
- ✅ Start small, scale as needed
- ✅ Hot partition auto-splitting
- ✅ Cold partition auto-merging
- ✅ True elastic scalability

**Configuration:**

```toml
[log.my_events]
partitioning_mode = "dynamic"
initial_partitions = 5

[log.my_events.split_policy]
max_partition_size = 10_000_000_000  # 10GB
max_write_rate = 100_000.0            # 100K/sec
load_imbalance_threshold = 2.0

[log.my_events.merge_policy]
min_partition_size = 1_000_000_000   # 1GB
min_write_rate = 100.0                # 100/sec
```

### Horizontal Scaling

Adding nodes increases capacity linearly:

```
Add Node Process:
1. Join cluster (Raft membership change)
2. Receive partition assignments
3. Fetch data from existing replicas
4. Join ISR when caught up
5. Start serving as leader for assigned partitions
6. Start serving as follower for other partitions

Result:
  N nodes → N+1 nodes
  Leadership distributed across N+1 nodes
  Throughput increases proportionally
```

**Example scaling timeline:**

```
Initial: 3 nodes, 30 partitions
  Each node: 10 partitions (leader)
  Write capacity: 3M records/sec

Add Node 4:
  Rebalance: Each node now leads ~7-8 partitions
  Write capacity: ~3M records/sec (same, but more headroom)

Add more partitions: 3 nodes, 60 partitions
  Each node: 20 partitions (leader)
  Write capacity: 6M records/sec ✅

Add Node 5, 6: 6 nodes, 60 partitions
  Each node: 10 partitions (leader)
  Write capacity: 6M records/sec (more fault tolerance)
```

### Partition Rebalancing

Automatic load balancing when cluster topology changes:

```
Rebalance Triggers:
  - New node added
  - Node removed
  - Uneven load distribution
  - Manual rebalancing requested

Rebalance Process:
  1. Calculate optimal partition assignment
     (minimize movement, balance load)
  
  2. Create new replicas on target nodes
     (fetch data from existing replicas)
  
  3. Wait for new replicas to sync
     (join ISR when caught up)
  
  4. Update metadata
     (new leader/follower assignments)
  
  5. Remove old replicas
     (cleanup previous assignments)

During rebalancing:
  ✅ System remains available
  ✅ No data loss
  ✅ Minimal performance impact
```

### Future Optimizations

**1. Partition Splitting**

```rust
// Automatic partition splitting for hot partitions
if partition_metrics.throughput > threshold {
    // Split partition: hash range 0-65535 → 0-32767 + 32768-65535
    split_partition(partition_id)?;
}
```

**2. Dynamic Leader Rebalancing**

```rust
// Move leadership to less-loaded nodes
if node_load_imbalance > threshold {
    rebalance_leaders()?;
    // Transfer leadership without moving data
}
```

**3. Multi-Leader for Geo-Replication**

```rust
// Each datacenter has a leader (eventual consistency)
config.topology = Topology::MultiDatacenter {
    allow_multi_leader: true,
    conflict_resolution: ConflictResolution::LastWriteWins,
};
```

**4. Read Replicas**

```rust
// Dedicated read-only replicas (don't participate in quorum)
config.replication.read_replicas = 2;
// Increases read capacity without affecting write quorum
```

### Scalability Summary

| Aspect | Strategy | Result |
|--------|----------|--------|
| Write throughput | Partitioning | Linear scaling with partitions |
| Read throughput | Replicas | Linear scaling with RF |
| Storage capacity | Add nodes | Linear scaling with nodes |
| Fault tolerance | Replication | Tolerates RF-W node failures |
| Hot partitions | App sharding, future split | Mitigated |
| Leadership | Distributed via partitions | No single bottleneck |

**Real-world capacity example:**
```
10 nodes × 10 partitions/node × 100K records/sec = 10M records/sec
With RF=3: 30M reads/sec possible
```

## Monitoring and Observability

Key metrics:
- Write latency (p50, p99, p999)
- Read latency (p50, p99, p999)
- Throughput (bytes/sec, records/sec)
- Replication lag
- ISR count
- Disk usage
- Network I/O

## Conclusion

Pyralog's architecture represents a synthesis of the best ideas from modern distributed log systems, designed for extreme performance and scalability.

### Key Architectural Innovations

**1. Dual Raft Clusters** ⭐

Separate consensus domains for massive scalability:
- **Global Raft**: Cluster-wide metadata (membership, partition creation)
- **Per-Partition Raft**: Partition-specific operations (epoch changes, failover)
- **Parallel failover**: 1000 partitions fail over in 10ms (not 10 seconds!)
- **No global bottleneck**: Partition operations don't contend with each other
- **Efficient multiplexing**: 600+ Raft groups per node with batched heartbeats

**2. Epochs (from LogDevice)**

The most impactful optimization:
- **100x throughput improvement** by decoupling offset assignment from consensus
- Per-partition Raft consensus once per epoch (not per record!)
- Local offset increment: millions of records/sec without consensus bottleneck
- Safe failover without split-brain scenarios

**3. Smart Client Pattern (from Kafka)**

Eliminates proxy overhead:
- Direct connection to partition leaders (1 hop vs 2)
- Client-side load balancing via metadata caching
- Metadata refresh only on topology changes (~5 min)
- Amortized overhead: essentially zero

**4. Distributed Leadership via Partitioning**

Spreads write decisions across the cluster:
- Each partition has one leader
- Leadership distributed across all nodes
- Linear scaling: N partitions → N× write throughput
- No single leader bottleneck

**5. CopySet Replication (from LogDevice)**

Critical complement to partitioning:
- Two strategies: Per-partition (simple) or per-record (maximum distribution)
- Per-record: Distributes replication load across entire cluster
- Per-record with coordinator mode: Leader doesn't store data (99%+ less I/O!)
- Achieves 90%+ cluster utilization vs 50% without it
- Every node contributes equally to leadership and replication
- Leader can handle 20x-50x more partitions in coordinator mode

**6. Flexible Quorums**

Runtime configurability:
- Configure CAP position per use case
- Strong consistency (CP), high availability (AP), or balanced
- W+R > RF constraint ensures safety
- No architectural lock-in

**7. Multiple Optimizations Working Together**

```
┌──────────────────────────────────────────────────────────┐
│         The Synergistic Effect                           │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Dual Raft + Partitioning                                │
│    = Parallel failover, no global bottleneck            │
│    → 1000 partitions fail over simultaneously            │
│                                                          │
│  Epochs + Per-Partition Raft                             │
│    = Fast epoch changes (only partition replicas vote)  │
│    → 10ms failover instead of seconds                   │
│                                                          │
│  Epochs + Smart Clients                                  │
│    = Million writes/sec with sub-ms latency             │
│                                                          │
│  Partitioning + CopySet                                  │
│    = True horizontal scalability, no bottlenecks        │
│                                                          │
│  Flexible Quorums + ISR                                  │
│    = Configurable consistency/availability              │
│                                                          │
│  Global Raft + Per-Partition Raft                        │
│    = Strong consistency without throughput penalty      │
│    → Cluster ops separate from partition ops            │
│                                                          │
│  Write Cache + Zero-Copy                                 │
│    = Sub-millisecond latencies                          │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### What Makes This Architecture Special

**Complementary Design:**

Every component enhances the others:
- Dual Raft separates concerns → enables parallel partition operations
- Epochs remove consensus bottleneck → enables million writes/sec  
- Per-partition Raft makes epochs fast → 10ms epoch changes (not seconds)
- Smart clients avoid proxy → enables direct scaling
- Partitioning distributes leadership → enables horizontal scaling
- CopySet distributes replication → prevents follower bottleneck
- Together: **No single point of bottleneck anywhere!**

**Production-Ready Capabilities:**

```
Expected Performance (10 nodes, 100 partitions):
─────────────────────────────────────────────────
Write throughput:  10M+ records/sec
Read throughput:   30M+ records/sec (RF=3)
Write latency:     < 1ms (p99, with cache)
Read latency:      < 0.5ms (p99, with mmap)
Scalability:       Linear with nodes/partitions
Consistency:       Configurable (CP to AP spectrum)
Availability:      Tolerates RF-W node failures
```

**Learning from the Best:**

Pyralog synthesizes innovations from:
- **LogDevice** (Facebook): Epochs, CopySet, flexible quorums, hierarchical storage
- **Kafka** (LinkedIn): Smart clients, partitioning, ISR, log-structured storage
- **Redpanda** (Vectorized): Write caching, zero-copy I/O, thread-per-core
- **Raft** (Stanford): Proven consensus algorithm for cluster coordination

### Architectural Philosophy

**1. Optimize the Hot Path**

- Write path: Epochs avoid Raft, cache avoids fsync, smart client avoids proxy
- Read path: mmap for zero-copy, ISR for flexibility, metadata for direct routing
- Result: Sub-millisecond latencies at million ops/sec

**2. Eliminate Bottlenecks at Every Level**

- Global consensus for everything → Dual Raft (separate domains)
- Single leader → Distributed leadership (partitioning)
- Follower overload → Distributed replication (CopySet)
- Proxy overhead → Smart clients (direct routing)
- Consensus per record → Consensus per epoch (100x gain)
- Sequential partition failover → Parallel per-partition Raft (1000x faster)

**3. Make Trade-offs Configurable**

- CAP spectrum: Choose consistency vs availability at runtime
- Read policy: Leader, replicas, quorum, or nearest
- Quorum sizes: Balance durability vs latency
- No one-size-fits-all: You decide the trade-offs

**4. Horizontal Scalability**

- Add nodes → Add capacity (linear scaling)
- Add partitions → Add throughput (linear scaling)
- Replication → Fault tolerance (configurable)
- Result: Start small, scale to billions of records/day

### The Big Picture

```
┌─────────────────────────────────────────────────────┐
│   Why Pyralog's Architecture Succeeds                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│  Traditional Distributed Log:                       │
│    ❌ Leader bottleneck                             │
│    ❌ Consensus per record                          │
│    ❌ Proxy overhead                                │
│    ❌ Follower bottleneck                           │
│    ❌ Fixed consistency model                       │
│                                                     │
│  Pyralog's Solution:                                   │
│    ✅ Distributed leadership (partitioning)         │
│    ✅ Consensus per epoch (100x faster)             │
│    ✅ Smart clients (direct routing)                │
│    ✅ Distributed replication (CopySet)             │
│    ✅ Flexible quorums (configurable)               │
│                                                     │
│  Result: 10M+ writes/sec, sub-ms latency,          │
│          horizontal scaling, no bottlenecks         │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Final Thoughts

Pyralog isn't just another distributed log - it's a **synthesis of proven innovations** that complement each other perfectly:

1. **Epochs** make high throughput possible (remove consensus bottleneck)
2. **Smart clients** make it scalable (remove proxy bottleneck)
3. **Partitioning** makes it distributed (remove leader bottleneck)
4. **CopySet** makes it efficient (remove follower bottleneck)
5. **Flexible quorums** make it adaptable (configure for your needs)

Each innovation solves a specific bottleneck. Together, they create a system with **no fundamental limitations** - just add more nodes and partitions to scale.

**This is the power of learning from a decade of production distributed log systems and combining their best ideas in a modern, Rust-based implementation.**

The modular design allows for easy extension and customization while maintaining strong guarantees about data durability and consistency. Whether you need strong consistency for financial transactions or high availability for analytics, Pyralog's architecture can be configured to meet your requirements.

**Welcome to the next generation of distributed logs.** 🚀

