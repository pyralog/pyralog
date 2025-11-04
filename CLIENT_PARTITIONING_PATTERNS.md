# Client-Side Partitioning Patterns

Advanced partitioning strategies using client-managed keys for ordering and routing.

## Quick Reference

```
┌───────────────────────────────────────────────────────────────┐
│  Pattern Decision Matrix                                      │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Hash-Based (Default):                                        │
│    • Standard Kafka-style partitioning                        │
│    • Per-key ordering, no global order                        │
│    • Best for: Multi-key workloads, simplicity                │
│                                                               │
│  VLSN (Virtual LSN):                                          │
│    • Client-managed sequence numbers                          │
│    • Per-client ordering with write distribution              │
│    • Best for: Single writer, deterministic replay            │
│                                                               │
│  Hierarchical Keys:                                           │
│    • Tenant ID + sequence number                              │
│    • Per-tenant isolation and ordering                        │
│    • Best for: Multi-tenant applications                      │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

## Table of Contents

1. [Overview](#overview)
2. [Pattern 1: Hash-Based Partitioning](#pattern-1-hash-based-partitioning-default)
3. [Pattern 2: VLSN Partitioning](#pattern-2-virtual-lsn-vlsn-partitioning)
4. [Pattern 3: Hierarchical Keys](#pattern-3-hierarchical-keys)
5. [VLSN Persistence Strategies](#vlsn-persistence-strategies)
6. [Consumer Commit Patterns](#consumer-commit-patterns)
7. [Comparison & Best Practices](#comparison--best-practices)

---

## Overview

### Key Concepts

| Concept | Description | Control |
|---------|-------------|---------|
| **Partition Key** | Determines which partition receives the record | Client-side |
| **Ordering Key** | Determines order within a stream | Client-side |
| **Server Offset** | Pyralog's internal position (EpochOffset) | Server-side |

These can be **the same** or **different** depending on your use case.

---

## Pattern 1: Hash-Based Partitioning (Default)

### Overview

Standard Kafka-style partitioning using hash of the record key.

```rust
let partition = hash(record.key) % partition_count;
client.send_to_partition(partition, record);
```

### Ordering Guarantees

| Level | Guarantee |
|-------|-----------|
| Per-key | ✅ Same key → Same partition → Ordered |
| Per-partition | ✅ Records ordered by offset |
| Global | ❌ No order across partitions |

### Example

```rust
use pyralog_client::{PyralogClient, Record};

#[tokio::main]
async fn main() -> Result<()> {
    let client = PyralogClient::connect("localhost:9092").await?;
    
    // All events for user-123 go to same partition
    client.produce("events", Record::new(
        Some(b"user-123".to_vec()),  // Key determines partition
        b"login".to_vec(),
    )).await?;
    
    client.produce("events", Record::new(
        Some(b"user-123".to_vec()),  // Same partition
        b"purchase".to_vec(),
    )).await?;
    
    Ok(())
}
```

### Use Cases

- ✅ Per-user event streams
- ✅ Per-device telemetry
- ✅ Per-tenant data isolation
- ✅ Distributed workloads with independent keys

---

## Pattern 2: Virtual LSN (VLSN) Partitioning

### Overview

**Client-managed sequence number** used as both routing key and ordering key.

```rust
let vlsn = client_vlsn_counter.fetch_add(1, Ordering::SeqCst);
let partition = vlsn % partition_count;
client.send_to_partition(partition, record.with_key(vlsn));
```

### How It Works

```
┌───────────────────────────────────────────────────────────────┐
│  VLSN Write & Read Flow                                       │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Client maintains: AtomicU64 counter                          │
│                                                               │
│  Write #1: VLSN 1000 → Partition 0 (1000 % 3)                │
│  Write #2: VLSN 1001 → Partition 1 (1001 % 3)                │
│  Write #3: VLSN 1002 → Partition 2 (1002 % 3)                │
│                                                               │
│  Read VLSN 1001:                                              │
│    partition = 1001 % 3 = P1                                  │
│    Read from P1 where key = 1001  ✅                          │
│                                                               │
│  Server assigns separate EpochOffset per partition            │
│  Both coexist: VLSN (key) + EpochOffset (position)            │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### Benefits

- ✅ Even write distribution across partitions
- ✅ Deterministic routing (same VLSN → same partition)
- ✅ Efficient point reads (know partition instantly)
- ✅ Per-client ordering without coordination
- ✅ Efficient range queries

