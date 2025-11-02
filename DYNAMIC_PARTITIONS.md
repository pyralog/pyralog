# Dynamic Partitions in Pyralog

Design document for adding dynamic partition splitting and merging to Pyralog.

## Table of Contents

1. [Overview](#overview)
2. [Current Architecture: Static Partitions](#current-architecture-static-partitions)
3. [New Architecture: Dynamic Partitions](#new-architecture-dynamic-partitions)
4. [Partition Splitting](#partition-splitting)
5. [Partition Merging](#partition-merging)
6. [Implementation Details](#implementation-details)
7. [Migration Strategy](#migration-strategy)
8. [Performance Considerations](#performance-considerations)
9. [Comparison with TiKV](#comparison-with-tikv)

---

## Overview

### Motivation

**Current limitation with static partitions:**

```
Problem: Hot partition
──────────────────────────────────────────
Log "events" with 10 partitions:
  Partition 0: 1K writes/sec  ✅
  Partition 1: 1K writes/sec  ✅
  Partition 2: 1K writes/sec  ✅
  Partition 3: 100K writes/sec ❌ (hot!)
  Partition 4: 1K writes/sec  ✅
  ...

Issue:
  - Partition 3 is bottlenecked by single leader
  - Can't add more partitions without reconfiguration
  - Other partitions are underutilized
  - Manual rebalancing required
```

**Solution: Dynamic partitions**

```
Automatic splitting:
──────────────────────────────────────────
Partition 3 (hot) → Split into:
  Partition 3a: 50K writes/sec ✅
  Partition 3b: 50K writes/sec ✅

Result:
  - Load distributed across 2 leaders
  - Automatic rebalancing
  - No manual intervention
  - Linear scalability
```

### Goals

1. **Automatic scaling**: Split hot partitions automatically
2. **Efficiency**: Merge cold partitions to reduce overhead
3. **Transparency**: No client-side changes required
4. **Zero downtime**: Split/merge without interruption
5. **Compatibility**: Support both static and dynamic modes

---

## Current Architecture: Static Partitions

### How It Works

```rust
pub struct LogMetadata {
    log_id: LogId,
    partition_count: u32,  // Fixed at creation!
    partitions: Vec<PartitionMetadata>,
}

pub struct PartitionMetadata {
    partition_id: u32,      // 0..partition_count-1
    leader: NodeId,
    replicas: Vec<NodeId>,
    // No key range - determined by hash(key) % partition_count
}
```

**Partitioning logic:**

```rust
// Client-side partitioning
fn select_partition(key: &[u8], partition_count: u32) -> u32 {
    hash(key) % partition_count
}

// Once partition_count is set, it never changes!
```

### Limitations

```
┌─────────────────────────────────────────────────────────┐
│   Static Partitions: Limitations                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. Hot Partition Problem                               │
│     → One partition gets 90% of traffic                 │
│     → Single leader bottleneck                          │
│     → Can't split without full reconfiguration          │
│                                                         │
│  2. Over-provisioning Required                          │
│     → Must create 100s of partitions upfront            │
│     → Overhead for small logs                           │
│     → Wasted resources                                  │
│                                                         │
│  3. No Automatic Rebalancing                            │
│     → Growth requires manual intervention               │
│     → Risk of uneven load                               │
│                                                         │
│  4. Fixed Capacity                                      │
│     → Each partition has throughput limit               │
│     → Can't exceed without adding partitions            │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## New Architecture: Dynamic Partitions

### Core Concept

Instead of fixed partition count, use **key ranges**:

```rust
pub struct DynamicPartitionMetadata {
    partition_id: PartitionId,
    key_range: KeyRange,        // NEW: [start, end)
    leader: NodeId,
    replicas: Vec<NodeId>,
    state: PartitionState,      // NEW: Normal, Splitting, Merging
    
    // Splitting metadata
    split_point: Option<Vec<u8>>,
    child_partitions: Option<(PartitionId, PartitionId)>,
    
    // Metrics for split/merge decisions
    size_bytes: u64,
    write_rate: f64,  // writes/sec
    read_rate: f64,   // reads/sec
}

pub struct KeyRange {
    start: Vec<u8>,  // Inclusive
    end: Vec<u8>,    // Exclusive
}

pub enum PartitionState {
    Normal,
    Splitting { progress: f64 },
    Merging { with: PartitionId, progress: f64 },
    Offline,
}
```

**Key routing with ranges:**

```rust
// Find partition by key
fn find_partition(key: &[u8], metadata: &LogMetadata) -> PartitionId {
    // Binary search through sorted key ranges
    metadata.partitions
        .binary_search_by(|p| {
            if key < &p.key_range.start {
                Ordering::Greater
            } else if key >= &p.key_range.end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .map(|idx| metadata.partitions[idx].partition_id)
        .expect("Key must fall in some partition")
}
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│   Dynamic Partitions: Key Space Division                │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Initial state:                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Partition 1: [MIN, MAX)                          │  │
│  │ Size: 100GB, Rate: 1K writes/sec                 │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
│  After growth (auto-split at threshold):                │
│  ┌───────────────────────┐┌──────────────────────────┐ │
│  │ Partition 1:          ││ Partition 2:             │ │
│  │ [MIN, "key_m")        ││ ["key_m", MAX)           │ │
│  │ Size: 50GB            ││ Size: 50GB               │ │
│  │ Rate: 500 writes/sec  ││ Rate: 500 writes/sec     │ │
│  └───────────────────────┘└──────────────────────────┘ │
│                                                         │
│  After more growth:                                     │
│  ┌──────┐┌───────┐┌────────┐┌──────────────────────┐  │
│  │ P1:  ││ P2:   ││ P3:    ││ P4:                  │  │
│  │ [MIN,││[key_e,││[key_m, ││[key_s, MAX)          │  │
│  │key_e)││key_m) ││key_s)  ││                      │  │
│  └──────┘└───────┘└────────┘└──────────────────────┘  │
│                                                         │
│  Automatic splitting creates balanced load!             │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Benefits

```
✅ Automatic load balancing
✅ Start small, grow as needed
✅ Hot partitions split automatically
✅ Cold partitions merge to reduce overhead
✅ No over-provisioning required
✅ True elastic scalability
```

---

## Partition Splitting

### When to Split

**Split triggers:**

```rust
pub struct SplitPolicy {
    // Size-based splitting
    max_partition_size: u64,  // Default: 10GB
    
    // Load-based splitting
    max_write_rate: f64,      // Default: 100K writes/sec
    max_read_rate: f64,       // Default: 500K reads/sec
    
    // Heuristics
    load_imbalance_threshold: f64,  // Default: 2.0x average
    min_partition_lifetime: Duration,  // Default: 1 hour
}

impl SplitDecider {
    pub fn should_split(&self, partition: &PartitionMetadata) -> bool {
        // Size check
        if partition.size_bytes > self.policy.max_partition_size {
            return true;
        }
        
        // Load check
        if partition.write_rate > self.policy.max_write_rate {
            return true;
        }
        
        if partition.read_rate > self.policy.max_read_rate {
            return true;
        }
        
        // Imbalance check
        let avg_load = self.cluster_average_load();
        if partition.write_rate > avg_load * self.policy.load_imbalance_threshold {
            return true;
        }
        
        false
    }
}
```

### Split Process

**Step-by-step:**

```
Phase 1: Decide to Split
──────────────────────────────────────────
1. Scheduler detects hot partition
   → Partition 3: 100K writes/sec (threshold: 50K)
   
2. Propose split to Global Raft
   → Split Partition 3 at key "key_m"
   → Create Partition 3a: [MIN, "key_m")
   → Create Partition 3b: ["key_m", MAX)
   
3. Global Raft commits split operation
   → All nodes see split decision

Phase 2: Prepare Split
──────────────────────────────────────────
4. Partition 3 leader pauses writes (brief)
   → Set state = Splitting
   → Determine split point (median key)
   
5. Create child partition Raft groups
   → Partition 3a Raft: [N1, N2, N3]
   → Partition 3b Raft: [N2, N3, N4]
   
6. Allocate child partition metadata
   → Store in Global Raft
   → Broadcast to all nodes

Phase 3: Data Migration
──────────────────────────────────────────
7. Copy data to child partitions (background)
   → Records < "key_m" → Partition 3a
   → Records >= "key_m" → Partition 3b
   → Continue accepting writes to parent
   
8. Tail parent partition
   → New writes route to children
   → Parent partition becomes read-only

Phase 4: Cutover
──────────────────────────────────────────
9. Seal parent partition epoch
   → No more writes allowed
   
10. Finalize children
    → Partition 3a ready (epoch 1)
    → Partition 3b ready (epoch 1)
    
11. Update metadata
    → Remove Partition 3
    → Add Partition 3a, 3b
    → Global Raft commits
    
12. Resume operations
    → Clients fetch new metadata
    → Route to 3a or 3b based on key
    
Total time: ~30 seconds for 10GB partition
```

### Split Point Selection

```rust
pub trait SplitPointSelector {
    fn select_split_point(&self, partition: &Partition) -> Result<Vec<u8>>;
}

// Strategy 1: Median key (balanced size)
pub struct MedianSplitSelector;

impl SplitPointSelector for MedianSplitSelector {
    fn select_split_point(&self, partition: &Partition) -> Result<Vec<u8>> {
        // Scan partition, find median key
        let keys = partition.sample_keys(1000)?;
        keys.sort();
        Ok(keys[keys.len() / 2].clone())
    }
}

// Strategy 2: Load-based split (balanced traffic)
pub struct LoadBasedSplitSelector;

impl SplitPointSelector for LoadBasedSplitSelector {
    fn select_split_point(&self, partition: &Partition) -> Result<Vec<u8>> {
        // Analyze write patterns
        let hot_keys = partition.analyze_write_patterns()?;
        
        // Find key that divides traffic 50/50
        let split_key = hot_keys.find_balanced_split_point();
        Ok(split_key)
    }
}

// Strategy 3: Hash-based split (deterministic)
pub struct HashBasedSplitSelector;

impl SplitPointSelector for HashBasedSplitSelector {
    fn select_split_point(&self, partition: &Partition) -> Result<Vec<u8>> {
        // Split at hash midpoint
        let start = partition.key_range.start;
        let end = partition.key_range.end;
        
        // Calculate midpoint in hash space
        let midpoint = calculate_hash_midpoint(&start, &end);
        Ok(midpoint)
    }
}
```

### Handling Writes During Split

**Zero-downtime split:**

```rust
pub struct SplittingPartition {
    parent: Arc<Partition>,
    split_point: Vec<u8>,
    child_a: Arc<Partition>,  // [start, split)
    child_b: Arc<Partition>,  // [split, end)
    state: Arc<RwLock<SplitState>>,
}

impl SplittingPartition {
    pub async fn append(&self, record: Record) -> Result<LogOffset> {
        let state = self.state.read();
        
        match *state {
            SplitState::Copying => {
                // Still copying, write to parent
                self.parent.append(record).await
            }
            
            SplitState::Routing => {
                // Route to appropriate child
                if record.key.as_ref().unwrap() < &self.split_point {
                    self.child_a.append(record).await
                } else {
                    self.child_b.append(record).await
                }
            }
            
            SplitState::Complete => {
                // Split done, shouldn't reach here
                Err(PyralogError::PartitionSplit)
            }
        }
    }
}

pub enum SplitState {
    Copying,     // Background copy in progress
    Routing,     // New writes go to children
    Complete,    // Split finished
}
```

---

## Partition Merging

### When to Merge

**Merge triggers:**

```rust
pub struct MergePolicy {
    min_partition_size: u64,        // Default: 1GB
    min_write_rate: f64,            // Default: 100 writes/sec
    min_partition_count: u32,       // Default: 3 (never merge below)
    idle_duration: Duration,        // Default: 24 hours
}

impl MergeDecider {
    pub fn should_merge(&self, partition: &PartitionMetadata) -> Option<PartitionId> {
        // Too small?
        if partition.size_bytes < self.policy.min_partition_size {
            // Find adjacent partition to merge with
            return self.find_merge_candidate(partition);
        }
        
        // Too cold?
        if partition.write_rate < self.policy.min_write_rate &&
           partition.idle_since > self.policy.idle_duration {
            return self.find_merge_candidate(partition);
        }
        
        None
    }
    
    fn find_merge_candidate(&self, partition: &PartitionMetadata) -> Option<PartitionId> {
        // Find adjacent partition with similar load
        self.find_adjacent_partitions(partition)
            .into_iter()
            .filter(|adj| adj.size_bytes + partition.size_bytes < self.policy.max_merged_size)
            .min_by_key(|adj| adj.load_difference(partition))
            .map(|adj| adj.partition_id)
    }
}
```

### Merge Process

```
Phase 1: Decide to Merge
──────────────────────────────────────────
1. Scheduler detects cold partitions
   → Partition 5: 500MB, 50 writes/sec
   → Partition 6: 600MB, 30 writes/sec
   → Adjacent: [key_p, key_q), [key_q, key_r)
   
2. Propose merge to Global Raft
   → Merge Partition 5 + 6 → Partition 7
   → New range: [key_p, key_r)
   
3. Global Raft commits merge

Phase 2: Prepare Merge
──────────────────────────────────────────
4. Create new merged partition
   → Partition 7: [key_p, key_r)
   → Raft group: [N1, N2, N3]
   
5. Set parent partitions to merging state
   → Partition 5: state = Merging(into=7)
   → Partition 6: state = Merging(into=7)

Phase 3: Data Migration
──────────────────────────────────────────
6. Stream data from parents to child
   → Partition 5 data → Partition 7
   → Partition 6 data → Partition 7
   → Maintain order by offset
   
7. Tail parent partitions
   → New writes go to merged partition 7

Phase 4: Cutover
──────────────────────────────────────────
8. Seal parent partitions
   → Partition 5 epoch sealed
   → Partition 6 epoch sealed
   
9. Finalize merged partition
   → Partition 7 ready
   
10. Update metadata
    → Remove Partition 5, 6
    → Add Partition 7
    → Global Raft commits
    
11. Cleanup
    → Delete parent partition data (after retention)
    → Clients fetch new metadata
```

---

## Implementation Details

### Metadata Management

**Global Raft operations:**

```rust
pub enum MetadataOperation {
    CreatePartition {
        partition_id: PartitionId,
        key_range: KeyRange,
        replicas: Vec<NodeId>,
    },
    
    SplitPartition {
        parent_id: PartitionId,
        split_point: Vec<u8>,
        child_a_id: PartitionId,
        child_b_id: PartitionId,
    },
    
    MergePartitions {
        parent_a_id: PartitionId,
        parent_b_id: PartitionId,
        merged_id: PartitionId,
    },
    
    DeletePartition {
        partition_id: PartitionId,
    },
}

// Propose to Global Raft
impl ClusterManager {
    pub async fn split_partition(
        &self,
        partition_id: PartitionId,
        split_point: Vec<u8>,
    ) -> Result<(PartitionId, PartitionId)> {
        let child_a_id = self.allocate_partition_id();
        let child_b_id = self.allocate_partition_id();
        
        let op = MetadataOperation::SplitPartition {
            parent_id: partition_id,
            split_point,
            child_a_id,
            child_b_id,
        };
        
        // Propose to Global Raft
        self.global_raft.propose(serialize(&op)?).await?;
        
        // Wait for commit
        self.wait_for_metadata_update(child_a_id).await?;
        
        Ok((child_a_id, child_b_id))
    }
}
```

### Client-Side Routing

**Smart client with dynamic routing:**

```rust
pub struct DynamicRoutingClient {
    metadata_cache: Arc<RwLock<MetadataCache>>,
    connections: Arc<RwLock<HashMap<NodeId, Connection>>>,
}

impl DynamicRoutingClient {
    pub async fn produce(&self, log_id: LogId, record: Record) -> Result<LogOffset> {
        let key = record.key.as_ref().ok_or(PyralogError::MissingKey)?;
        
        loop {
            // 1. Find partition by key range
            let metadata = self.metadata_cache.read();
            let partition = metadata.find_partition_by_key(log_id, key)?;
            
            // 2. Send to partition leader
            match self.send_to_partition(partition, record.clone()).await {
                Ok(offset) => return Ok(offset),
                
                Err(PyralogError::PartitionSplitting { new_partitions }) => {
                    // Partition is splitting, refresh metadata
                    self.refresh_metadata(log_id).await?;
                    continue;
                }
                
                Err(PyralogError::WrongPartition { correct_partition }) => {
                    // Stale metadata, update cache
                    self.update_partition_cache(correct_partition).await?;
                    continue;
                }
                
                Err(e) => return Err(e),
            }
        }
    }
}

pub struct MetadataCache {
    logs: HashMap<LogId, LogMetadata>,
    last_refresh: HashMap<LogId, Instant>,
}

impl MetadataCache {
    pub fn find_partition_by_key(&self, log_id: LogId, key: &[u8]) -> Result<&PartitionMetadata> {
        let log = self.logs.get(&log_id).ok_or(PyralogError::LogNotFound)?;
        
        // Binary search through sorted partitions
        log.partitions
            .binary_search_by(|p| {
                if key < &p.key_range.start {
                    Ordering::Greater
                } else if key >= &p.key_range.end {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .map(|idx| &log.partitions[idx])
            .map_err(|_| PyralogError::NoPartitionForKey)
    }
}
```

### Scheduler Component

**Automatic split/merge scheduling:**

```rust
pub struct PartitionScheduler {
    cluster: Arc<ClusterManager>,
    split_policy: SplitPolicy,
    merge_policy: MergePolicy,
    interval: Duration,
}

impl PartitionScheduler {
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.interval);
        
        loop {
            interval.tick().await;
            
            // 1. Collect partition metrics
            let partitions = self.cluster.get_all_partitions().await;
            let metrics = self.collect_metrics(&partitions).await;
            
            // 2. Decide splits
            for partition in &partitions {
                if self.should_split(partition, &metrics) {
                    self.schedule_split(partition).await;
                }
            }
            
            // 3. Decide merges
            for partition in &partitions {
                if let Some(merge_with) = self.should_merge(partition, &metrics) {
                    self.schedule_merge(partition, merge_with).await;
                }
            }
        }
    }
    
    async fn schedule_split(&self, partition: &PartitionMetadata) {
        info!("Scheduling split for partition {}", partition.partition_id);
        
        // Select split point
        let split_point = self.select_split_point(partition).await.unwrap();
        
        // Propose split to Global Raft
        match self.cluster.split_partition(partition.partition_id, split_point).await {
            Ok((child_a, child_b)) => {
                info!("Split partition {} into {} and {}", 
                      partition.partition_id, child_a, child_b);
            }
            Err(e) => {
                warn!("Failed to split partition {}: {}", partition.partition_id, e);
            }
        }
    }
}
```

---

## Migration Strategy

### Backwards Compatibility

**Support both modes:**

```rust
pub enum PartitioningMode {
    Static {
        partition_count: u32,
    },
    Dynamic {
        initial_partitions: u32,
        split_policy: SplitPolicy,
        merge_policy: MergePolicy,
    },
}

pub struct LogConfig {
    log_id: LogId,
    partitioning_mode: PartitioningMode,
    // ... other config
}
```

**Configuration:**

```toml
[log.my_events]
# Old way: static partitions
partitioning_mode = "static"
partition_count = 10

[log.my_dynamic_log]
# New way: dynamic partitions
partitioning_mode = "dynamic"
initial_partitions = 3

[log.my_dynamic_log.split_policy]
max_partition_size = 10_000_000_000  # 10GB
max_write_rate = 100_000.0            # 100K/sec
load_imbalance_threshold = 2.0

[log.my_dynamic_log.merge_policy]
min_partition_size = 1_000_000_000   # 1GB
min_write_rate = 100.0                # 100/sec
```

### Migration Path

**Existing logs → Dynamic partitions:**

```
Step 1: Upgrade cluster to version with dynamic partition support
  → Deploy new binaries
  → Old logs continue with static partitions
  
Step 2: Enable dynamic mode for specific logs
  → Update log config: partitioning_mode = "dynamic"
  → Trigger conversion process
  
Step 3: Conversion process
  → Create dynamic partitions with same key ranges as static
  → Example: 10 static partitions → 10 dynamic partitions
    Static P0 (hash % 10 == 0) → Dynamic P0 [0x00, 0x19...]
    Static P1 (hash % 10 == 1) → Dynamic P1 [0x19.., 0x33...]
    ...
  
Step 4: Cut over
  → Update metadata: mark log as dynamic
  → Clients fetch new metadata
  → Start using key-range routing
  
Step 5: Optimize
  → Scheduler starts monitoring load
  → Hot partitions split automatically
  → Cold partitions merge as needed
```

---

## Performance Considerations

### Overhead of Dynamic Partitions

```
┌──────────────────────────────────────────────────────────┐
│   Performance Comparison                                  │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Static Partitions:                                       │
│    Routing: O(1) - hash(key) % N                         │
│    Metadata: O(N) - N partition entries                  │
│    Lookup: Single hash computation                       │
│                                                          │
│  Dynamic Partitions:                                      │
│    Routing: O(log N) - binary search                     │
│    Metadata: O(N) - N partition entries                  │
│    Lookup: Binary search through ranges                  │
│                                                          │
│  Overhead: Negligible                                     │
│    ~10 ns vs ~100 ns (10x slower but still < 1 µs)      │
│    Amortized by network latency (~1 ms)                  │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Split/Merge Performance

```
Partition Split (10GB partition):
──────────────────────────────────────────
Preparation: ~100ms
  - Raft consensus on split decision
  - Allocate child partition IDs
  - Create Raft groups

Data migration: ~20-30 seconds
  - Background copy of 10GB data
  - Parallel writes to children
  - Continues serving reads

Cutover: ~10ms
  - Seal parent epoch
  - Update metadata
  - Activate children

Total disruption: ~10ms (just cutover)
Total time: ~30 seconds

Partition Merge (2 × 1GB partitions):
──────────────────────────────────────────
Similar timeline, faster due to smaller size
Total time: ~5 seconds
Disruption: ~10ms
```

### Scalability

```
Number of Partitions:
──────────────────────────────────────────
Static mode:
  - Fixed at creation (e.g., 100 partitions)
  - Per-node: 100 partitions = 100 Raft groups
  - Overhead: 600 MB (100 × 6 MB)

Dynamic mode:
  - Start with 10 partitions
  - Grow to 1000 partitions as needed
  - Per-node: ~600 partitions average
  - Overhead: ~3.6 GB (600 × 6 MB)
  
  Benefit: Pay only for what you need!
```

---

## Comparison with TiKV

### Similarities

Both Pyralog and TiKV use dynamic sharding:

```
┌─────────────────────────────────────────────────────────┐
│   Pyralog Dynamic Partitions vs TiKV Regions               │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Concept:                                               │
│    Pyralog: Partitions with key ranges                    │
│    TiKV: Regions with key ranges                       │
│    Both: Split hot shards, merge cold ones              │
│                                                         │
│  Metadata:                                              │
│    Pyralog: Global Raft cluster                           │
│    TiKV: PD (Placement Driver)                         │
│    Both: Centralized metadata management                │
│                                                         │
│  Split triggers:                                        │
│    Pyralog: Size, write rate, load imbalance              │
│    TiKV: Size (96MB default), load (optional)          │
│    Both: Automatic detection and execution              │
│                                                         │
│  Per-shard Raft:                                        │
│    Pyralog: Per-partition Raft group                      │
│    TiKV: Per-region Raft group                         │
│    Both: Independent consensus domains                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Differences

```
┌─────────────────────────────────────────────────────────┐
│   Key Differences                                        │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Data model:                                            │
│    Pyralog: Append-only log (records have offsets)        │
│    TiKV: Mutable key-value (MVCC timestamps)           │
│                                                         │
│  Split granularity:                                     │
│    Pyralog: Split point selected from log keys            │
│    TiKV: Split at any key in range                     │
│                                                         │
│  Migration:                                             │
│    Pyralog: Copy log segments to children                 │
│    TiKV: Move RocksDB SSTables                         │
│                                                         │
│  Default threshold:                                     │
│    Pyralog: 10GB or 100K writes/sec                       │
│    TiKV: 96MB (much smaller, more aggressive)          │
│                                                         │
│  Order preservation:                                    │
│    Pyralog: Must maintain offset order in children        │
│    TiKV: No ordering constraint                        │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Why Pyralog's Approach is Different

```
Log-specific considerations:
──────────────────────────────────────────
1. Offset continuity
   → Clients expect sequential offsets
   → Split must maintain offset ordering
   → TiKV doesn't have this constraint

2. Append-only workload
   → Pyralog splits are less frequent (write-once)
   → TiKV regions split more often (updates)

3. Time-series nature
   → Older data is rarely accessed
   → Enables aggressive merging of cold partitions
   → TiKV has uniform access patterns

4. Consumer groups
   → Must handle partition assignment changes
   → Rebalancing on split/merge
   → TiKV doesn't have consumers
```

---

## Configuration Reference

```toml
# Example: Dynamic partition configuration

[log.events]
partitioning_mode = "dynamic"
initial_partitions = 5

[log.events.split_policy]
# Size-based triggers
max_partition_size = 10_000_000_000        # 10GB
max_segment_count = 100

# Load-based triggers
max_write_rate = 100_000.0                 # 100K writes/sec
max_read_rate = 500_000.0                  # 500K reads/sec
load_imbalance_threshold = 2.0             # 2x cluster average

# Safety limits
min_partition_lifetime = "1h"              # Don't split too soon
max_split_frequency = "1/hour"             # Rate limit

[log.events.merge_policy]
# Merge triggers
min_partition_size = 1_000_000_000         # 1GB
min_write_rate = 100.0                     # 100 writes/sec
idle_duration = "24h"                      # 24 hours idle

# Safety limits
min_partition_count = 3                    # Never go below 3
max_merge_frequency = "1/day"

[log.events.scheduler]
check_interval = "60s"                     # Check every minute
enable_auto_split = true
enable_auto_merge = true
```

---

## Implementation Phases

### Phase 1: Foundation (4 weeks)

```
Week 1-2: Data structures and metadata
  - Add KeyRange to PartitionMetadata
  - Update Global Raft to handle split/merge ops
  - Client-side binary search routing
  
Week 3-4: Basic split implementation
  - Split point selection algorithms
  - Data migration logic
  - Epoch sealing during split
  
Deliverable: Manual partition splits working
```

### Phase 2: Automation (4 weeks)

```
Week 1-2: Metrics collection
  - Partition size tracking
  - Write/read rate monitoring
  - Load distribution analysis
  
Week 3-4: Scheduler component
  - Split policy evaluation
  - Automatic split triggering
  - Testing under various loads
  
Deliverable: Automatic splits working
```

### Phase 3: Merging (3 weeks)

```
Week 1-2: Merge implementation
  - Merge candidate selection
  - Data consolidation
  - Adjacent partition handling
  
Week 3: Testing and optimization
  - Merge policy tuning
  - Edge case handling
  
Deliverable: Automatic merges working
```

### Phase 4: Production Hardening (4 weeks)

```
Week 1: Split/merge observability
  - Metrics and dashboards
  - Progress tracking
  - Failure detection
  
Week 2: Error handling
  - Retry logic
  - Rollback on failure
  - Partial split cleanup
  
Week 3: Performance optimization
  - Parallel data migration
  - Incremental copying
  - Background prioritization
  
Week 4: Documentation and migration tools
  - User guide
  - Static → Dynamic migration tool
  - Best practices
  
Deliverable: Production-ready dynamic partitions
```

---

## Summary

### Benefits of Dynamic Partitions

```
✅ Automatic load balancing (no manual intervention)
✅ Start small, scale as needed (efficient resource usage)
✅ Hot partition handling (automatic splitting)
✅ Cost optimization (merge idle partitions)
✅ True elastic scalability (grow and shrink)
✅ Better multi-tenancy (fair resource allocation)
```

### Trade-offs

```
⚠️  More complex implementation (vs static partitions)
⚠️  Metadata overhead (tracking key ranges)
⚠️  Client complexity (binary search routing)
⚠️  Split/merge operations (temporary disruption)
⚠️  Testing complexity (more edge cases)
```

### Recommendation

**Use dynamic partitions when:**
- Workload is unpredictable
- Multi-tenant deployments
- Cost optimization is important
- Hot partition problems occur

**Use static partitions when:**
- Workload is predictable
- Simple deployment preferred
- Over-provisioning is acceptable
- Kafka compatibility required

---

**Dynamic partitions make Pyralog truly elastic, combining the best of Kafka's simplicity with TiKV's scalability!** 🚀

