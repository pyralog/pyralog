# Pyralog Data Path Architecture

Detailed documentation of write and read paths through Pyralog, including diagrams and step-by-step flows.

## Table of Contents

1. [Write Path](#write-path)
2. [Read Path](#read-path)
3. [Batch Write Path](#batch-write-path)
4. [Replication Flow](#replication-flow)
5. [Failure Scenarios](#failure-scenarios)
6. [Performance Optimizations](#performance-optimizations)
7. [Smart Client Architecture](#smart-client-architecture)

---

## Write Path

### High-Level Write Flow

```
┌─────────┐
│ Client  │
└────┬────┘
     │ 1. produce(record)
     ▼
┌─────────────────┐
│  Pyralog Server    │
│  (Protocol)     │
└────┬────────────┘
     │ 2. route to partition
     ▼
┌─────────────────┐
│  Partitioner    │ ───→ hash(key) % partition_count
└────┬────────────┘
     │ 3. assign epoch & offset
     ▼
┌─────────────────┐
│  Sequencer      │ ───→ current_epoch, next_offset
└────┬────────────┘
     │ 4. write to cache/storage
     ▼
┌─────────────────┐
│  Write Cache    │ ───→ buffer in memory
└────┬────────────┘
     │ 5. flush (async or on threshold)
     ▼
┌─────────────────┐
│  Log Storage    │
│  - Segment      │ ───→ append to active segment
│  - Index        │ ───→ update offset index
└────┬────────────┘
     │ 6. replicate (parallel)
     ▼
┌─────────────────┐
│  Replication    │ ───→ send to followers
│  Manager        │
└────┬────────────┘
     │ 7. wait for quorum
     ▼
┌─────────────────┐
│  Quorum         │ ───→ W nodes ACK
│  Coordinator    │
└────┬────────────┘
     │ 8. return offset
     ▼
┌─────────┐
│ Client  │ ←─── offset: 12345
└─────────┘
```

### Detailed Write Path Steps

#### Step 1: Client Request

```rust
// Client code
let record = Record::new(
    Some(Bytes::from("user-123")),  // key
    Bytes::from("order data"),       // value
);

let offset = client.produce(log_id, record).await?;
```

**What happens**:
- Client serializes record
- Sends produce request to server
- Includes log ID and partition (optional)

#### Step 2: Server Protocol Layer

```
┌──────────────────────────────────────┐
│         Pyralog Server (Node 1)         │
├──────────────────────────────────────┤
│  ┌────────────────────────────────┐  │
│  │  Protocol Handler              │  │
│  │  - Parse request               │  │
│  │  - Validate permissions        │  │
│  │  - Extract log_id & record     │  │
│  └────────────┬───────────────────┘  │
│               │                       │
│               ▼                       │
│  ┌────────────────────────────────┐  │
│  │  Log Router                    │  │
│  │  - Find log metadata           │  │
│  │  - Get partition count         │  │
│  └────────────┬───────────────────┘  │
└───────────────┼───────────────────────┘
                │
                ▼
```

```rust
impl PyralogServer {
    async fn handle_produce(&self, request: ProduceRequest) -> Result<ProduceResponse> {
        // 1. Get log metadata
        let metadata = self.cluster.get_log(&request.log_id)?;
        
        // 2. Determine partition
        let partition = self.determine_partition(&request, &metadata)?;
        
        // 3. Check if leader for this partition
        if !self.is_leader(partition) {
            return Err(PyralogError::NotLeader(self.get_leader(partition)));
        }
        
        // 4. Continue to write path...
    }
}
```

#### Step 3: Partitioning

```
┌─────────────────────────────────────┐
│        Partitioner                  │
├─────────────────────────────────────┤
│                                     │
│  if record.key.is_some() {          │
│      hash(key) % partition_count    │
│  } else {                           │
│      round_robin()                  │
│  }                                  │
│                                     │
└──────────────┬──────────────────────┘
               │
               ▼
    Partition: 2 (of 0-7)
```

**Partitioning strategies**:

```rust
match strategy {
    KeyHash => {
        let hash = hash(record.key);
        partition = hash % partition_count;
    }
    RoundRobin => {
        partition = counter.fetch_add(1) % partition_count;
    }
    Sticky => {
        partition = current_sticky_partition;
    }
}
```

#### Step 4: Epoch & Offset Assignment

```
┌─────────────────────────────────────┐
│         Sequencer                   │
├─────────────────────────────────────┤
│                                     │
│  Current State:                     │
│    partition_id: 2                  │
│    current_epoch: 5                 │
│    next_offset: 1000                │
│                                     │
│  Assign to record:                  │
│    record.epoch = 5                 │
│    record.offset = 1000             │
│    next_offset++ = 1001             │
│                                     │
└──────────────┬──────────────────────┘
               │
               ▼
      Record with epoch=5, offset=1000
```

```rust
impl Sequencer {
    pub fn assign(&mut self, partition: PartitionId, record: &mut Record) -> Result<()> {
        let epoch = self.current_epoch(partition)?;
        let offset = self.next_offset(partition)?;
        
        // Check if we can write
        if !self.can_write(partition, epoch) {
            return Err(PyralogError::EpochSealed);
        }
        
        record.epoch = epoch;
        record.offset = LogOffset::new(offset);
        
        self.increment_offset(partition);
        Ok(())
    }
}
```

#### Step 5: Write Cache

```
┌─────────────────────────────────────────────┐
│           Write Cache                       │
├─────────────────────────────────────────────┤
│                                             │
│  Buffer: [record1, record2, ..., recordN]   │
│  Total Size: 8MB / 16MB                     │
│  Last Flush: 5ms ago                        │
│                                             │
│  Check flush conditions:                    │
│    if size >= max_size        ──→ FLUSH     │
│    if time >= max_buffer_time ──→ FLUSH     │
│    if count >= max_records    ──→ FLUSH     │
│  else:                                      │
│    buffer.push(record)                      │
│    return Ok(offset) immediately            │
│                                             │
└──────────────┬──────────────────────────────┘
               │
               ▼
     Fast return to client (sub-ms)
```

**Write cache decision tree**:

```
New record arrives
    │
    ▼
Cache enabled? ──No──→ Write directly to storage
    │
    Yes
    ▼
Cache full? ──Yes──→ Flush cache, then write
    │
    No
    ▼
Buffer timeout? ──Yes──→ Flush cache, then write
    │
    No
    ▼
Add to cache buffer
    │
    ▼
Return offset to client (fast!)
```

#### Step 6: Storage Layer

```
┌──────────────────────────────────────────────┐
│            Log Storage                       │
├──────────────────────────────────────────────┤
│                                              │
│  ┌────────────────────────────────────────┐ │
│  │  Active Segment                        │ │
│  │  (/data/log/partition-2/1000000.log)  │ │
│  │                                        │ │
│  │  Current Size: 850MB / 1GB             │ │
│  │  Base Offset: 1000000                  │ │
│  │  Records: 1000000-1050000              │ │
│  └────────────┬───────────────────────────┘ │
│               │                              │
│               ▼                              │
│  1. Serialize record (bincode)              │
│  2. Calculate CRC checksum                  │
│  3. Append to segment file                  │
│  4. Update index                            │
│               │                              │
│               ▼                              │
│  ┌────────────────────────────────────────┐ │
│  │  Index                                 │ │
│  │  (/data/log/partition-2/1000000.index)│ │
│  │                                        │ │
│  │  [offset][position][size]              │ │
│  │  1000000  0         256                │ │
│  │  1000100  25600     512                │ │
│  │  1000200  51200     384                │ │
│  │  ...                                   │ │
│  │  1050000  850000000 412  ← new entry  │ │
│  └────────────────────────────────────────┘ │
└──────────────────────────────────────────────┘
```

**Storage operations**:

```rust
impl LogStorage {
    async fn append(&self, record: Record) -> Result<LogOffset> {
        // 1. Serialize
        let data = bincode::serialize(&record)?;
        
        // 2. Get active segment
        let segment = self.get_active_segment()?;
        
        // 3. Check if segment has space
        if !segment.can_fit(data.len()) {
            self.roll_segment().await?;
            segment = self.get_active_segment()?;
        }
        
        // 4. Append to segment
        let position = segment.append(&data)?;
        
        // 5. Update index
        let index = self.get_index(segment.id())?;
        index.append(record.offset, position, data.len())?;
        
        Ok(record.offset)
    }
}
```

#### Step 7: Replication

```
┌─────────────────────────────────────────────────────────┐
│                    Replication                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Leader (Node 1)                                        │
│     │                                                   │
│     │ 1. Select CopySet [Node 1, Node 2, Node 3]       │
│     │                                                   │
│     ├──────────────────┬────────────────────────┐      │
│     │                  │                        │      │
│     ▼                  ▼                        ▼      │
│  ┌─────────┐      ┌─────────┐             ┌─────────┐ │
│  │ Node 1  │      │ Node 2  │             │ Node 3  │ │
│  │ (self)  │      │         │             │         │ │
│  │ Offset: │      │ Offset: │             │ Offset: │ │
│  │  1000   │      │  998    │             │  995    │ │
│  └────┬────┘      └────┬────┘             └────┬────┘ │
│       │                │                        │      │
│       │ 2. Send AppendEntries RPC              │      │
│       │                │                        │      │
│       │                ▼                        ▼      │
│       │           Write record             Write record│
│       │           Return ACK               Return ACK  │
│       │                │                        │      │
│       └────────────────┴────────────────────────┘      │
│                        │                                │
│  3. Wait for W=2 ACKs (quorum)                         │
│                        │                                │
│  4. Update ISR: [Node 1, Node 2, Node 3]               │
│                        │                                │
│  5. Commit offset: 1000                                │
└────────────────────────┼────────────────────────────────┘
                         │
                         ▼
                    Return to client
```

**Replication flow**:

```rust
impl ReplicationManager {
    async fn replicate(&self, partition: PartitionId, record: Record) -> Result<()> {
        // 1. Get CopySet for partition
        let copyset = self.get_copyset(partition)?;
        
        // 2. Create quorum tracker
        let mut quorum = QuorumSet::new(
            copyset.nodes.clone(),
            self.config.write_quorum,
        );
        
        // 3. Send to all replicas in parallel
        let futures: Vec<_> = copyset.nodes.iter()
            .filter(|&&node| node != self.node_id) // Skip self
            .map(|&node| self.send_to_replica(node, record.clone()))
            .collect();
        
        // 4. Wait for write quorum
        for result in futures::future::join_all(futures).await {
            if result.is_ok() {
                quorum.add_response(result.node_id);
                if quorum.is_satisfied() {
                    break; // Got quorum, can return early
                }
            }
        }
        
        // 5. Check if quorum reached
        if !quorum.is_satisfied() {
            return Err(PyralogError::QuorumNotAvailable);
        }
        
        Ok(())
    }
}
```

#### Step 8: Client Response

```
┌────────────────────────────────────┐
│         Response Flow              │
├────────────────────────────────────┤
│                                    │
│  ACK Mode: Leader                  │
│    │                               │
│    ▼                               │
│  Quorum satisfied?                 │
│    │                               │
│    Yes                             │
│    ▼                               │
│  Build response:                   │
│    partition: 2                    │
│    offset: 1000                    │
│    error: None                     │
│    │                               │
│    ▼                               │
│  Serialize & send to client        │
│    │                               │
│    ▼                               │
│  ┌──────────────────────────────┐ │
│  │  Client receives:            │ │
│  │  {                           │ │
│  │    "partition": 2,           │ │
│  │    "offset": 1000,           │ │
│  │    "timestamp": "...",       │ │
│  │  }                           │ │
│  └──────────────────────────────┘ │
└────────────────────────────────────┘
```

### Complete Write Path Diagram

```
Client
  │
  │ produce(key="user-123", value="order data")
  ▼
┌─────────────────────────────────────────────────────────┐
│ Pyralog Server (Leader for Partition 2)                   │
│                                                         │
│  Step 1: Protocol Layer                                │
│  ├─ Parse request                                      │
│  ├─ Validate                                           │
│  └─ Extract record                                     │
│      │                                                 │
│      ▼                                                 │
│  Step 2: Partitioner                                   │
│  ├─ hash("user-123") % 8                              │
│  └─ partition = 2                                     │
│      │                                                 │
│      ▼                                                 │
│  Step 3: Check Leadership                              │
│  ├─ Am I leader for partition 2? ✓                    │
│  └─ Continue...                                        │
│      │                                                 │
│      ▼                                                 │
│  Step 4: Sequencer                                     │
│  ├─ epoch = 5                                         │
│  ├─ offset = 1000                                     │
│  └─ record.epoch = 5, record.offset = 1000           │
│      │                                                 │
│      ▼                                                 │
│  Step 5: Write Cache (if enabled)                     │
│  ├─ cache.push(record)                                │
│  ├─ size < max? Yes → buffer                         │
│  └─ Return offset=1000 (fast path!)                  │
│      │                                                 │
│      │ (async flush triggers later)                   │
│      ▼                                                 │
│  Step 6: Storage Write                                 │
│  ├─ serialize(record)                                 │
│  ├─ segment.append(data)                              │
│  ├─ index.append(offset, position, size)             │
│  └─ fsync (if sync_on_write)                         │
│      │                                                 │
│      ▼                                                 │
│  Step 7: Replication                                   │
│  ├─ Select CopySet: [Node1, Node2, Node3]            │
│  ├─ Send to Node2 ─────────────► Node 2              │
│  ├─ Send to Node3 ─────────────► Node 3              │
│  ├─ Wait for W=2 ACKs                                │
│  └─ Quorum satisfied ✓                               │
│      │                                                 │
│      ▼                                                 │
│  Step 8: Response                                      │
│  └─ Return ProduceResponse{partition:2, offset:1000}  │
└─────────────────────────────────────────────────────────┘
  │
  ▼
Client receives offset=1000
```

---

## Read Path

### High-Level Read Flow

```
┌─────────┐
│ Client  │
└────┬────┘
     │ 1. consume(partition, offset)
     ▼
┌─────────────────┐
│  Pyralog Server    │
└────┬────────────┘
     │ 2. locate partition
     ▼
┌─────────────────┐
│  Partition      │
│  Manager        │
└────┬────────────┘
     │ 3. find segment
     ▼
┌─────────────────┐
│  Index          │ ───→ offset → position
└────┬────────────┘
     │ 4. read from storage
     ▼
┌─────────────────┐
│  Segment        │
│  (mmap or read) │
└────┬────────────┘
     │ 5. deserialize
     ▼
┌─────────────────┐
│  Record         │
└────┬────────────┘
     │ 6. return to client
     ▼
┌─────────┐
│ Client  │
└─────────┘
```

### Detailed Read Path Steps

#### Step 1: Client Request

```rust
// Client code
let records = client.consume(
    log_id,
    PartitionId::new(2),
    LogOffset::new(1000),
    max_records: 100,
).await?;
```

#### Step 2: Server Request Handling

```
┌──────────────────────────────────────┐
│         Pyralog Server                  │
├──────────────────────────────────────┤
│                                      │
│  Parse ConsumeRequest:               │
│    log_id: "events/user-actions"    │
│    partition: 2                      │
│    offset: 1000                      │
│    max_records: 100                  │
│                                      │
│  Validate:                           │
│    ✓ Log exists                      │
│    ✓ Partition exists                │
│    ✓ Has permission                  │
│                                      │
└──────────────┬───────────────────────┘
               │
               ▼
```

#### Step 3: Segment Location

```
┌────────────────────────────────────────────────┐
│         Segment Selection                      │
├────────────────────────────────────────────────┤
│                                                │
│  Partition 2 Segments:                         │
│  ┌──────────────────────────────────────────┐ │
│  │ 0000000000000000000.log (offsets 0-999) │ │
│  ├──────────────────────────────────────────┤ │
│  │ 0000000000001000000.log (1000-1999) ✓   │ │ ← Target
│  ├──────────────────────────────────────────┤ │
│  │ 0000000000002000000.log (2000-2999)     │ │
│  └──────────────────────────────────────────┘ │
│                                                │
│  Find segment containing offset 1000:         │
│    binary_search(segments, offset) → segment  │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
       Segment: 0000000000001000000.log
```

```rust
impl LogStorage {
    fn find_segment(&self, offset: LogOffset) -> Result<&Segment> {
        // Binary search through segments
        self.segments
            .binary_search_by(|seg| {
                if offset < seg.base_offset() {
                    Ordering::Greater
                } else if offset >= seg.base_offset() + seg.record_count() {
                    Ordering::Less
                } else {
                    Ordering::Equal
                }
            })
            .map(|idx| &self.segments[idx])
            .map_err(|_| PyralogError::InvalidOffset(offset))
    }
}
```

#### Step 4: Index Lookup

```
┌────────────────────────────────────────────────┐
│         Index Lookup                           │
├────────────────────────────────────────────────┤
│                                                │
│  Index File: 0000000000001000000.index        │
│                                                │
│  Sparse Index (every ~4KB):                    │
│  ┌───────────┬──────────┬───────┐            │
│  │  Offset   │ Position │ Size  │            │
│  ├───────────┼──────────┼───────┤            │
│  │  1000000  │    0     │  512  │            │
│  │  1000010  │  5120    │  256  │            │
│  │  1000020  │ 10240    │  384  │            │
│  │  ...      │  ...     │  ...  │            │
│  │  1000100  │ 51200    │  412  │ ← Target   │
│  └───────────┴──────────┴───────┘            │
│                                                │
│  Binary search: O(log n)                       │
│  offset=1000 → position=0, size=512           │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
       Position in segment: byte 0
```

**Index lookup algorithm**:

```rust
impl Index {
    pub fn lookup(&self, offset: LogOffset) -> Option<(u64, u32)> {
        // Binary search in sparse index
        let entries = self.entries.read();
        
        entries
            .binary_search_by_key(&offset.as_u64(), |entry| entry.offset)
            .ok()
            .map(|idx| {
                let entry = &entries[idx];
                (entry.position, entry.size)
            })
    }
}
```

#### Step 5: Storage Read

```
┌────────────────────────────────────────────────┐
│         Storage Read                           │
├────────────────────────────────────────────────┤
│                                                │
│  Two read paths:                               │
│                                                │
│  A) Memory-Mapped I/O (if enabled):            │
│     ┌─────────────────────────────────────┐   │
│     │  mmap region                        │   │
│     │  [address + position]               │   │
│     │   ↓                                 │   │
│     │  Direct memory access               │   │
│     │  Zero-copy!                         │   │
│     └─────────────────────────────────────┘   │
│                                                │
│  B) File I/O (fallback):                       │
│     ┌─────────────────────────────────────┐   │
│     │  file.seek(position)                │   │
│     │  file.read_exact(buffer, size)      │   │
│     │  Copy to memory                     │   │
│     └─────────────────────────────────────┘   │
│                                                │
│  Read data at position=0, size=512            │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
       Raw bytes (512 bytes)
```

**Memory-mapped read (fast path)**:

```rust
impl Segment {
    pub fn read(&self, offset: u64, length: usize) -> Result<Bytes> {
        // Try mmap first (zero-copy)
        if let Some(mmap) = self.mmap.read().as_ref() {
            let start = offset as usize;
            let end = start + length;
            return Ok(Bytes::copy_from_slice(&mmap[start..end]));
        }
        
        // Fallback to file I/O
        let mut file = self.file.write();
        let mut buffer = vec![0u8; length];
        
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut buffer)?;
        
        Ok(Bytes::from(buffer))
    }
}
```

#### Step 6: Deserialization

```
┌────────────────────────────────────────────────┐
│         Deserialization                        │
├────────────────────────────────────────────────┤
│                                                │
│  Raw bytes                                     │
│    ↓                                           │
│  bincode::deserialize()                        │
│    ↓                                           │
│  Record {                                      │
│    offset: 1000,                              │
│    epoch: 5,                                  │
│    timestamp: "2025-01-01T12:00:00Z",         │
│    key: Some(b"user-123"),                    │
│    value: b"order data",                      │
│    headers: [...],                            │
│  }                                             │
│                                                │
│  Validate:                                     │
│    ✓ CRC checksum                             │
│    ✓ Epoch is valid                           │
│    ✓ Offset matches                           │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
       Validated Record
```

#### Step 7: Return to Client

```
┌────────────────────────────────────────────────┐
│         Response                               │
├────────────────────────────────────────────────┤
│                                                │
│  ConsumeResponse {                             │
│    partition: 2,                               │
│    high_watermark: 1050,                       │
│    records: [                                  │
│      Record { offset: 1000, ... },            │
│      Record { offset: 1001, ... },            │
│      ...                                       │
│      Record { offset: 1099, ... },            │
│    ],                                          │
│    error: None,                                │
│  }                                             │
│                                                │
│  Serialize & send to client                    │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
       Client processes records
```

### Complete Read Path Diagram

```
Client
  │
  │ consume(partition=2, offset=1000, max=100)
  ▼
┌─────────────────────────────────────────────────────────┐
│ Pyralog Server                                             │
│                                                         │
│  Step 1: Request Validation                             │
│  ├─ Parse ConsumeRequest                               │
│  ├─ Check log exists                                   │
│  ├─ Check partition exists                             │
│  └─ Validate permissions                               │
│      │                                                 │
│      ▼                                                 │
│  Step 2: Segment Selection                             │
│  ├─ List segments for partition 2                     │
│  ├─ Binary search for offset 1000                     │
│  └─ Found: 0000000000001000000.log                    │
│      │                                                 │
│      ▼                                                 │
│  Step 3: Index Lookup                                  │
│  ├─ Load index: 0000000000001000000.index            │
│  ├─ Binary search for offset 1000                     │
│  └─ Found: position=0, size=512                       │
│      │                                                 │
│      ▼                                                 │
│  Step 4: Storage Read                                  │
│  ├─ Check if segment is mmap'd                        │
│  │   ↓                                                │
│  │   Yes → Zero-copy read from memory                │
│  │   No  → File I/O read                             │
│  │                                                    │
│  ├─ Read bytes at position=0, length=512             │
│  └─ Got raw data                                      │
│      │                                                 │
│      ▼                                                 │
│  Step 5: Deserialization                               │
│  ├─ bincode::deserialize(bytes)                       │
│  ├─ Validate CRC checksum                             │
│  ├─ Validate epoch                                    │
│  └─ Record reconstructed                              │
│      │                                                 │
│      ▼                                                 │
│  Step 6: Repeat for next records                       │
│  ├─ Continue reading offsets 1001-1099                │
│  ├─ Or until max_records reached                      │
│  └─ Or until end of segment                           │
│      │                                                 │
│      ▼                                                 │
│  Step 7: Build Response                                │
│  ├─ Collect all records                               │
│  ├─ Add high_watermark                                │
│  └─ Serialize response                                │
└─────────────────────────────────────────────────────────┘
  │
  ▼
Client receives records[0-99]
```

---

## Batch Write Path

### Batch vs Single Record

```
Single Record Write:
  Client ─→ Server ─→ Storage
  (1 network RTT per record)

Batch Write:
  Client ─→ Server ─→ Storage
  (1 network RTT for N records)

Efficiency gain: N records for cost of 1 RTT
```

### Batch Write Flow

```
┌────────────────────────────────────────────────┐
│         Client Batching                        │
├────────────────────────────────────────────────┤
│                                                │
│  Buffer: Vec<Record>                           │
│  ┌─────────────────────────────────────────┐  │
│  │ record1: {key:"u1", val:"data1"}        │  │
│  │ record2: {key:"u2", val:"data2"}        │  │
│  │ ...                                     │  │
│  │ record100: {key:"u100", val:"data100"}  │  │
│  └─────────────────────────────────────────┘  │
│                                                │
│  Flush conditions:                             │
│  ├─ Count >= 100                              │
│  ├─ Size >= 1MB                               │
│  └─ Time >= 100ms                             │
│                                                │
│  client.produce_batch(log_id, buffer)         │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
┌────────────────────────────────────────────────┐
│         Server Batch Processing                │
├────────────────────────────────────────────────┤
│                                                │
│  Receive RecordBatch                           │
│    │                                           │
│    ├─ Assign sequential offsets               │
│    │  record1.offset = 1000                   │
│    │  record2.offset = 1001                   │
│    │  ...                                      │
│    │  record100.offset = 1099                 │
│    │                                           │
│    ├─ All get same epoch: 5                   │
│    │                                           │
│    └─ Write as single I/O operation           │
│        ↓                                       │
│  ┌──────────────────────────────────────┐    │
│  │  Storage writes 100 records          │    │
│  │  in one segment.append() call        │    │
│  │  → Single fsync                      │    │
│  │  → Amortized overhead                │    │
│  └──────────────────────────────────────┘    │
│                                                │
└────────────────┬───────────────────────────────┘
                 │
                 ▼
       100 records written, 1 RTT
```

### Batch Performance Comparison

```
Single Record (1000 records):
─────────────────────────────────────────
Write 1: 1ms  ──┐
Write 2: 1ms    │
Write 3: 1ms    │ 1000 x 1ms = 1000ms total
...             │
Write 1000: 1ms ┘
─────────────────────────────────────────

Batch Write (1000 records, batch size 100):
─────────────────────────────────────────
Batch 1 (100): 5ms  ──┐
Batch 2 (100): 5ms    │
...                   │ 10 x 5ms = 50ms total
Batch 10 (100): 5ms ──┘
─────────────────────────────────────────

Speedup: 20x faster!
```

---

## Replication Flow

### Leader-Based Replication

```
┌──────────────────────────────────────────────────────────┐
│                  Replication Flow                        │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Client                                                  │
│    │                                                     │
│    │ produce(record)                                     │
│    ▼                                                     │
│  ┌────────────────────────────────────────────────────┐ │
│  │ Leader (Node 1) - Partition 2                      │ │
│  │                                                     │ │
│  │  1. Write to local storage                         │ │
│  │     offset = 1000                                  │ │
│  │                                                     │ │
│  │  2. Send AppendEntries to followers                │ │
│  │     ├─────────────────┬─────────────────┐         │ │
│  │     │                 │                 │         │ │
│  └─────┼─────────────────┼─────────────────┼─────────┘ │
│        │                 │                 │           │
│        ▼                 ▼                 ▼           │
│  ┌─────────┐       ┌─────────┐     ┌─────────┐       │
│  │ Node 2  │       │ Node 3  │     │ Node 4  │       │
│  │ Follower│       │ Follower│     │ Follower│       │
│  │         │       │         │     │         │       │
│  │ Write   │       │ Write   │     │ (slow)  │       │
│  │ offset  │       │ offset  │     │         │       │
│  │ 1000    │       │ 1000    │     │         │       │
│  │         │       │         │     │         │       │
│  │ ACK ✓   │       │ ACK ✓   │     │ ...     │       │
│  └────┬────┘       └────┬────┘     └─────────┘       │
│       │                 │                             │
│       └────────┬────────┘                             │
│                │                                       │
│  3. Quorum satisfied (W=2, got 2 ACKs)               │
│                │                                       │
│                ▼                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ Leader commits offset 1000                     │  │
│  │ Returns success to client                      │  │
│  └────────────────────────────────────────────────┘  │
│                │                                       │
│                ▼                                       │
│  4. Later, Node 4 catches up (async)                  │
│                                                        │
└────────────────────────────────────────────────────────┘
```

### ISR (In-Sync Replicas) Management

```
┌──────────────────────────────────────────────────────────┐
│         ISR Tracking                                     │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Partition 2 State:                                      │
│  ┌────────────────────────────────────────────────────┐ │
│  │ Leader: Node 1                                     │ │
│  │ High Watermark: 1000                               │ │
│  │                                                     │ │
│  │ Replicas:                                          │ │
│  │ ┌──────┬────────┬─────────┬──────────┐           │ │
│  │ │ Node │ Offset │ Lag     │ ISR?     │           │ │
│  │ ├──────┼────────┼─────────┼──────────┤           │ │
│  │ │  1   │ 1000   │ 0       │ ✓ Leader │           │ │
│  │ │  2   │ 1000   │ 0       │ ✓ Yes    │           │ │
│  │ │  3   │ 998    │ 2       │ ✓ Yes    │           │ │
│  │ │  4   │ 850    │ 150     │ ✗ No     │           │ │
│  │ └──────┴────────┴─────────┴──────────┘           │ │
│  │                                                     │ │
│  │ ISR = [Node 1, Node 2, Node 3]                    │ │
│  │                                                     │ │
│  │ ISR threshold: lag < 1000                          │ │
│  │ Node 4 is too far behind → removed from ISR       │ │
│  └────────────────────────────────────────────────────┘ │
│                                                          │
│  Quorum check uses ISR:                                  │
│    write_quorum = 2                                      │
│    ISR.len() = 3 ≥ 2  ✓ Can accept writes              │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## Failure Scenarios

### Scenario 1: Leader Failure During Write

```
Time: T0
┌─────────────────────────────────────────────┐
│  Client sends write to Leader (Node 1)     │
│                                             │
│  Node 1 (Leader) ──┐                       │
│  Node 2 (Follower) │  Epoch 5              │
│  Node 3 (Follower) ┘                       │
└─────────────────────────────────────────────┘

Time: T1
┌─────────────────────────────────────────────┐
│  Node 1 writes locally, starts replication │
│                                             │
│  Node 1: offset=1000 ✓                     │
│  Node 2: ───────────► writing...           │
│  Node 3: ───────────► writing...           │
│                                             │
│  💥 Node 1 crashes!                         │
└─────────────────────────────────────────────┘

Time: T2
┌─────────────────────────────────────────────┐
│  Node 1: ✗ Down                             │
│  Node 2: timeout, start election            │
│  Node 3: timeout, start election            │
│                                             │
│  Election:                                  │
│    Node 2 votes for Node 3                 │
│    Node 3 votes for self                   │
│    Node 3 wins (has latest data)           │
│                                             │
│  Node 3 becomes Leader with Epoch 6        │
└─────────────────────────────────────────────┘

Time: T3
┌─────────────────────────────────────────────┐
│  Node 3 (New Leader, Epoch 6)              │
│                                             │
│  Seal old epoch 5                          │
│  Check if offset 1000 was committed:       │
│    - Node 2: has offset 1000? Check...    │
│    - If yes: keep it                       │
│    - If no: discard from Node 1            │
│                                             │
│  Client request:                            │
│    - Error: Leader changed                 │
│    - Retry with new leader (Node 3)        │
└─────────────────────────────────────────────┘
```

**Epoch prevents split-brain**:
- Node 1's write had epoch 5
- Node 3 has epoch 6
- If Node 1 comes back, it can't write with old epoch
- Clients see epoch mismatch, redirect to new leader

### Scenario 2: Follower Slow/Unavailable

```
┌──────────────────────────────────────────────┐
│  Leader (Node 1) - Quorum W=2, RF=3         │
│                                              │
│  Followers:                                  │
│    Node 2: ✓ Fast, in ISR                   │
│    Node 3: ⚠️  Slow, dropping out of ISR     │
│                                              │
│  Write arrives:                              │
│    1. Write locally ✓                       │
│    2. Send to Node 2 ──────► ACK ✓          │
│    3. Send to Node 3 ──────► (timeout...)   │
│                                              │
│  Quorum satisfied: 2 ACKs (self + Node 2)   │
│  Return success to client ✓                 │
│                                              │
│  Node 3:                                     │
│    - Lag increases                          │
│    - Eventually catches up (async)          │
│    - Or removed from ISR if lag > threshold │
└──────────────────────────────────────────────┘
```

### Scenario 3: Network Partition

```
Partition occurs:
┌────────────────┐     │     ┌────────────────┐
│   Node 1       │     │     │   Node 2       │
│   (Leader)     │     │     │   (Follower)   │
│                │     ╳     │                │
│   Node 3       │     │     │                │
│   (Follower)   │     │     │                │
└────────────────┘     │     └────────────────┘
  Majority (2/3)       │       Minority (1/3)

With W=2, R=2:
─────────────────────────────────────────────
Left partition (Nodes 1,3):
  - Has majority ✓
  - Can elect leader ✓
  - Can accept writes ✓
  - Can serve reads ✓

Right partition (Node 2):
  - No majority ✗
  - Cannot be leader ✗
  - Cannot accept writes ✗
  - Cannot serve reads ✗

Result: System continues on left, unavailable on right
        CP behavior: Consistency preserved
─────────────────────────────────────────────

With W=1, R=1:
─────────────────────────────────────────────
Left partition:
  - Can write ✓
  - Can read ✓

Right partition:
  - Can write ✓  (but with epoch protection)
  - Can read ✓   (but may be stale)

Epoch system prevents true split-brain:
  - Old leader (right) has epoch 5
  - New leader (left) has epoch 6
  - Writes from old leader rejected
  - When partition heals, old writes discarded

Result: AP-like availability, CP-like safety
─────────────────────────────────────────────
```

---

## Performance Optimizations

### 1. Write Cache

```
Without Cache:
─────────────────────────────────────────
Client write → Storage → fsync() → Response
                ↓
           ~10ms latency

With Cache:
─────────────────────────────────────────
Client write → Cache → Response (immediate)
                ↓
           ~0.5ms latency
                │
                │ (async, later)
                ▼
           Storage → fsync()
```

### 2. Memory-Mapped I/O

```
Regular File I/O:
─────────────────────────────────────────
seek(position) → system call
read(buffer)   → system call → copy data
Total: 2 syscalls + 1 copy

Memory-Mapped I/O:
─────────────────────────────────────────
&mmap[position] → direct memory access
Total: 0 syscalls + 0 copies (zero-copy!)

Speedup: 2-3x for reads
```

### 3. Batching

```
Impact on throughput:

Single records:
1000 records × 1ms each = 1000ms
Throughput: 1,000 records/sec

Batches of 100:
10 batches × 5ms each = 50ms
Throughput: 20,000 records/sec

Speedup: 20x
```

### 4. Parallel Replication

```
Sequential:
────────────────────────────────────
Replica 1: [====] 10ms
Replica 2:       [====] 10ms
Replica 3:             [====] 10ms
Total: 30ms

Parallel:
────────────────────────────────────
Replica 1: [====]
Replica 2: [====]  All at once
Replica 3: [====]
Total: 10ms

Speedup: 3x
```

---

## Smart Client Architecture

### The Problem: Naive Proxy Model

A naive approach would have clients connect to any server, which then proxies requests to the correct leader:

```
┌────────────────────────────────────────────────┐
│         NAIVE PROXY MODEL ❌                   │
├────────────────────────────────────────────────┤
│                                                │
│  Client                                        │
│    │                                           │
│    │ 1. Write request                         │
│    ▼                                           │
│  ┌──────────────┐                             │
│  │  Any Server  │ ← Client connects here      │
│  │  (Node 2)    │                             │
│  └──────┬───────┘                             │
│         │                                      │
│         │ 2. Proxy to actual leader           │
│         ▼                                      │
│  ┌──────────────┐                             │
│  │   Leader     │ ← Extra hop!                │
│  │  (Node 5)    │                             │
│  └──────┬───────┘                             │
│         │                                      │
│         │ 3. Replicate                        │
│         ▼                                      │
│    Followers                                   │
│                                                │
│  Problems:                                     │
│    ❌ Extra network hop (2x latency)          │
│    ❌ Proxy node becomes bottleneck            │
│    ❌ Wastes server resources on routing       │
│    ❌ Doesn't scale well                       │
│                                                │
└────────────────────────────────────────────────┘
```

### The Solution: Smart Client Pattern

Pyralog uses the **smart client pattern** (like Kafka) where clients fetch metadata and connect directly to the correct leader:

```
┌────────────────────────────────────────────────┐
│         SMART CLIENT MODEL ✅                  │
├────────────────────────────────────────────────┤
│                                                │
│  Phase 1: Metadata Discovery (once)            │
│  ─────────────────────────────────────         │
│  Client                                        │
│    │                                           │
│    │ 1. MetadataRequest                       │
│    ▼                                           │
│  ┌──────────────┐                             │
│  │  Any Server  │                             │
│  │  (Node 2)    │                             │
│  └──────┬───────┘                             │
│         │                                      │
│         │ 2. MetadataResponse                 │
│         │    {                                 │
│         │      partition_0: leader=Node5,      │
│         │      partition_1: leader=Node3,      │
│         │      partition_2: leader=Node1       │
│         │    }                                 │
│         ▼                                      │
│  Client caches metadata locally                │
│                                                │
│  Phase 2: Direct Write (hot path)             │
│  ─────────────────────────────────────         │
│  Client                                        │
│    │                                           │
│    │ hash(key) % 3 = 0 → partition 0          │
│    │ partition 0 leader = Node 5              │
│    │                                           │
│    │ 3. Write directly to Node 5! ✅          │
│    ▼                                           │
│  ┌──────────────┐                             │
│  │   Leader     │ ← Direct connection!        │
│  │  (Node 5)    │                             │
│  └──────┬───────┘                             │
│         │                                      │
│         │ 4. Replicate                        │
│         ▼                                      │
│    Followers                                   │
│                                                │
│  Benefits:                                     │
│    ✅ One network hop (no proxy)              │
│    ✅ No server-side routing overhead          │
│    ✅ Client-side load balancing               │
│    ✅ Scales perfectly with cluster size       │
│                                                │
└────────────────────────────────────────────────┘
```

### Metadata Request/Response Protocol

```rust
// Client requests metadata
#[derive(Serialize, Deserialize)]
pub struct MetadataRequest {
    pub log_ids: Vec<LogId>,  // Which logs to get metadata for
}

// Server responds with partition topology
#[derive(Serialize, Deserialize)]
pub struct MetadataResponse {
    pub logs: Vec<LogMetadata>,
    pub brokers: Vec<BrokerMetadata>,
}

#[derive(Serialize, Deserialize)]
pub struct LogMetadata {
    pub log_id: LogId,
    pub partitions: Vec<PartitionMetadata>,
}

#[derive(Serialize, Deserialize)]
pub struct PartitionMetadata {
    pub partition_id: PartitionId,
    pub leader: NodeId,           // Who is the leader
    pub replicas: Vec<NodeId>,    // All replicas
    pub isr: Vec<NodeId>,         // In-Sync Replicas
}

#[derive(Serialize, Deserialize)]
pub struct BrokerMetadata {
    pub node_id: NodeId,
    pub host: String,
    pub port: u16,
    pub rack: Option<String>,     // For rack-aware clients
}
```

**Example Response:**

```json
{
  "logs": [{
    "log_id": "events/user-actions",
    "partitions": [
      {
        "partition_id": 0,
        "leader": 5,
        "replicas": [5, 2, 3],
        "isr": [5, 2, 3]
      },
      {
        "partition_id": 1,
        "leader": 3,
        "replicas": [3, 1, 5],
        "isr": [3, 1]
      },
      {
        "partition_id": 2,
        "leader": 1,
        "replicas": [1, 5, 2],
        "isr": [1, 5, 2]
      }
    ]
  }],
  "brokers": [
    { "node_id": 1, "host": "node1.pyralog.io", "port": 9092 },
    { "node_id": 2, "host": "node2.pyralog.io", "port": 9092 },
    { "node_id": 3, "host": "node3.pyralog.io", "port": 9092 },
    { "node_id": 5, "host": "node5.pyralog.io", "port": 9092 }
  ]
}
```

### Client-Side Implementation

```rust
pub struct PyralogClient {
    // Bootstrap servers (initial connection)
    bootstrap_servers: Vec<String>,
    
    // Cached metadata
    metadata_cache: Arc<RwLock<MetadataCache>>,
    
    // Connections to each node
    connections: Arc<RwLock<HashMap<NodeId, Connection>>>,
    
    // Partitioning strategy
    partitioner: Box<dyn Partitioner>,
}

impl PyralogClient {
    pub async fn produce(
        &self,
        log_id: LogId,
        key: Option<Bytes>,
        value: Bytes,
    ) -> Result<LogOffset> {
        // 1. Calculate partition (client-side!)
        let partition = self.partitioner.partition(&key, &log_id)?;
        
        // 2. Get leader from cached metadata
        let leader = self.get_leader(&log_id, partition).await?;
        
        // 3. Send directly to leader
        let record = Record::new(key, value);
        
        match self.send_to_node(leader, record).await {
            Ok(offset) => Ok(offset),
            
            // 4. Handle leader change
            Err(PyralogError::NotLeader(new_leader)) => {
                // Invalidate cache
                self.invalidate_metadata(&log_id).await;
                
                // Refresh metadata
                self.refresh_metadata(&log_id).await?;
                
                // Retry with new leader
                self.send_to_node(new_leader, record).await
            }
            
            Err(e) => Err(e),
        }
    }
    
    async fn get_leader(
        &self,
        log_id: &LogId,
        partition: PartitionId,
    ) -> Result<NodeId> {
        // Try cache first
        if let Some(leader) = self.metadata_cache.read().get_leader(log_id, partition) {
            return Ok(leader);
        }
        
        // Cache miss - refresh metadata
        self.refresh_metadata(log_id).await?;
        
        self.metadata_cache
            .read()
            .get_leader(log_id, partition)
            .ok_or(PyralogError::LeaderNotAvailable)
    }
    
    async fn refresh_metadata(&self, log_id: &LogId) -> Result<()> {
        // Try each bootstrap server until one succeeds
        for server in &self.bootstrap_servers {
            let request = MetadataRequest {
                log_ids: vec![log_id.clone()],
            };
            
            match self.send_metadata_request(server, request).await {
                Ok(metadata) => {
                    // Update cache
                    self.metadata_cache.write().update(metadata);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to fetch metadata from {}: {}", server, e);
                    continue;
                }
            }
        }
        
        Err(PyralogError::NoAvailableServers)
    }
}
```

### Complete Flow Example

```
Step-by-Step: Client Writes a Record
═════════════════════════════════════════════════

T0: Client startup
    ├─ Connect to bootstrap servers: ["node1:9092", "node2:9092"]
    └─ Metadata cache: empty

T1: First write
    ├─ Client: produce(log="events", key="user-123", value="...")
    ├─ Calculate partition: hash("user-123") % 3 = 0
    ├─ Check cache: partition 0 leader = ? (cache miss)
    └─ Need to fetch metadata

T2: Metadata request
    ├─ Client → node1:9092: MetadataRequest{log="events"}
    └─ Node1 responds:
        {
          partition_0: leader=node5,
          partition_1: leader=node3,
          partition_2: leader=node1
        }

T3: Cache metadata
    ├─ Update metadata_cache
    ├─ partition_0 → node5
    ├─ partition_1 → node3
    └─ partition_2 → node1

T4: Direct write
    ├─ partition 0 leader = node5 (from cache)
    ├─ Open connection to node5:9092
    └─ Client → node5: ProduceRequest(record)

T5: Leader processes
    ├─ node5 (leader) receives request
    ├─ Assigns epoch and offset
    ├─ Writes to storage
    ├─ Replicates to followers
    └─ Returns ProduceResponse{offset=1000}

T6: Client receives response
    └─ Success! offset=1000

T7: Subsequent writes (fast path!)
    ├─ Client: produce(log="events", key="user-456", value="...")
    ├─ Calculate partition: hash("user-456") % 3 = 1
    ├─ Check cache: partition 1 leader = node3 ✅ (cache hit!)
    └─ Client → node3 directly! (no metadata fetch needed)
```

### Handling Leader Changes

```
Scenario: Leader Failover During Write
═════════════════════════════════════════════════

T0: Client has cached metadata
    partition 0 leader = node5

T1: Client sends write to node5
    Client → node5: ProduceRequest

T2: Node5 has failed, node3 is new leader
    (Client doesn't know yet)

T3: Connection timeout or connection refused
    Error: ConnectionError

T4: Client invalidates cache
    Remove cached metadata for partition 0

T5: Client refreshes metadata
    Client → node1: MetadataRequest
    Response: partition 0 leader = node3 (new!)

T6: Client updates cache
    partition 0 leader = node3

T7: Client retries write
    Client → node3: ProduceRequest
    Success! ✅

Cost of failover: 1 extra RTT (metadata refresh)
Frequency: Rare (only on leader changes)
```

### Read Strategy with Metadata

Clients can use metadata to implement different read strategies:

```rust
impl PyralogClient {
    pub async fn consume(
        &self,
        log_id: LogId,
        partition: PartitionId,
        offset: LogOffset,
    ) -> Result<Vec<Record>> {
        let metadata = self.get_partition_metadata(&log_id, partition).await?;
        
        // Choose read strategy
        let node = match self.config.read_policy {
            // 1. Leader reads (strong consistency)
            ReadPolicy::LeaderOnly => {
                metadata.leader
            }
            
            // 2. Any replica (eventual consistency, best latency)
            ReadPolicy::AnyReplica => {
                metadata.replicas
                    .choose(&mut rand::thread_rng())
                    .copied()
                    .unwrap()
            }
            
            // 3. ISR only (consistent with recent writes)
            ReadPolicy::InSyncReplica => {
                metadata.isr
                    .choose(&mut rand::thread_rng())
                    .copied()
                    .unwrap_or(metadata.leader)
            }
            
            // 4. Nearest replica (datacenter-aware)
            ReadPolicy::NearestReplica => {
                self.choose_nearest_replica(&metadata)?
            }
        };
        
        // Send read request directly to chosen node
        self.read_from_node(node, log_id, partition, offset).await
    }
    
    fn choose_nearest_replica(
        &self,
        metadata: &PartitionMetadata,
    ) -> Result<NodeId> {
        // Use rack/datacenter info from broker metadata
        let client_rack = self.config.rack.as_ref();
        
        for &replica in &metadata.replicas {
            let broker = self.get_broker_metadata(replica)?;
            if broker.rack.as_ref() == client_rack {
                return Ok(replica);
            }
        }
        
        // Fall back to leader
        Ok(metadata.leader)
    }
}
```

### Load Balancing Benefits

```
┌────────────────────────────────────────────────┐
│         Client-Side Load Balancing             │
├────────────────────────────────────────────────┤
│                                                │
│  10 Clients, 3 Partitions, 3 Nodes             │
│                                                │
│  Client A: key="user-1" → partition 0 → node1  │
│  Client B: key="user-2" → partition 1 → node2  │
│  Client C: key="user-3" → partition 2 → node3  │
│  Client D: key="user-4" → partition 0 → node1  │
│  Client E: key="user-5" → partition 1 → node2  │
│  Client F: key="user-6" → partition 2 → node3  │
│  ...                                           │
│                                                │
│  Result: Load naturally distributed!           │
│    node1: 33% of traffic                       │
│    node2: 33% of traffic                       │
│    node3: 33% of traffic                       │
│                                                │
│  No explicit load balancer needed! ✅          │
│                                                │
└────────────────────────────────────────────────┘
```

### Performance Comparison

```
Proxy Model (2 hops):
─────────────────────────────────────────
Client → Proxy → Leader → Replicas

Latency breakdown:
  Client → Proxy:    1ms
  Proxy → Leader:    1ms
  Leader → Replicas: 10ms
  Leader → Proxy:    1ms
  Proxy → Client:    1ms
  ────────────────────────
  Total:             14ms

Smart Client Model (1 hop):
─────────────────────────────────────────
Client → Leader → Replicas

Latency breakdown:
  Client → Leader:   1ms
  Leader → Replicas: 10ms
  Leader → Client:   1ms
  ────────────────────────
  Total:             12ms

Improvement: 14% faster (2ms saved)

With metadata caching:
─────────────────────────────────────────
Metadata fetch: Once per 5 minutes
  Cost: 1-2ms

Per-write cost: 0ms (using cache)

Amortized overhead: ~0.0001ms per write

Result: Essentially free! ✅
```

### Metadata Refresh Strategies

```rust
// 1. On-demand refresh (when needed)
if let Err(PyralogError::NotLeader(_)) = result {
    self.refresh_metadata(log_id).await?;
}

// 2. Periodic refresh (proactive)
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        if let Err(e) = client.refresh_all_metadata().await {
            warn!("Periodic metadata refresh failed: {}", e);
        }
    }
});

// 3. Exponential backoff on errors
let mut backoff = Duration::from_millis(100);
loop {
    match self.refresh_metadata(log_id).await {
        Ok(_) => break,
        Err(_) => {
            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, Duration::from_secs(10));
        }
    }
}

// 4. Subscription-based (push model, advanced)
// Server pushes metadata updates to clients
client.subscribe_to_metadata_updates(|metadata| {
    cache.update(metadata);
});
```

### Comparison with Other Systems

```
┌─────────────────────────────────────────────────────┐
│   Kafka (Smart Client)                              │
├─────────────────────────────────────────────────────┤
│  • Clients fetch metadata                           │
│  • Direct connection to partition leaders           │
│  • Client-side load balancing                       │
│  • Scales to 1000s of clients                       │
│  ✅ Pyralog uses this model                            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│   Cassandra (Smart Client)                          │
├─────────────────────────────────────────────────────┤
│  • Clients know token ring topology                 │
│  • Route directly to coordinator                    │
│  • No leader, any node can handle writes            │
│  • Pyralog: Similar metadata approach, but leader-based│
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│   MongoDB (Proxy Model)                             │
├─────────────────────────────────────────────────────┤
│  • mongos routers proxy requests                    │
│  • Clients connect to mongos, not shards directly   │
│  • Extra hop, but simpler client                    │
│  ❌ Pyralog avoids this model (performance)            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│   Redis Cluster (Smart Client)                      │
├─────────────────────────────────────────────────────┤
│  • Clients learn slot → node mapping                │
│  • Direct connection to slot master                 │
│  • MOVED/ASK redirects for topology changes         │
│  • Similar to Pyralog's NotLeader error handling       │
└─────────────────────────────────────────────────────┘
```

### Smart Client Advantages Summary

| Aspect | Smart Client | Proxy Model |
|--------|--------------|-------------|
| Network hops | 1 (direct) | 2 (via proxy) |
| Latency | Lower ✅ | Higher ❌ |
| Server CPU | Lower ✅ | Higher (routing) ❌ |
| Scalability | Better ✅ | Limited ❌ |
| Client complexity | Higher ❌ | Lower ✅ |
| Load balancing | Built-in ✅ | Needs LB ❌ |
| Failure handling | Client retries | Proxy handles |

**Pyralog uses smart clients because:**
1. Performance is critical (every ms matters)
2. Client libraries handle complexity
3. Scales better with cluster size
4. Industry standard (Kafka, Cassandra)
5. No single point of failure

---

## Summary

### Write Path Key Points

1. **Fast path**: Client → Cache → Response (sub-ms)
2. **Slow path**: Cache → Storage → Replication → Commit
3. **Epoch assignment**: Every write gets current epoch
4. **Quorum waiting**: Only wait for W acknowledgments
5. **Parallel replication**: Send to all replicas simultaneously

### Read Path Key Points

1. **Fast path**: mmap → zero-copy read (sub-ms)
2. **Index lookup**: Binary search O(log n)
3. **Sequential friendly**: Reading multiple records is efficient
4. **No replication needed**: Can read from any replica

### Performance Characteristics

| Operation | Latency (p99) | Notes |
|-----------|---------------|-------|
| Write (cached) | < 1ms | Fast path, write cache enabled |
| Write (sync) | ~10ms | fsync on every write |
| Read (mmap) | < 0.5ms | Zero-copy memory-mapped |
| Read (file I/O) | ~2ms | Regular file operations |
| Batch write (100) | ~5ms | Amortized overhead |

---

**For more details, see:**
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning
- [EPOCHS.md](EPOCHS.md) - Epoch system details
- [CAP_THEOREM.md](CAP_THEOREM.md) - Consistency tradeoffs