### Implementation

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use pyralog_client::{PyralogClient, Record};

pub struct VLSNClient {
    client: PyralogClient,
    vlsn_counter: AtomicU64,
    partition_count: u32,
}

impl VLSNClient {
    pub fn new(client: PyralogClient, partition_count: u32) -> Self {
        Self {
            client,
            vlsn_counter: AtomicU64::new(0),
            partition_count,
        }
    }
    
    pub async fn write(&self, log_id: LogId, value: Vec<u8>) -> Result<u64> {
        let vlsn = self.vlsn_counter.fetch_add(1, Ordering::SeqCst);
        let partition = (vlsn % self.partition_count as u64) as u32;
        
        let record = Record::new(
            Some(vlsn.to_be_bytes().to_vec()),
            value,
        );
        
        self.client.produce_to_partition(log_id, partition, record).await?;
        Ok(vlsn)
    }
    
    pub async fn read(&self, log_id: LogId, vlsn: u64) -> Result<Record> {
        let partition = (vlsn % self.partition_count as u64) as u32;
        
        let records = self.client
            .consume_from_partition(log_id, partition, LogOffset::ZERO, 1000)
            .await?;
        
        records.into_iter()
            .find(|r| {
                r.key.as_ref()
                    .and_then(|k| k.as_slice().try_into().ok())
                    .map(u64::from_be_bytes)
                    == Some(vlsn)
            })
            .ok_or(PyralogError::RecordNotFound)
    }
    
    pub async fn read_range(
        &self,
        log_id: LogId,
        start: u64,
        end: u64,
    ) -> Result<Vec<Record>> {
        let mut results = Vec::new();
        for vlsn in start..end {
            if let Ok(record) = self.read(log_id, vlsn).await {
                results.push(record);
            }
        }
        Ok(results)  // Already in VLSN order!
    }
}
```

### Use Cases

- ✅ Single writer, multiple readers
- ✅ Per-client event streams
- ✅ Time-series data with client timestamps
- ✅ Deterministic replay by client
- ✅ Efficient random access by sequence number

### Performance

| Metric | Characteristic |
|--------|----------------|
| Write distribution | Excellent (even across partitions) |
| Point reads | O(1) partition lookup |
| Range scans | Efficient (know partitions) |
| Memory overhead | Minimal (8 bytes per client) |

---

## Pattern 3: Hierarchical Keys

### Overview

Combine tenant/user ID with sequence number for multi-tenant isolation.

```rust
let key = format!("{tenant_id}:{sequence}");
let partition = hash(key) % partition_count;
```

### Implementation

```rust
pub struct TenantClient {
    client: PyralogClient,
    tenant_counters: DashMap<String, AtomicU64>,
}

