# Epochs in Pyralog

## Overview

Pyralog now implements **epochs** inspired by LogDevice's sequencer epochs. This document explains what epochs are, why they're important, and how Pyralog uses them.

## What are Epochs?

An **epoch** is a monotonically increasing number that identifies which sequencer (leader) wrote a set of records. When a sequencer fails and a new one takes over, it gets a new, higher epoch number.

### Key Properties

1. **Monotonically Increasing**: Epoch numbers always increase
2. **Generation Tracking**: Each epoch represents a "generation" of writes
3. **Failover Safety**: Prevents ambiguity during leader changes
4. **Recovery Aid**: Helps identify which records to keep during recovery

## Why Epochs Matter

### Problem: Leader Failover Ambiguity

Without epochs:
```
Timeline:
T0: Leader A writes records at offsets 0-999
T1: Leader A fails
T2: Leader B becomes leader
T3: Leader B writes records at offset 1000+
T4: Leader A comes back online (network partition healed)
T5: Leader A tries to write at offset 1000 (conflict!)
```

With epochs:
```
Timeline:
T0: Leader A (epoch 1) writes records at offsets 0-999
T1: Leader A fails, epoch 1 sealed
T2: Leader B becomes leader with epoch 2
T3: Leader B (epoch 2) writes records at offset 1000+
T4: Leader A comes back online
T5: Leader A sees epoch 2 > epoch 1, refuses to write
```

### Benefits

1. **Safe Failover**: Old leaders can't accidentally write after failover
2. **Clear Lineage**: Know which leader wrote which records
3. **Recovery**: Can identify and discard duplicate writes
4. **Auditing**: Track leadership changes over time

## The Key Innovation: Decoupling Offset Assignment from Consensus

### Problem: Consensus Bottleneck

**Without epochs**, every record write would need to go through Raft consensus:

```
Record arrives → Raft propose(offset) → Consensus → Commit → Assign offset
                        ↓
                 Bottleneck: ~10,000 records/sec
```

This creates a fundamental throughput limit because:
- Every record needs Raft agreement
- Raft log grows with every record
- Metadata store updated for every record
- Latency: multiple network round-trips per record

### Solution: Epochs Optimize Offset Assignment

**With epochs**, consensus happens once per epoch, not per record:

```
Epoch change (rare):
  New leader → Raft propose(new_epoch) → Consensus → Activate epoch
  
Record writes (frequent):
  Record arrives → Local offset++ → Write immediately (no Raft!)
                        ↓
                 Throughput: Millions of records/sec
```

### Performance Comparison

```
┌─────────────────────────────────────────────────┐
│         WITHOUT EPOCHS (Naive)                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  1,000,000 records to write                     │
│                                                 │
│  Each record:                                   │
│    1. Propose to Raft                          │
│    2. Wait for consensus                       │
│    3. Commit offset                            │
│    4. Write record                             │
│                                                 │
│  Raft throughput: ~10,000 ops/sec              │
│  Time: 1,000,000 / 10,000 = 100 seconds        │
│                                                 │
│  ❌ Raft log size: 1 million entries           │
│  ❌ Metadata writes: 1 million updates         │
│                                                 │
└─────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────┐
│         WITH EPOCHS (LogDevice Innovation)      │
├─────────────────────────────────────────────────┤
│                                                 │
│  1,000,000 records to write                     │
│                                                 │
│  Once per epoch (e.g., leader election):        │
│    1. Propose epoch change to Raft             │
│    2. Wait for consensus                       │
│    3. Activate epoch                           │
│                                                 │
│  Then for each record:                          │
│    1. Local offset++ (no Raft!)                │
│    2. Write record                             │
│                                                 │
│  Record throughput: ~1,000,000 ops/sec          │
│  Time: ~1 second                                │
│                                                 │
│  ✅ Raft log size: 1 entry (just epoch)        │
│  ✅ Metadata writes: 1 update (just epoch)     │
│                                                 │
│  Speedup: 100x faster! 🚀                       │
│                                                 │
└─────────────────────────────────────────────────┘
```

### What Gets Stored Where

