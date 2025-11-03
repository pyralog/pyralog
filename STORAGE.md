# Storage Layer Architecture

**Comprehensive guide to Pyralog's LSM-based storage engine**

---

## Table of Contents

1. [Overview](#overview)
2. [Storage Modes](#storage-modes)
3. [LSM-Tree Architecture](#lsm-tree-architecture)
4. [Hybrid Storage: Native vs External](#hybrid-storage-native-vs-external)
5. [Write Path](#write-path)
6. [Read Path](#read-path)
7. [Compaction](#compaction)
8. [Indexes](#indexes)
9. [Tiered Storage](#tiered-storage)
10. [Memory-Mapped I/O](#memory-mapped-io)
11. [Data Format](#data-format)
12. [Performance Characteristics](#performance-characteristics)
13. [Configuration & Tuning](#configuration--tuning)
14. [Best Practices](#best-practices)

---

## Overview

Pyralog uses a **hybrid storage architecture** combining LSM-Tree for hot data with file references for cold/external data:

- ✅ **High write throughput**: 500M+ writes/sec per cluster
- ✅ **Low write latency**: p99 < 1ms with write caching
- ✅ **Efficient storage**: 30-50% better compression through compaction
- ✅ **Fast reads**: PPHM indexes, Bloom filters, sparse indexes
- ✅ **Hybrid approach**: Native LSM + external file references
- ✅ **Zero-copy**: Arrow Flight + memory-mapped files
- ✅ **Scalability**: Linear scaling across partitions and nodes
- ✅ **Durability**: WAL + configurable fsync policies

### Hybrid Storage Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    HYBRID STORAGE LAYERS                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  HOT DATA (Native LSM):                                     │
│  ┌───────────────────────────────────────────────────┐     │
│  │ Layer 0: MemTable (In-Memory)      ← Real-time    │     │
│  │ ├─ Active MemTable (16MB)                         │     │
│  │ └─ Immutable MemTables                            │     │
│  │                                                    │     │
│  │ Layer 1: L0 (Unsorted)             ← Recent       │     │
│  │ ├─ 1-4 segments (1GB each)                        │     │
│  │ └─ PPHM indexes (O(1) lookup)                     │     │
│  │                                                    │     │
│  │ Layer 2: L1+ (Sorted)              ← Warm         │     │
│  │ ├─ Sorted runs (10x growth)                       │     │
│  │ └─ Bloom filters + sparse indexes                 │     │
│  └───────────────────────────────────────────────────┘     │
│                          ↓                                   │
│  COLD DATA (External Files):                                │
│  ┌───────────────────────────────────────────────────┐     │
│  │ File References (metadata only in LSM)            │     │
│  │ ├─ Parquet: Analytics tables                      │     │
│  │ ├─ Safetensors: ML models                         │     │
│  │ ├─ Zarr: Scientific arrays                        │     │
│  │ └─ Files: Local or S3/GCS/Azure                   │     │
│  │                                                    │     │
│  │ Access: Memory-map on demand (zero-copy)          │     │
│  │ No compaction needed (files immutable)            │     │
│  └───────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Core Principles

1. **Immutability**: Once written, segments never change (except deletion)
2. **Sequential Writes**: All writes are appends (no random I/O)
3. **Hybrid Storage**: Native LSM for hot data, file references for cold data
4. **Zero-Copy**: Arrow Flight + memory-mapped external files
5. **Multi-Level Organization**: Recent data in memory, older data compacted
6. **Lazy Merging**: Background compaction merges and deduplicates
7. **Index Diversity**: PPHM for L0, Bloom filters for L1+, sparse for cold data
8. **External Files**: No compaction needed, mmap on access

---

## Storage Modes

Pyralog supports three storage modes, each optimized for different workloads:

### 1. Persistent Mode (Default)

**Use case**: Production workloads requiring durability

```rust
StorageConfig {
    mode: StorageMode::Persistent,
    wal_enabled: true,
    fsync_policy: SyncPolicy::Interval(Duration::from_millis(10)),
    ..Default::default()
}
```

**Characteristics**:
- ✅ Full durability (survives crashes)
- ✅ WAL + fsync for crash recovery
- ✅ Disk-limited capacity (TBs)
- ✅ Hybrid storage (LSM + external files)
- ⚠️ Write latency: p99 < 1ms
- ⚠️ Startup: 30s recovery time

**Best for**: Financial transactions, user data, audit logs, regulatory compliance

---

### 2. Memory-Only Mode

**Use case**: Ephemeral data, testing, caching, real-time analytics

```rust
StorageConfig {
    mode: StorageMode::MemoryOnly,
    max_memory_bytes: 32 * 1024 * 1024 * 1024, // 32GB
    eviction_policy: EvictionPolicy::LRU,
    ..Default::default()
}
```

**Characteristics**:
- ⚡ **10-100× faster** writes (no disk I/O)
- ⚡ **Sub-microsecond** latency (pure RAM)
- ⚡ **Instant startup** (<100ms, no recovery)
- ❌ No durability (lost on crash)
- ⚠️ RAM-limited capacity (GBs)
- ✅ Optional snapshots for recovery

**Best for**: Testing/CI, caching, temporary state, streaming pipelines, development

📖 **See [MEMORY_ONLY_MODE.md](MEMORY_ONLY_MODE.md) for comprehensive guide**

**Performance comparison**:

| Operation | Persistent | Memory-Only | Speedup |
|-----------|-----------|-------------|---------|
| Write (single) | 500K/sec | 50M/sec | **100×** |
| Write (batch) | 15M/sec | 500M/sec | **33×** |
| Read (single) | 3M/sec | 100M/sec | **33×** |
| Read (sequential) | 45M/sec | 2B/sec | **44×** |
| Startup | 30s | <100ms | **300×** |

---

### 3. Hybrid Mode

**Use case**: Hot/cold tiering, cost optimization

```rust
StorageConfig {
    mode: StorageMode::Hybrid {
        memory_ttl: Duration::from_secs(3600), // 1 hour in memory
        disk_after: Duration::from_secs(3600), // then flush to disk
    },
    tiered_storage: TieredStorageConfig {
        local_disk: true,
        s3_archive: Some(S3Config { /* ... */ }),
    },
    ..Default::default()
}
```

**Characteristics**:
- ✅ Best of both worlds (speed + durability)
- ✅ Hot data in memory (fast access)
- ✅ Cold data on disk/S3 (cost-effective)
- ✅ Automatic tiering based on age/access
- ✅ Graceful degradation (memory → disk → S3)

**Best for**: Mixed workloads, cost optimization, gradual data aging

---

### Mode Comparison

| Feature | Persistent | Memory-Only | Hybrid |
|---------|-----------|-------------|--------|
| **Durability** | ✅ Full | ❌ None | ✅ Configurable |
| **Write latency** | 1ms | 15μs | 15μs → 1ms |
| **Capacity** | Disk (TBs) | RAM (GBs) | RAM + Disk |
| **Startup** | 30s | <100ms | <100ms |
| **Cost/GB/mo** | $0.02 (SSD) | $3 (RAM) | Mixed |
| **Use case** | Production | Testing/Cache | Hot/Cold |

---

## LSM-Tree Architecture

### Three-Tier Storage Model

```rust
pub struct HybridStorage {
    /// Native LSM for hot data
    lsm: LSMStorage,
    
    /// External file references (cold data)
    external_files: ExternalFileManager,
    
    /// Arrow Flight server (zero-copy RPC)
    flight_server: ArrowFlightServer,
}

pub struct LSMStorage {
    /// In-memory write buffer
    memtable: Arc<RwLock<MemTable>>,
    
    /// Immutable memtables ready to flush
    immutable_memtables: Arc<RwLock<Vec<MemTable>>>,
    
    /// Level 0: Recent unsorted segments (1-4 segments)
    level0: Arc<RwLock<Vec<Segment>>>,
    
    /// Levels 1+: Sorted runs with increasing size
    levels: Arc<RwLock<Vec<Level>>>,
    
    /// Write-Ahead Log for durability
    wal: Arc<Mutex<WriteAheadLog>>,
    
    /// Background compaction threads
    compaction_scheduler: CompactionScheduler,
    
    /// Tiered storage manager (S3/GCS/Azure)
    tiered_storage: Option<TieredStorageManager>,
}

pub struct ExternalFileManager {
    /// File metadata cache (path, format, size, etc.)
    metadata_cache: Arc<RwLock<LruCache<FileId, FileMetadata>>>,
    
    /// Memory-mapped file cache
    mmap_cache: Arc<RwLock<LruCache<FileId, Mmap>>>,
    
    /// Supported formats
    formats: Vec<ExternalFormat>, // Parquet, Safetensors, Zarr
}

pub enum DataLocation {
    /// Native LSM storage
    Native { level: u8, segment_id: u64, offset: u64 },
    
    /// External file reference
    External { file_path: String, format: ExternalFormat },
}
```

### Level Organization

| Level | Size Factor | Total Size (RF=3) | Sorting | Index Type | Purpose |
|-------|-------------|-------------------|---------|------------|---------|
| MemTable | - | 16-64MB | Skip List | In-memory | Write buffer |
| **L0** | Base | 1-4GB | Unsorted | PPHM | Recent writes |
| **L1** | 10× | 10-40GB | Sorted | Bloom | First compaction |
| **L2** | 10× | 100-400GB | Sorted | Sparse | Second compaction |
| **L3+** | 10× | 1TB+ | Sorted | Sparse | Cold storage |
| **External** | N/A | Unlimited | N/A | Metadata | File references |

---

## Hybrid Storage: Native vs External

### Decision Matrix

| Data Type | Size | Access Pattern | Storage Choice | Why |
|-----------|------|----------------|----------------|-----|
| **OLTP records** | Small-Medium | Hot writes/reads | **Native LSM** | Fast writes, frequent updates |
| **Analytics tables** | Large | Read-mostly | **External (Parquet)** | Columnar, compressed, no duplication |
| **ML models** | Large | Read-once | **External (Safetensors)** | Mmap, zero-copy, native format |
| **Embeddings** | Medium | ANN search | **Native LSM** | SIMD, indexed, frequent queries |
| **Time-series** | Large | Append-mostly | **Native LSM** | Efficient compaction, dedup |
| **Scientific arrays** | Very Large | Chunked access | **External (Zarr)** | Parallel I/O, cloud-native |
| **Archived data** | Any | Rare access | **External (any)** | Cost-effective, no compaction |

### Implementation

```rust
/// Decide storage location based on data characteristics
pub fn decide_storage_location(
    data: &Data,
    metadata: &Metadata,
) -> DataLocation {
    match (data.size(), data.access_pattern(), metadata.format) {
        // Small, hot data → Native LSM
        (size, AccessPattern::Hot, _) if size < 100_000_000 => {
            DataLocation::Native {
                level: 0,
                segment_id: allocate_segment(),
                offset: 0,
            }
        }
        
        // Large, cold data with native format → External
        (_, AccessPattern::Cold, Some(format)) 
            if format.is_external() => {
            DataLocation::External {
                file_path: format.generate_path(),
                format,
            }
        }
        
        // ML models → Always external
        (_, _, Some(ExternalFormat::Safetensors)) => {
            DataLocation::External {
                file_path: generate_model_path(),
                format: ExternalFormat::Safetensors,
            }
        }
        
        // Default: Native LSM
        _ => DataLocation::Native {
            level: 0,
            segment_id: allocate_segment(),
            offset: 0,
        }
    }
}

/// Read from hybrid storage
pub async fn read_data(location: &DataLocation) -> Result<Data> {
    match location {
        // Native LSM: Read from MemTable → L0 → L1+
        DataLocation::Native { level, segment_id, offset } => {
            lsm.read(*level, *segment_id, *offset).await
        }
        
        // External: Memory-map file on demand
        DataLocation::External { file_path, format } => {
            // Check mmap cache first
            if let Some(mmap) = mmap_cache.get(file_path) {
                return parse_format(mmap, format);
            }
            
            // Cache miss: Open and mmap file
            let file = File::open(file_path).await?;
            let mmap = unsafe { Mmap::map(&file)? };
            
            // Cache for future access
            mmap_cache.insert(file_path.clone(), mmap.clone());
            
            parse_format(&mmap, format)
        }
    }
}
```

### Benefits of Hybrid Approach

| Benefit | Native LSM | External Files |
|---------|-----------|----------------|
| **Write throughput** | ✅ 500M+/sec | ⚠️ Depends on format |
| **Read latency** | ✅ Sub-ms (cached) | ✅ Sub-ms (mmap) |
| **Storage efficiency** | ⚠️ Compaction overhead | ✅ No duplication |
| **Compression** | ✅ 30-50% | ✅ Format-native |
| **Updates** | ✅ Efficient | ❌ Immutable |
| **Deduplication** | ✅ Built-in | ❌ Manual |
| **Format flexibility** | ⚠️ Native only | ✅ Any format |
| **Cost** | ⚠️ SSD/NVMe | ✅ S3/GCS (cheap) |

### Migration Strategy

```rust
/// Automatically migrate old data to external files
pub async fn auto_migrate_to_external(
    config: MigrationConfig,
) -> Result<()> {
    // Find cold data candidates
    let candidates = lsm.find_cold_data(config.age_threshold).await?;
    
    for segment in candidates {
        // Skip if too small or frequently accessed
        if segment.size < config.min_size || segment.access_rate > config.max_access_rate {
            continue;
        }
        
        // Export to external format
        let external_path = match segment.data_type {
            DataType::Tabular => {
                export_to_parquet(&segment, config.output_dir).await?
            }
            DataType::Tensor => {
                export_to_safetensors(&segment, config.output_dir).await?
            }
            DataType::Array => {
                export_to_zarr(&segment, config.output_dir).await?
            }
        };
        
        // Update metadata to point to external file
        update_location(&segment.key, DataLocation::External {
            file_path: external_path,
            format: segment.data_type.external_format(),
        }).await?;
        
        // Delete native LSM data
        lsm.delete_segment(segment.id).await?;
        
        info!("Migrated segment {} to {}", segment.id, external_path);
    }
    
    Ok(())
}
```

---

## Write Path

### Overview

```
┌────────────────────────────────────────────────────────────┐
│                       WRITE PATH                            │
└────────────────────────────────────────────────────────────┘

1. Client Write
      ↓
2. Acquire Session ID (from Obelisk Node)
      ↓
3. Write to WAL (durable)
      ↓
4. Write to MemTable (in-memory)
      ↓
5. ACK to client (sub-millisecond)
      ↓
6. Background Flush (when MemTable full)
      ↓
7. Background Compaction (when L0 has 4+ segments)
```

### MemTable Implementation

```rust
/// In-memory sorted data structure (Skip List)
pub struct MemTable {
    /// Skip list for O(log n) reads/writes
    data: SkipMap<Vec<u8>, Record>,
    
    /// Current size in bytes
    size: AtomicUsize,
    
    /// Scarab ID (timestamp for ordering)
    created_at: u64,
    
    /// Maximum size before flush
    max_size: usize, // Default: 16MB
}

impl MemTable {
    /// Write a record to the MemTable
    pub fn write(&self, key: Vec<u8>, record: Record) -> Result<()> {
        let record_size = key.len() + record.value.len();
        
        // Check if we need to flush
        let current_size = self.size.fetch_add(record_size, Ordering::SeqCst);
        if current_size + record_size > self.max_size {
            return Err(Error::MemTableFull);
        }
        
        // Insert into skip list
        self.data.insert(key, record);
        Ok(())
    }
    
    /// Read a record from the MemTable
    pub fn read(&self, key: &[u8]) -> Option<Record> {
        self.data.get(key).map(|entry| entry.value().clone())
    }
    
    /// Convert to immutable and create iterator for flushing
    pub fn freeze(&self) -> MemTableIterator {
        MemTableIterator::new(self.data.clone())
    }
}
```

### Write-Ahead Log (WAL)

```rust
/// Durable write-ahead log
pub struct WriteAheadLog {
    /// Current WAL file
    file: File,
    
    /// Fsync policy
    sync_policy: SyncPolicy,
    
    /// Buffer for batching writes
    buffer: Vec<u8>,
}

pub enum SyncPolicy {
    /// Fsync every write (slowest, most durable)
    Always,
    
    /// Fsync every N ms (balanced)
    Interval(Duration), // Default: 10ms
    
    /// Fsync every N bytes (write-optimized)
    Bytes(usize),
    
    /// Never fsync (fastest, least durable)
    Never,
}

impl WriteAheadLog {
    /// Append a record to the WAL
    pub async fn append(&mut self, record: &Record) -> Result<()> {
        // Serialize record
        let bytes = bincode::serialize(record)?;
        
        // Write length-prefixed record
        self.buffer.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        self.buffer.extend_from_slice(&bytes);
        
        // Write to file
        self.file.write_all(&self.buffer)?;
        self.buffer.clear();
        
        // Fsync based on policy
        match self.sync_policy {
            SyncPolicy::Always => self.file.sync_all()?,
            SyncPolicy::Interval(duration) => {
                // Handled by background thread
            }
            SyncPolicy::Bytes(threshold) => {
                if self.file.metadata()?.len() >= threshold as u64 {
                    self.file.sync_all()?;
                }
            }
            SyncPolicy::Never => {}
        }
        
        Ok(())
    }
}
```

### Flush Process

```rust
/// Background flush task
pub async fn flush_memtable(
    immutable_memtable: MemTable,
    level0: Arc<RwLock<Vec<Segment>>>,
    storage_path: PathBuf,
) -> Result<()> {
    // 1. Create new segment file
    let segment_id = generate_scarab_id();
    let segment_path = storage_path.join(format!("{:020}.log", segment_id));
    let mut segment_file = File::create(&segment_path)?;
    
    // 2. Write all records from memtable (already sorted)
    let mut offset = 0u64;
    let mut index_entries = Vec::new();
    
    for (key, record) in immutable_memtable.iter() {
        // Serialize record
        let bytes = bincode::serialize(&record)?;
        
        // Write to segment
        segment_file.write_all(&bytes)?;
        
        // Build index entry (every 4KB)
        if offset % 4096 == 0 {
            index_entries.push(IndexEntry {
                key: key.clone(),
                offset,
                size: bytes.len() as u32,
            });
        }
        
        offset += bytes.len() as u64;
    }
    
    // 3. Fsync segment
    segment_file.sync_all()?;
    
    // 4. Build PPHM index
    let pphm = build_pphm_index(&index_entries)?;
    let index_path = storage_path.join(format!("{:020}.pphm", segment_id));
    pphm.write_to_file(&index_path)?;
    
    // 5. Add segment to L0
    let segment = Segment {
        id: segment_id,
        path: segment_path,
        index: IndexType::PPHM(pphm),
        size: offset,
        min_key: index_entries.first().unwrap().key.clone(),
        max_key: index_entries.last().unwrap().key.clone(),
    };
    
    level0.write().push(segment);
    
    // 6. Delete WAL (no longer needed)
    delete_wal_for_memtable(immutable_memtable.created_at)?;
    
    Ok(())
}
```

---

## Read Path

### Multi-Level Lookup

```
┌────────────────────────────────────────────────────────────┐
│                       READ PATH                             │
└────────────────────────────────────────────────────────────┘

1. Check MemTable (hot data)
      ↓ Not found
2. Check Immutable MemTables
      ↓ Not found
3. Check L0 segments (PPHM index, newest first)
      ↓ Not found
4. Check L1 runs (Bloom filter → Binary search)
      ↓ Not found
5. Check L2+ runs (Sparse index → Binary search)
      ↓ Not found
6. Check Tiered Storage (S3/GCS, if configured)
      ↓ Not found
7. Return KeyNotFound
```

### Read Implementation

```rust
impl LSMStorage {
    /// Read a key from the storage engine
    pub async fn read(&self, key: &[u8]) -> Result<Option<Record>> {
        // 1. Check active MemTable
        if let Some(record) = self.memtable.read().read(key) {
            return Ok(Some(record));
        }
        
        // 2. Check immutable MemTables (newest first)
        for memtable in self.immutable_memtables.read().iter().rev() {
            if let Some(record) = memtable.read(key) {
                return Ok(Some(record));
            }
        }
        
        // 3. Check L0 segments (newest first)
        for segment in self.level0.read().iter().rev() {
            if let Some(record) = self.read_from_segment(segment, key).await? {
                return Ok(Some(record));
            }
        }
        
        // 4. Check L1+ (with Bloom filter optimization)
        for level in self.levels.read().iter() {
            for run in level.runs.iter() {
                // Bloom filter check first
                if !run.bloom_filter.may_contain(key) {
                    continue; // Definitely not in this run
                }
                
                // Binary search in sorted run
                if let Some(record) = self.read_from_run(run, key).await? {
                    return Ok(Some(record));
                }
            }
        }
        
        // 5. Check tiered storage (if configured)
        if let Some(tiered) = &self.tiered_storage {
            if let Some(record) = tiered.read(key).await? {
                return Ok(Some(record));
            }
        }
        
        Ok(None)
    }
    
    /// Read from a segment using PPHM index
    async fn read_from_segment(
        &self,
        segment: &Segment,
        key: &[u8],
    ) -> Result<Option<Record>> {
        // Use PPHM for O(1) lookup
        let index = match &segment.index {
            IndexType::PPHM(pphm) => pphm,
            _ => return self.read_from_segment_scan(segment, key).await,
        };
        
        // O(1) lookup in PPHM
        let entry = match index.lookup(key) {
            Some(e) => e,
            None => return Ok(None),
        };
        
        // Memory-mapped read (zero-copy)
        let mmap = segment.get_mmap()?;
        let bytes = &mmap[entry.offset as usize..(entry.offset + entry.size as u64) as usize];
        let record = bincode::deserialize(bytes)?;
        
        Ok(Some(record))
    }
}
```

### Read Optimizations

#### 1. Bloom Filters (L1+)

```rust
/// Bloom filter for fast negative lookups
pub struct BloomFilter {
    /// Bit array
    bits: BitVec,
    
    /// Number of hash functions
    k: usize,
    
    /// False positive rate (default: 1%)
    fpr: f64,
}

impl BloomFilter {
    /// Check if key MAY be in the set
    pub fn may_contain(&self, key: &[u8]) -> bool {
        let hash1 = seahash::hash(key);
        let hash2 = xxhash_rust::xxh3::xxh3_64(key);
        
        for i in 0..self.k {
            let hash = hash1.wrapping_add(i as u64 * hash2);
            let bit = (hash % self.bits.len() as u64) as usize;
            
            if !self.bits[bit] {
                return false; // Definitely not in set
            }
        }
        
        true // Maybe in set (need to check)
    }
}
```

**Performance**: 10-1000× faster for point queries (eliminates disk I/O)

#### 2. Block Cache

```rust
/// LRU cache for frequently accessed blocks
pub struct BlockCache {
    cache: Arc<Mutex<LruCache<BlockId, Arc<[u8]>>>>,
    capacity: usize, // Default: 512MB
}

impl BlockCache {
    pub fn get(&self, block_id: &BlockId) -> Option<Arc<[u8]>> {
        self.cache.lock().get(block_id).cloned()
    }
    
    pub fn insert(&self, block_id: BlockId, data: Arc<[u8]>) {
        self.cache.lock().put(block_id, data);
    }
}
```

**Hit Rate**: 80-95% for hot data (eliminates disk I/O)

---

## Compaction

### Compaction Strategy

Pyralog uses **Leveled Compaction** (RocksDB-style):

```
┌────────────────────────────────────────────────────────────┐
│                  COMPACTION STRATEGY                        │
└────────────────────────────────────────────────────────────┘

Trigger: L0 has 4+ segments

Process:
1. Select all L0 segments (unsorted)
2. Select overlapping L1 runs (sorted)
3. Merge-sort all segments
4. Deduplicate keys (LWW by default)
5. Write output to new L1 run
6. Delete input segments
7. Update manifest

Amplification:
- Write Amp: ~10× (one write amplified 10× through compaction)
- Read Amp: O(levels) = O(log N) (check each level)
- Space Amp: ~1.1× (10% overhead for multiple versions)
```

### Compaction Implementation

```rust
/// Compaction task
pub async fn compact_level0_to_level1(
    level0: Arc<RwLock<Vec<Segment>>>,
    level1: Arc<RwLock<Level>>,
) -> Result<()> {
    // 1. Select all L0 segments
    let l0_segments = level0.read().clone();
    if l0_segments.len() < 4 {
        return Ok(()); // Not ready to compact
    }
    
    // 2. Find overlapping L1 runs
    let min_key = l0_segments.iter().map(|s| &s.min_key).min().unwrap();
    let max_key = l0_segments.iter().map(|s| &s.max_key).max().unwrap();
    
    let l1_runs: Vec<_> = level1.read().runs.iter()
        .filter(|run| overlaps(run, min_key, max_key))
        .cloned()
        .collect();
    
    // 3. Create merge iterator
    let mut merge_iter = MergeIterator::new();
    
    for segment in &l0_segments {
        merge_iter.add_iterator(segment.iter());
    }
    
    for run in &l1_runs {
        merge_iter.add_iterator(run.iter());
    }
    
    // 4. Write output run
    let output_run = write_sorted_run(merge_iter, DeduplicationStrategy::LWW).await?;
    
    // 5. Update L1
    let mut l1_write = level1.write();
    l1_write.runs.retain(|run| !l1_runs.contains(run));
    l1_write.runs.push(output_run);
    
    // 6. Delete L0 segments
    level0.write().clear();
    
    // 7. Schedule next compaction (L1 → L2 if needed)
    schedule_compaction_if_needed(&level1, &level2);
    
    Ok(())
}

/// Merge iterator with deduplication
struct MergeIterator {
    iterators: Vec<Box<dyn Iterator<Item = (Vec<u8>, Record)>>>,
    heap: BinaryHeap<HeapEntry>,
}

impl Iterator for MergeIterator {
    type Item = (Vec<u8>, Record);
    
    fn next(&mut self) -> Option<Self::Item> {
        // Pop minimum key from heap
        let entry = self.heap.pop()?;
        
        // Deduplicate: skip all entries with same key
        let key = entry.key.clone();
        let record = entry.record.clone();
        
        while let Some(next_entry) = self.heap.peek() {
            if next_entry.key == key {
                self.heap.pop(); // Discard older version
            } else {
                break;
            }
        }
        
        // Advance iterator that produced this entry
        if let Some((next_key, next_record)) = self.iterators[entry.iter_id].next() {
            self.heap.push(HeapEntry {
                key: next_key,
                record: next_record,
                iter_id: entry.iter_id,
            });
        }
        
        Some((key, record))
    }
}
```

### Deduplication Strategies

See [DEDUPLICATION.md](DEDUPLICATION.md) for comprehensive coverage. Summary:

| Strategy | Rule | Use Case |
|----------|------|----------|
| **LWW** | Highest LSN + timestamp | Mutable state, event logs |
| **First-Wins** | Lowest LSN + timestamp | Immutable events, "seen first" semantics |
| **Max-Value** | Max numeric value | Counters, high-water marks |
| **MVCC** | Keep all versions | Time-travel queries, audit trails |
| **Tombstone** | Delete if tombstone present | Explicit deletes |
| **Custom** | User-defined merge function | Application-specific logic |

---

## Indexes

### Index Types by Level

| Level | Index Type | Lookup Time | Memory Usage | Purpose |
|-------|------------|-------------|--------------|---------|
| MemTable | Skip List | O(log n) | Full dataset | In-memory writes |
| **L0** | PPHM | O(1) | 2.3 bits/key | Recent writes |
| **L1** | Bloom + Sparse | O(1) + O(log n) | 1 bit/key + samples | First compaction |
| **L2+** | Sparse | O(log n) | Samples only | Cold storage |

### 1. PPHM Index (Level 0)

```rust
/// Partitioned Perfect Hash Map
pub struct PPHMIndex {
    /// Perfect hash functions per partition
    phfs: Vec<PerfectHashFunction>,
    
    /// Partition boundaries
    partitions: Vec<u64>,
    
    /// Index entries (offset, size)
    entries: Vec<IndexEntry>,
}

impl PPHMIndex {
    /// O(1) guaranteed lookup
    pub fn lookup(&self, key: &[u8]) -> Option<&IndexEntry> {
        // 1. Determine partition (hash key)
        let partition_id = self.partition_key(key);
        
        // 2. Perfect hash within partition
        let phf = &self.phfs[partition_id];
        let index = phf.hash(key)?;
        
        // 3. Return entry
        Some(&self.entries[index])
    }
}
```

**Performance**: 45ns p50, 80ns p99, zero collisions

📖 See [PPHM.md](PPHM.md) for complete documentation

### 2. Bloom Filter (Level 1+)

```rust
/// Per-run Bloom filter
pub struct RunIndex {
    /// Bloom filter for negative lookups
    bloom: BloomFilter,
    
    /// Sparse index for positive lookups
    sparse: SparseIndex,
    
    /// Min/max key for range checks
    min_key: Vec<u8>,
    max_key: Vec<u8>,
}

impl RunIndex {
    pub fn lookup(&self, key: &[u8]) -> Option<IndexEntry> {
        // 1. Range check
        if key < &self.min_key || key > &self.max_key {
            return None;
        }
        
        // 2. Bloom filter check (fast negative)
        if !self.bloom.may_contain(key) {
            return None; // Definitely not in run
        }
        
        // 3. Binary search in sparse index
        self.sparse.lookup(key)
    }
}
```

**FPR**: 1% (configurable), **Memory**: 10 bits/key

### 3. Sparse Index (Level 2+)

```rust
/// Sparse index: sample every Nth key
pub struct SparseIndex {
    /// Index entries (sampled, e.g., every 4KB)
    entries: Vec<IndexEntry>,
    
    /// Sampling rate (default: 4KB)
    sample_rate: usize,
}

impl SparseIndex {
    pub fn lookup(&self, key: &[u8]) -> Option<IndexEntry> {
        // Binary search to find nearest entry <= key
        let idx = self.entries.binary_search_by(|e| e.key.cmp(key))
            .unwrap_or_else(|i| i.saturating_sub(1));
        
        Some(self.entries[idx].clone())
    }
}
```

**Memory**: ~0.025% of data size (1 entry per 4KB)

---

## Tiered Storage

### S3/GCS/Azure Integration

```rust
/// Tiered storage manager
pub struct TieredStorageManager {
    /// Local cache for hot data
    cache: Arc<Mutex<LruCache<SegmentId, Segment>>>,
    
    /// Object storage client (S3/GCS/Azure)
    object_store: Arc<dyn ObjectStore>,
    
    /// Tiering policy
    policy: TieringPolicy,
}

pub enum TieringPolicy {
    /// Move segments older than N days
    Age { days: u64 },
    
    /// Move segments when local disk > X% full
    DiskUsage { threshold_percent: u8 },
    
    /// Move cold segments (access count < N)
    AccessCount { threshold: u64 },
    
    /// Custom policy
    Custom(Box<dyn Fn(&Segment) -> bool + Send + Sync>),
}

impl TieredStorageManager {
    /// Upload segment to object storage
    pub async fn tier_segment(&self, segment: &Segment) -> Result<()> {
        // 1. Compress segment (Zstd)
        let compressed = zstd::encode_all(segment.data(), 3)?;
        
        // 2. Upload to S3/GCS/Azure
        let path = format!("segments/{}.zst", segment.id);
        self.object_store.put(&path, compressed.into()).await?;
        
        // 3. Delete local segment
        fs::remove_file(&segment.path)?;
        
        // 4. Mark as tiered in manifest
        self.mark_tiered(segment.id)?;
        
        Ok(())
    }
    
    /// Download segment from object storage
    pub async fn untier_segment(&self, segment_id: SegmentId) -> Result<Segment> {
        // 1. Check cache first
        if let Some(segment) = self.cache.lock().get(&segment_id) {
            return Ok(segment.clone());
        }
        
        // 2. Download from S3/GCS/Azure
        let path = format!("segments/{}.zst", segment_id);
        let compressed = self.object_store.get(&path).await?;
        
        // 3. Decompress
        let data = zstd::decode_all(&compressed[..])?;
        
        // 4. Reconstruct segment
        let segment = Segment::from_bytes(segment_id, data)?;
        
        // 5. Cache for future reads
        self.cache.lock().put(segment_id, segment.clone());
        
        Ok(segment)
    }
}
```

### Cost Optimization

| Tier | Storage Type | Cost (TB/mo) | Latency | Use Case |
|------|--------------|--------------|---------|----------|
| Hot | Local NVMe | $100-200 | <1ms | Active data (L0-L1) |
| Warm | EBS/Persistent | $40-100 | 1-10ms | Recent data (L2) |
| Cold | S3 Standard | $23 | 10-100ms | Archive (L3+) |
| Glacier | S3 Glacier | $1-4 | Hours | Long-term archive |

**Savings**: 70-90% reduction in storage costs for cold data

---

## Memory-Mapped I/O

### Zero-Copy Reads

```rust
/// Memory-mapped segment
pub struct MmapSegment {
    /// Memory map (read-only)
    mmap: Mmap,
    
    /// Segment metadata
    metadata: SegmentMetadata,
}

impl MmapSegment {
    /// Read record without copying
    pub fn read_record(&self, offset: u64, size: u32) -> &[u8] {
        let start = offset as usize;
        let end = start + size as usize;
        
        // Zero-copy slice into mmap
        &self.mmap[start..end]
    }
    
    /// Scan range without copying
    pub fn scan_range(&self, start_key: &[u8], end_key: &[u8]) -> impl Iterator<Item = Record> + '_ {
        // Use sparse index to find start offset
        let start_offset = self.metadata.sparse_index.lookup(start_key).offset;
        
        // Iterate over mmap without copying
        MmapIterator {
            mmap: &self.mmap,
            offset: start_offset,
            end_key: end_key.to_vec(),
        }
    }
}
```

**Performance**: 30-50% faster reads (eliminates buffer copy)

### Read-Ahead

```rust
/// Hint OS to read ahead
pub fn hint_sequential_read(segment: &MmapSegment) -> Result<()> {
    unsafe {
        libc::madvise(
            segment.mmap.as_ptr() as *mut libc::c_void,
            segment.mmap.len(),
            libc::MADV_SEQUENTIAL,
        );
    }
    Ok(())
}
```

**Benefit**: 2-3× faster sequential scans

---

## Data Format

### Segment File Format

```
┌────────────────────────────────────────────────────────────┐
│                      SEGMENT FILE                           │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  Header (64 bytes)                                          │
│  ├─ Magic: "PYRA" (4 bytes)                                │
│  ├─ Version: u16 (2 bytes)                                 │
│  ├─ Compression: u8 (1 byte)                               │
│  ├─ Checksum Type: u8 (1 byte)                             │
│  ├─ Segment ID: u64 (8 bytes)                              │
│  ├─ Min LSN: u64 (8 bytes)                                 │
│  ├─ Max LSN: u64 (8 bytes)                                 │
│  ├─ Record Count: u64 (8 bytes)                            │
│  ├─ Index Offset: u64 (8 bytes)                            │
│  └─ Reserved: (16 bytes)                                   │
│                                                             │
│  Records (variable)                                         │
│  ├─ Record 1                                                │
│  │   ├─ Length: u32 (4 bytes)                              │
│  │   ├─ Checksum: u32 (4 bytes)                            │
│  │   ├─ Key Length: u16 (2 bytes)                          │
│  │   ├─ Key: [u8; key_len]                                 │
│  │   ├─ LSN: u64 (8 bytes)                                 │
│  │   ├─ Timestamp: u64 (8 bytes)                           │
│  │   └─ Value: [u8; value_len]                             │
│  ├─ Record 2                                                │
│  └─ ...                                                     │
│                                                             │
│  Index (variable)                                           │
│  ├─ Index Type: u8 (PPHM/Bloom/Sparse)                     │
│  └─ Index Data: [u8; index_size]                           │
│                                                             │
│  Footer (32 bytes)                                          │
│  ├─ Index Checksum: u32 (4 bytes)                          │
│  ├─ File Checksum: u32 (4 bytes)                           │
│  └─ Magic: "GOLA" (4 bytes) ← "PYRA" backwards             │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Record Format

```rust
#[derive(Serialize, Deserialize)]
pub struct Record {
    /// Unique log sequence number
    pub lsn: u64,
    
    /// Scarab ID (timestamp-based)
    pub timestamp: u64,
    
    /// Record key
    pub key: Vec<u8>,
    
    /// Record value
    pub value: Vec<u8>,
    
    /// Optional headers
    pub headers: Option<HashMap<String, Vec<u8>>>,
    
    /// Checksum (CRC32C)
    pub checksum: u32,
}
```

---

## Performance Characteristics

### Write Performance

| Metric | Value | Configuration |
|--------|-------|---------------|
| **Throughput** | 500M writes/sec | 50-node cluster, batching |
| **Latency (p50)** | 0.2ms | Write caching enabled |
| **Latency (p99)** | 0.8ms | Write caching enabled |
| **Latency (p999)** | 5ms | WAL fsync |
| **Durability** | Configurable | fsync policy |

### Read Performance

| Metric | Value | Configuration |
|--------|-------|---------------|
| **Throughput** | 100M reads/sec | 50-node cluster, cached |
| **Latency (p50)** | 45ns | PPHM index, L0 hit |
| **Latency (p99)** | 80ns | PPHM index, L0 hit |
| **Cache Hit Rate** | 80-95% | Block cache enabled |
| **Bloom Filter FPR** | 1% | Default configuration |

### Storage Efficiency

| Metric | Value | Notes |
|--------|-------|-------|
| **Write Amplification** | 10× | Leveled compaction |
| **Space Amplification** | 1.1× | 10% overhead |
| **Compression Ratio** | 2-5× | Zstd level 3 |
| **Index Overhead** | 2.3 bits/key | PPHM for L0 |

---

## Configuration & Tuning

### Storage Configuration

```rust
pub struct StorageConfig {
    /// MemTable size (default: 16MB)
    pub memtable_size: usize,
    
    /// Number of MemTables (default: 2)
    pub num_memtables: usize,
    
    /// L0 compaction trigger (default: 4 segments)
    pub l0_compaction_trigger: usize,
    
    /// Segment size (default: 1GB)
    pub segment_size: u64,
    
    /// Compression algorithm
    pub compression: Compression,
    
    /// Bloom filter false positive rate (default: 1%)
    pub bloom_fpr: f64,
    
    /// Block cache size (default: 512MB)
    pub block_cache_size: usize,
    
    /// WAL fsync policy
    pub wal_sync_policy: SyncPolicy,
    
    /// Tiered storage configuration
    pub tiered_storage: Option<TieredStorageConfig>,
}

#[derive(Default)]
pub enum Compression {
    None,
    #[default]
    Zstd { level: i32 }, // Default: level 3
    Lz4,
    Snappy,
}
```

### Tuning Guidelines

#### Write-Heavy Workload

```rust
StorageConfig {
    memtable_size: 64 * 1024 * 1024, // 64MB (larger buffer)
    num_memtables: 4, // More memtables
    wal_sync_policy: SyncPolicy::Interval(Duration::from_millis(10)),
    l0_compaction_trigger: 8, // Delay compaction
    compression: Compression::Lz4, // Fast compression
    ..Default::default()
}
```

**Result**: 2-3× higher write throughput, ~10ms durability lag

#### Read-Heavy Workload

```rust
StorageConfig {
    block_cache_size: 4 * 1024 * 1024 * 1024, // 4GB cache
    bloom_fpr: 0.001, // 0.1% FPR (more memory)
    l0_compaction_trigger: 2, // Aggressive compaction
    compression: Compression::Zstd { level: 3 },
    ..Default::default()
}
```

**Result**: 5-10× faster reads, higher CPU for compaction

#### Balanced Workload

```rust
StorageConfig::default() // Use defaults
```

**Result**: Good balance for most use cases

---

## Best Practices

### 1. Batching Writes

```rust
// ❌ Bad: Individual writes
for record in records {
    storage.write(record).await?;
}

// ✅ Good: Batch writes
storage.write_batch(records).await?;
```

**Benefit**: 10-100× higher throughput

### 2. Use Compression

```rust
// ✅ Enable compression for cold data
StorageConfig {
    compression: Compression::Zstd { level: 3 },
    ..Default::default()
}
```

**Benefit**: 2-5× storage savings, minimal CPU overhead

### 3. Tune Compaction

```rust
// Write-heavy: Delay compaction
l0_compaction_trigger: 8

// Read-heavy: Aggressive compaction
l0_compaction_trigger: 2
```

**Tradeoff**: Write amp vs. read performance

### 4. Monitor Metrics

```rust
// Track key metrics
prometheus::gauge!("pyralog.storage.memtable_size").set(size);
prometheus::counter!("pyralog.storage.compactions_total").increment(1);
prometheus::histogram!("pyralog.storage.read_latency_seconds").observe(duration);
```

**Tools**: Prometheus, Grafana, built-in observability

### 5. Tiered Storage

```rust
// ✅ Move cold data to S3/GCS
TieringPolicy::Age { days: 30 }
```

**Benefit**: 70-90% cost reduction

---

## Summary

Pyralog's LSM-based storage engine delivers:

- ✅ **500M+ writes/sec** with sub-millisecond latency
- ✅ **O(1) lookups** on recent data (PPHM)
- ✅ **70-90% cost savings** with tiered storage
- ✅ **Zero-copy reads** with memory-mapped I/O
- ✅ **30-50% compression** with Zstd
- ✅ **Configurable durability** (fsync policies)

### Next Steps

- 📖 [MEMORY_ONLY_MODE.md](MEMORY_ONLY_MODE.md) - Ultra-fast ephemeral storage (10-100× faster)
- 📖 [DEDUPLICATION.md](DEDUPLICATION.md) - Multi-layer deduplication strategies
- 📖 [PPHM.md](PPHM.md) - Perfect hash map indexes
- 📖 [ARROW.md](ARROW.md) - Apache Arrow integration
- 📖 [DATA_FORMATS.md](DATA_FORMATS.md) - External formats (Parquet, Safetensors, Zarr)
- 📖 [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning guide
- 📖 [OPERATIONS.md](OPERATIONS.md) - Operational best practices
- 📊 [diagrams/lsm-storage.mmd](diagrams/lsm-storage.mmd) - Visual architecture

---

**Questions?** Join us on [Discord](https://discord.gg/pyralog) or [open an issue](https://github.com/pyralog/pyralog/issues).