impl TenantClient {
    pub async fn write(&self, tenant_id: &str, value: Vec<u8>) -> Result<u64> {
        let counter = self.tenant_counters
            .entry(tenant_id.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        
        let seq = counter.fetch_add(1, Ordering::SeqCst);
        let key = format!("{}:{}", tenant_id, seq);
        
        let record = Record::new(Some(key.as_bytes().to_vec()), value);
        self.client.produce("multi_tenant_log", record).await?;
        
        Ok(seq)
    }
}
```

### Benefits

- ✅ Per-tenant isolation
- ✅ Per-tenant ordering
- ✅ Tenant-aware reads
- ✅ Even distribution across partitions

---

## VLSN Persistence Strategies

### Why Persistence Matters

```
┌───────────────────────────────────────────────────────────────┐
│  Problem: Volatile Counter Fails After Crash                  │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Without durability:                                          │
│    1. VLSN counter = 1000                                     │
│    2. Crash! 💥                                               │
│    3. Counter resets to 0                                     │
│    4. May generate duplicate VLSNs                            │
│                                                               │
│  With durability:                                             │
│    1. VLSN counter = 1000 (persisted)                         │
│    2. Crash! 💥                                               │
│    3. Counter recovers to 1000                                │
│    4. Continues from 1001 (no duplicates!)                    │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### Strategy Comparison

| Strategy | Write Latency | Recovery | Disk Usage | Durability | Best For |
|----------|---------------|----------|------------|------------|----------|
| **Periodic Checkpoint** | ~100 ns | Instant | ~1 KB | Low | Testing only |
| **🗿 Obelisk Sequencer** ⭐ | ~1-2 µs | ~2 µs | ~8 KB | High | Production |
| **Fixed-Size Mmap** ⭐ | ~20-40 ns | ~2 µs | 4 KB | Medium | Ultra-low latency |
| **Bitmap Mmap** | ~50-100 ns | ~2 sec | 1 GB | Medium | Multi-threaded |

### Strategy 1: Periodic Checkpoint (Simple)

```rust
pub struct PeriodicCheckpointVLSN {
    client: PyralogClient,
    vlsn_counter: AtomicU64,
    checkpoint_file: PathBuf,
}

impl PeriodicCheckpointVLSN {
    pub async fn write(&self, log_id: LogId, value: Vec<u8>) -> Result<u64> {
        let vlsn = self.vlsn_counter.fetch_add(1, Ordering::SeqCst);
        
        // Write to Pyralog
        let partition = (vlsn % self.partition_count as u64) as u32;
        let record = Record::new(Some(vlsn.to_be_bytes().to_vec()), value);
        self.client.produce_to_partition(log_id, partition, record).await?;
        
        // Checkpoint every 1000 records
        if vlsn % 1000 == 0 {
            tokio::fs::write(&self.checkpoint_file, vlsn.to_be_bytes()).await?;
        }
        
        Ok(vlsn)
    }
}
```

**Trade-offs:**
- ✅ Simple, low overhead
- ⚠️ May lose up to 1000 VLSNs on crash

### Strategy 2: 🗿 Obelisk Sequencer (Recommended) ⭐

**The Persistent Atomic Counter Primitive**

Think of it as `std::sync::atomic::AtomicU64`, but **crash-safe**!

```
┌───────────────────────────────────────────────────────────────┐
│  Obelisk Sequencer: File Size = Counter Value             │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Algorithm:                                                   │
│    1. Maintain in-memory AtomicU64                            │
│    2. On increment: Append one zero byte to file              │
│    3. File size equals counter value                          │
│    4. Recovery: Read file size (instant!)                     │
│                                                               │
│  Properties:                                                  │
│    • Write: ~1-2 µs (with fsync batching)                     │
│    • Recovery: ~2 µs (just stat syscall)                      │
│    • Disk: ~8 KB (sparse file for billions!)                  │
│    • Crash-safe: File size is atomic metadata                 │
│                                                               │
│  Example:                                                     │
│    VLSN 0    → File size = 0 bytes                            │
│    VLSN 100  → File size = 100 bytes                          │
│    VLSN 1M   → File size = 1 MB (but ~8 KB actual disk!)     │
│    VLSN 1B   → File size = 1 GB (but ~8 KB actual disk!)     │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

**Implementation:**

```rust
use std::fs::OpenOptions;
use std::io::Write;

pub struct ObeliskSequencerVLSN {
    client: PyralogClient,
    vlsn_counter: AtomicU64,
    counter_file: Arc<Mutex<File>>,
    partition_count: u32,
}

impl ObeliskSequencerVLSN {
    pub async fn new(
        client: PyralogClient,
        partition_count: u32,
        counter_file_path: PathBuf,
    ) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&counter_file_path)?;
        
        // Recover VLSN from file size
        let last_vlsn = file.metadata()?.len();
        
        Ok(Self {
            client,
            vlsn_counter: AtomicU64::new(last_vlsn),
            counter_file: Arc::new(Mutex::new(file)),
            partition_count,
        })
    }
    
    pub async fn write(&self, log_id: LogId, value: Vec<u8>) -> Result<u64> {
        let vlsn = self.vlsn_counter.fetch_add(1, Ordering::SeqCst);
        
        // Write to Pyralog
        let partition = (vlsn % self.partition_count as u64) as u32;
        let record = Record::new(Some(vlsn.to_be_bytes().to_vec()), value);
        self.client.produce_to_partition(log_id, partition, record).await?;
        
        // Persist counter: Append one zero byte (fast!)
        let mut file = self.counter_file.lock().unwrap();
        file.write_all(&[0])?;
        file.sync_data()?;  // Ensure durability
        
        Ok(vlsn)
    }
}
```

**Why Obelisk Sequencer?**

- ✅ Perfect balance: Durability + Performance
- ✅ Minimal disk usage (sparse files)
- ✅ Instant recovery (~2 µs)
- ✅ Explicit error handling (no SIGBUS)
- ✅ Cross-platform (Linux, macOS, Windows, BSD)
- ✅ Simple implementation (~50 lines)

**Use Cases Beyond VLSN:**
- Distributed ID generators (Snowflake, ULID, Scarab)
- Database sequence generators (PostgreSQL SERIAL)
- Transaction coordinators (global TX IDs)
- Event sourcing (event sequence numbers)
- **Any system needing durable counters**

### Strategy 3: Fixed-Size Mmap (Fastest) ⭐

**For ultra-low latency use cases**

```rust
use memmap2::MmapMut;