```
┌──────────────────────────────────────────────────┐
│           Raft Log (Consensus)                   │
├──────────────────────────────────────────────────┤
│                                                  │
│  Only epoch changes:                             │
│    - ActivateEpoch(partition=2, epoch=5)        │
│    - SealEpoch(partition=2, epoch=4)            │
│                                                  │
│  Frequency: Rare (only on leader changes)       │
│  Size: O(leadership changes), not O(records)    │
│                                                  │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│        Metadata Store (RocksDB/etc)              │
├──────────────────────────────────────────────────┤
│                                                  │
│  Current epoch state:                            │
│    - partition_2_epoch: 5                       │
│    - partition_2_next_offset: 1000              │
│    - partition_2_sealed: false                  │
│                                                  │
│  Frequency: Updated once per epoch              │
│  Size: O(partitions), not O(records)            │
│                                                  │
└──────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────┐
│           Log Storage (Segments)                 │
├──────────────────────────────────────────────────┤
│                                                  │
│  Actual records with epoch tags:                 │
│    Record { epoch: 5, offset: 1000, ... }      │
│    Record { epoch: 5, offset: 1001, ... }      │
│    Record { epoch: 5, offset: 1002, ... }      │
│    ...                                          │
│                                                  │
│  Frequency: Every record write                  │
│  Size: O(records) - but no consensus needed!   │
│                                                  │
└──────────────────────────────────────────────────┘
```

### The Brilliant Trade-off

**Epochs achieve:**
- ✅ **High throughput**: Offset assignment is local (no Raft)
- ✅ **Safety**: Epochs prevent conflicts during failover
- ✅ **Scalability**: Raft log doesn't grow with records
- ✅ **Low latency**: No consensus round-trip per record

**Cost:**
- Small epoch number (8 bytes) stored with each record
- Slightly more complex recovery logic
- Epoch metadata management

**Result:** 100x+ throughput improvement with minimal overhead!

### Real-World Impact

```rust
// Without epochs - SLOW
for record in records {
    let offset = raft.propose_and_commit(record).await?;  // ~100ms per record
    storage.write(record, offset).await?;
}
// 1000 records = ~100 seconds

// With epochs - FAST
let epoch = raft.activate_epoch(partition).await?;  // ~100ms ONCE
for record in records {
    let offset = local_counter++;  // ~0.001ms (just increment!)
    record.epoch = epoch;
    storage.write(record, offset).await?;
}
// 1000 records = ~1 second
```

This is why **epochs are a critical innovation** from LogDevice that makes high-throughput distributed logs practical.

## Pyralog Implementation

### Core Types

#### Epoch
```rust
pub struct Epoch(pub u64);

const INVALID: Epoch = Epoch(0);
const FIRST: Epoch = Epoch(1);
```

#### EpochOffset
Combines epoch and offset within that epoch:
```rust
pub struct EpochOffset {
    pub epoch: Epoch,
    pub offset: u32,  // Offset within this epoch
}
```

Can be encoded as a 64-bit LSN (Log Sequence Number):
```
[Epoch: 32 bits][Offset: 32 bits]
```

#### EpochMetadata
Tracks information about each epoch:
```rust
pub struct EpochMetadata {
    pub current_epoch: Epoch,
    pub sequencer_node: u64,
    pub start_offset: u64,
    pub sealed: bool,
    pub last_known_offset: Option<u32>,
}
```

### Sequencer

The sequencer manages epochs for partitions:

```rust
pub struct Sequencer {
    node_id: u64,
    partition_epochs: HashMap<PartitionId, EpochStore>,
}

impl Sequencer {
    // Activate as leader for a partition
    pub fn activate(&self, partition: PartitionId) -> Epoch;
    
    // Seal an epoch (no more writes)
    pub fn seal_epoch(&self, partition: PartitionId, epoch: Epoch);
    
    // Check if can write
    pub fn can_write(&self, partition: PartitionId, epoch: Epoch) -> bool;
}
```

### Records with Epochs

Records now track their epoch:

```rust
pub struct Record {
    pub offset: LogOffset,
    pub epoch: Epoch,  // NEW
    pub timestamp: SystemTime,
    pub key: Option<Bytes>,
    pub value: Bytes,
    pub headers: Vec<RecordHeader>,
}
```

## Usage Examples

### 1. Becoming a Leader

```rust
use dlog_core::{Sequencer, PartitionId};

let sequencer = Sequencer::new(node_id);
let partition = PartitionId::new(0);

// Activate as leader
let epoch = sequencer.activate(partition, start_offset);
println!("Now leader with epoch: {}", epoch);
```

### 2. Writing Records

```rust
use dlog_core::{Record, Epoch};
use bytes::Bytes;

// Create record
let mut record = Record::new(
    Some(Bytes::from("key")),
    Bytes::from("value"),
);

// Tag with current epoch
let epoch = sequencer.current_epoch(partition).unwrap();
record = record.with_epoch(epoch);

// Write record
storage.append(record).await?;
```