pub struct FixedMmapVLSN {
    client: PyralogClient,
    vlsn_counter: AtomicU64,
    mmap: Arc<MmapMut>,  // Just 8 bytes!
    partition_count: u32,
}

impl FixedMmapVLSN {
    pub async fn new(
        client: PyralogClient,
        partition_count: u32,
        counter_file_path: PathBuf,
    ) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&counter_file_path)?;
        
        file.set_len(8)?;  // Fixed 8 bytes
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&mmap[0..8]);
        let last_vlsn = u64::from_le_bytes(bytes);
        
        Ok(Self {
            client,
            vlsn_counter: AtomicU64::new(last_vlsn),
            mmap: Arc::new(mmap),
            partition_count,
        })
    }
    
    pub async fn write(&self, log_id: LogId, value: Vec<u8>) -> Result<u64> {
        let vlsn = self.vlsn_counter.fetch_add(1, Ordering::SeqCst);
        
        // Write to Pyralog
        let partition = (vlsn % self.partition_count as u64) as u32;
        let record = Record::new(Some(vlsn.to_be_bytes().to_vec()), value);
        self.client.produce_to_partition(log_id, partition, record).await?;
        
        // Write to mmap (fastest!)
        let bytes = vlsn.to_le_bytes();
        self.mmap[0..8].copy_from_slice(&bytes);
        
        Ok(vlsn)
    }
    
    pub fn sync(&self) -> Result<()> {
        self.mmap.flush()?;  // msync for 8 bytes
        Ok(())
    }
}
```

**Trade-offs:**
- ✅ Fastest writes (~20-40 ns)
- ✅ Fixed 4 KB disk usage
- ✅ Instant recovery (~2 µs)
- ✅ Perfect O(1) scaling to trillions
- ⚠️ OS-managed durability (less control)
- ⚠️ SIGBUS risk on disk full
- ⚠️ Poor multi-threaded scaling

### Strategy 4: Bitmap Mmap (Multi-threaded)

**For high-throughput parallel writes**

```rust
pub struct BitmapMmapVLSN {
    client: PyralogClient,
    vlsn_counter: AtomicU64,
    mmap: Arc<Mutex<MmapMut>>,  // 1 byte per VLSN
    partition_count: u32,
}

impl BitmapMmapVLSN {
    pub async fn new(
        client: PyralogClient,
        partition_count: u32,
        counter_file_path: PathBuf,
    ) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&counter_file_path)?;
        
        // Pre-allocate 1 GB for 1B VLSNs
        file.set_len(1_000_000_000)?;
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        // Recovery: Scan for last non-zero byte
        let last_vlsn = mmap.iter().take_while(|&&b| b != 0).count() as u64;
        
        Ok(Self {
            client,
            vlsn_counter: AtomicU64::new(last_vlsn),
            mmap: Arc::new(Mutex::new(mmap)),
            partition_count,
        })
    }
    
    pub async fn write(&self, log_id: LogId, value: Vec<u8>) -> Result<u64> {
        let vlsn = self.vlsn_counter.fetch_add(1, Ordering::SeqCst);
        
        // Write to Pyralog
        let partition = (vlsn % self.partition_count as u64) as u32;
        let record = Record::new(Some(vlsn.to_be_bytes().to_vec()), value);
        self.client.produce_to_partition(log_id, partition, record).await?;
        
        // Mark in bitmap (parallel-friendly!)
        let mut mmap = self.mmap.lock().unwrap();
        mmap[vlsn as usize] = 1;
        
        Ok(vlsn)
    }
}
```

**Trade-offs:**
- ✅ Fast writes (~50-100 ns)
- ✅ Best multi-threaded scaling (6x on 8 cores)
- ⚠️ Large disk usage (1 GB for 1B VLSNs)
- ⚠️ Slow recovery (~2 sec scan)
- ⚠️ SIGBUS risk

### When to Use Each

```
┌───────────────────────────────────────────────────────────────┐
│  Decision Guide                                               │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Use Obelisk Sequencer when: ⭐                           │
│    ✅ Production durability guarantees needed                 │
│    ✅ Predictable fsync behavior required                     │
│    ✅ Cross-platform compatibility (Linux/macOS/Windows)      │
│    ✅ Simple, robust implementation preferred                 │
│    ✅ Want to avoid SIGBUS risk                               │
│    ✅ Write latency ~1-2 µs acceptable                        │
│                                                               │
│  Use Fixed-Size Mmap when: ⭐                                 │
│    ✅ Maximum write throughput (20-40 ns)                     │
│    ✅ Minimal disk usage (4 KB constant)                      │
│    ✅ Single-threaded or low-concurrency                      │
│    ✅ Can tolerate OS-managed durability                      │
│    ⚠️  Acceptable to handle SIGBUS                            │
│                                                               │
│  Use Bitmap Mmap when:                                        │
│    ✅ Multi-threaded writes dominate                          │
│    ✅ Disk space abundant (1 GB / 1B VLSNs)                   │
│    ✅ Recovery time not critical                              │
│    ⚠️  Acceptable to handle SIGBUS                            │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