### 3. Failover Scenario

```rust
// Leader node 1
let seq1 = Sequencer::new(1);
let partition = PartitionId::new(0);

// Node 1 activates
let epoch1 = seq1.activate(partition, 0);
// Write records in epoch 1...

// Node 1 fails or steps down
seq1.seal_epoch(partition, epoch1, 999);

// Leader node 2
let seq2 = Sequencer::new(2);

// Node 2 activates with new epoch
let epoch2 = seq2.activate(partition, 1000);
assert!(epoch2 > epoch1);
// Write records in epoch 2...
```

### 4. Recovery

```rust
// During recovery, check epoch validity
for record in records {
    let current_epoch = sequencer.current_epoch(partition).unwrap();
    
    if record.epoch < current_epoch {
        // Old epoch, already committed
        continue;
    } else if record.epoch == current_epoch {
        // Current epoch, check if within range
        if record.offset <= last_committed_offset {
            // Already committed
            continue;
        } else {
            // Need to commit
            commit_record(record);
        }
    } else {
        // Future epoch? Shouldn't happen
        warn!("Record from future epoch: {}", record.epoch);
    }
}
```

## Integration with Storage

### Segment Metadata

Segments now track their epoch range:

```rust
pub struct SegmentMetadata {
    pub base_offset: LogOffset,
    pub start_epoch: Epoch,
    pub end_epoch: Option<Epoch>,  // None if still active
    // ...
}
```

### Index Entries

Index entries include epoch information:

```rust
struct IndexEntry {
    offset: LogOffset,
    epoch: Epoch,
    position: u64,
    size: u32,
}
```

## Comparison: Pyralog vs LogDevice

| Feature | LogDevice | Pyralog |
|---------|-----------|------|
| Epoch Tracking | ✅ | ✅ |
| Sequencer Failover | ✅ | ✅ |
| Epoch Sealing | ✅ | ✅ |
| LSN Format | ✅ | ✅ (EpochOffset) |
| Epoch Metadata | ✅ | ✅ |
| Multiple Sequencers | ✅ | ✅ (via Raft) |

## Best Practices

### 1. Always Check Epoch Before Writing

```rust
let epoch = sequencer.current_epoch(partition)?;
if !sequencer.can_write(partition, epoch) {
    return Err(PyralogError::EpochSealed);
}
// Proceed with write...
```

### 2. Seal Epochs on Shutdown

```rust
async fn shutdown(&self, partition: PartitionId) -> Result<()> {
    let epoch = self.current_epoch(partition)?;
    let last_offset = self.get_last_offset(partition)?;
    self.sequencer.seal_epoch(partition, epoch, last_offset);
    Ok(())
}
```

### 3. Use EpochOffset for Stable References

```rust
// Instead of just offset:
let offset = LogOffset::new(1000);

// Use epoch offset:
let epoch_offset = EpochOffset::new(Epoch::new(5), 1000);
// This is stable across failovers
```

### 4. Store Epoch Metadata

```rust
// Persist epoch metadata
let metadata = EpochMetadata {
    current_epoch,
    sequencer_node,
    start_offset,
    sealed: false,
    last_known_offset: None,
};
metadata_store.save(partition, metadata)?;
```

## Performance Considerations

### Overhead

- **Memory**: ~48 bytes per epoch metadata
- **Disk**: Epoch number stored with each record (8 bytes)
- **CPU**: Minimal (simple integer comparisons)

### Optimizations

1. **Epoch Caching**: Cache current epoch in memory
2. **Batch Epoch Assignment**: Assign same epoch to batch
3. **Lazy Sealing**: Seal epochs asynchronously

## Future Enhancements

1. **Epoch Recovery Protocol**: Automated epoch recovery on startup
2. **Epoch Compaction**: Remove old epoch metadata
3. **Cross-Partition Epochs**: Global epoch for transactions
4. **Epoch Metrics**: Monitor epoch changes and duration

## Conclusion

Epochs are a crucial feature from LogDevice that make Pyralog's failover behavior safe and predictable. By tracking which leader wrote which records, we can:

- Prevent split-brain scenarios
- Enable safe failover
- Simplify recovery logic
- Provide clear audit trails

This brings Pyralog closer to LogDevice's robust sequencer design while maintaining compatibility with Raft-based consensus.