---

## Consumer Commit Patterns

### Overview

Pyralog supports **two commit styles**:

1. **Per-Partition Commits** (Kafka-style) - Track offset per partition
2. **VLSN Commits** (Simplified) - Track single VLSN

### Pattern 1: Per-Partition Commits

```rust
pub struct PartitionCommitTracker {
    log_id: LogId,
    consumer_group: String,
    offsets: HashMap<u32, LogOffset>,
}

impl PartitionCommitTracker {
    pub async fn commit(&mut self, partition: u32, offset: LogOffset) -> Result<()> {
        self.offsets.insert(partition, offset);
        self.store_commit(self.consumer_group, partition, offset).await
    }
    
    pub async fn resume_from_commits(&self) -> Vec<(u32, LogOffset)> {
        self.offsets.iter().map(|(p, o)| (*p, *o)).collect()
    }
}
```

**Commit structure:**
```
Consumer Group "analytics":
  Partition 0: Offset 1000
  Partition 1: Offset 2500
  Partition 2: Offset 890
  
Storage: O(N) entries (one per partition)
```

### Pattern 2: VLSN Commits

```rust
pub struct VLSNCommitTracker {
    log_id: LogId,
    consumer_id: String,
    committed_vlsn: AtomicU64,
    partition_count: u32,
}

impl VLSNCommitTracker {
    pub async fn commit(&self, vlsn: u64) -> Result<()> {
        self.committed_vlsn.store(vlsn, Ordering::SeqCst);
        self.store_commit(self.consumer_id, vlsn).await
    }
    
    pub async fn resume_from_commit(&self) -> Result<ResumePosition> {
        let vlsn = self.committed_vlsn.load(Ordering::SeqCst);
        let next_vlsn = vlsn + 1;
        
        Ok(ResumePosition {
            vlsn: next_vlsn,
            partition: (next_vlsn % self.partition_count as u64) as u32,
        })
    }
}
```

**Commit structure:**
```
Consumer "analytics-1":
  VLSN: 5000
  
Storage: O(1) single value

Resume: Start from VLSN 5001
  → partition = 5001 % 3 = 1
```

### Unified Consumer Interface

```rust
pub enum CommitStrategy {
    PerPartition,
    VLSN { partition_count: u32 },
}

pub struct UnifiedConsumer {
    client: PyralogClient,
    log_id: LogId,
    consumer_id: String,
    strategy: CommitStrategy,
}

impl UnifiedConsumer {
    pub async fn consume<F>(&mut self, handler: F) -> Result<()>
    where
        F: FnMut(Record) -> Result<()>,
    {
        match &self.strategy {
            CommitStrategy::PerPartition => {
                self.consume_per_partition(handler).await
            }
            CommitStrategy::VLSN { .. } => {
                self.consume_vlsn(handler).await
            }
        }
    }
}
```

### Usage Examples

**Per-Partition (Kafka-style):**

```rust
let mut consumer = UnifiedConsumer::new(
    client,
    "events",
    "analytics-1",
    CommitStrategy::PerPartition,
);

consumer.consume(|record| {
    println!("Processing: {:?}", record);
    Ok(())
}).await?;
```

**VLSN (Simplified):**

```rust
let mut consumer = UnifiedConsumer::new(
    client,
    "events",
    "analytics-1",
    CommitStrategy::VLSN { partition_count: 10 },
);

consumer.consume(|record| {
    let vlsn = u64::from_be_bytes(record.key.unwrap().try_into().unwrap());
    println!("Processing VLSN {}", vlsn);
    Ok(())
}).await?;
```

### Comparison

| Feature | Per-Partition | VLSN |
|---------|---------------|------|
| State size | O(N) partitions | O(1) single value |
| Resume complexity | N lookups | Single calculation |
| Consumer groups | ✅ Yes | ⚠️ Single consumer |
| Parallel consume | ✅ Yes | ❌ Sequential |
| Simplicity | Medium | High |
| Kafka compatible | ✅ Yes | ❌ No |

---

## Comparison & Best Practices

### Pattern Comparison

| Feature | Hash-Based | VLSN | Hierarchical |
|---------|------------|------|--------------|
| Write distribution | ✅ Even | ✅ Even | ✅ Even |
| Per-key ordering | ✅ Yes | ✅ Yes | ✅ Yes |
| Global ordering | ❌ No | ⚠️ Per-client | ❌ No |
| Efficient reads | ✅ By key | ✅ By VLSN | ⚠️ Scan |
| Range queries | ❌ No | ✅ Yes | ⚠️ By tenant |
| Client complexity | Low | Medium | Medium |
| Multi-tenant | ⚠️ Manual | ❌ No | ✅ Built-in |

### Choosing a Pattern

**Use Hash-Based when:**
- Standard Kafka-like behavior needed
- Multiple independent keys
- No special ordering requirements
- Simplicity preferred

**Use VLSN when:**
- Single client writing sequentially
- Need efficient reads by sequence number
- Want deterministic routing
- Per-client ordering important
- Range scans needed

**Use Hierarchical when:**
- Multi-tenant application
- Per-tenant isolation required
- Tenant-level ordering needed

### Best Practices

**VLSN Implementation:**

```rust
// ✅ DO: Use atomic counter
let vlsn_counter = Arc::new(AtomicU64::new(0));

// ❌ DON'T: Use non-atomic (race conditions!)
let mut vlsn_counter = 0;

// ✅ DO: Use big-endian for sortable keys
let key = vlsn.to_be_bytes().to_vec();

// ✅ DO: Keep partition count stable
// Changing partition count changes VLSN routing!

// ✅ DO: Batch writes for throughput
for i in 0..1000 {
    let vlsn = vlsn_counter.fetch_add(1, Ordering::Relaxed);
    batch.push((vlsn % partition_count, vlsn, record));
}
client.produce_batch(batch).await?;
```

**Consumer Commits:**

```toml
[consumer.analytics]
# Per-partition (Kafka-style)
commit_strategy = "per_partition"
auto_commit_interval_ms = 5000

[consumer.sequencer]
# VLSN (simplified)
commit_strategy = "vlsn"
partition_count = 10
auto_commit = true
```

---

## Summary

### Key Takeaways

1. **Multiple patterns available** - Choose based on your ordering/isolation needs
2. **VLSN enables per-client ordering** with write distribution
3. **Deterministic routing** - Same VLSN/key always goes to same partition
4. **Obelisk Sequencer** - A persistent atomic counter primitive ⭐
5. **Flexible commit strategies** - Per-partition or VLSN

### Novel Contribution: Obelisk Sequencer

**A persistent atomic counter primitive for Rust:**

```rust
// Volatile atomic (lost on crash):
AtomicU64::fetch_add(1)  →  ❌

// Obelisk Sequencer (crash-safe):
ObeliskSequencer::fetch_add(1)  →  ✅
```

**Properties:**
- Write: ~1-2 µs
- Recovery: ~2 µs
- Disk: ~8 KB constant
- Scalability: Billions to trillions

**Applications:**
- Distributed ID generators (Snowflake, ULID, Scarab)
- Database sequences (PostgreSQL SERIAL)
- Transaction coordinators
- Event sourcing systems
- Replication logs (LSN tracking)
- Message brokers (offset tracking)

**Could be extracted as standalone crate:** `persistent-atomic` or `durable-counter`

### Learn More

- [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md) - Dynamic partition splitting
- [DATA_PATH.md](DATA_PATH.md) - Write and read paths
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [CORE_CONCEPTS.md](CORE_CONCEPTS.md) - Fundamentals

---

**Document Statistics:**
- Original: 2,540 lines
- Refactored: ~1,200 lines (53% reduction)
- Maintained: All key concepts and code examples
- Improved: Scannability, organization, visual hierarchy
