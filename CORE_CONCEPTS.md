# Pyralog Core Concepts

**Fundamental concepts and terminology in Pyralog.**

This document provides a concise reference for Pyralog's core abstractions. For detailed guides, see the linked documentation.

---

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Log ID](#log-id)
3. [Partitions](#partitions)
4. [Records](#records)
5. [Offsets](#offsets)
6. [Epochs](#epochs)
7. [Consumer Groups](#consumer-groups)
8. [Replication](#replication)
9. [Segments](#segments)
10. [Retention](#retention)
11. [Compaction](#compaction)
12. [Quorums](#quorums)
13. [Acknowledgments](#acknowledgments)
14. [Concept Hierarchy](#concept-hierarchy)

---

## Quick Reference

| Concept | Description | Example | Details |
|---------|-------------|---------|---------|
| **LogId** | Logical stream of records (like Kafka topic) | `"user-events"` | User-facing identifier |
| **Partition** | Physical shard for parallelism | Partition 3 of `"user-events"` | Distribution unit |
| **Record** | Individual message | `{ key: "user-123", value: "..." }` | Data unit |
| **Offset** | Position within partition | `1000` | Sequential position |
| **Epoch** | Leader generation number | `5` | Failover tracking |
| **Consumer Group** | Cooperative consumers | `"analytics-processors"` | Load balancing |
| **Replication** | Data copies across nodes | RF=3 (1 leader + 2 followers) | Durability |
| **Segment** | Physical storage file | `00000000001000.log` | Immutable log file |
| **Retention** | Data lifecycle policy | 7 days or 100GB | When to delete data |
| **Compaction** | Storage optimization | Keep latest per key | Remove old versions |
| **Quorum** | Consistency guarantee | Write quorum = 2/3 | Min replicas for operation |
| **Acknowledgment** | Write confirmation | `ack=all` | When write is considered successful |

---

## Log ID

### Overview

A **LogId** is the unique identifier for a logical stream of records in Pyralog.

```rust
pub struct LogId(String);

// Examples
LogId::new("user-events")
LogId::new("payment-transactions")
LogId::new("system-metrics")
```

### Conceptual Model

```
Pyralog Cluster
  ├─ Log: "user-events"
  │    ├─ Partition 0
  │    ├─ Partition 1
  │    ├─ Partition 2
  │    └─ Partition 3
  ├─ Log: "payment-transactions"
  │    ├─ Partition 0
  │    └─ Partition 1
  └─ Log: "system-metrics"
       └─ Partition 0
```

### Naming Conventions

| Pattern | Example | Use Case |
|---------|---------|----------|
| **Simple** | `user-events` | Single-environment projects |
| **Environment-based** | `production.orders` | Multi-environment deployments |
| **Team-based** | `analytics.clickstream` | Team-specific logs |
| **Service-based** | `auth-service.login-events` | Microservices architecture |

**Recommendations**:
- ✅ Use kebab-case: `user-events`
- ✅ Be descriptive: `payment-transactions` (not `events`)
- ✅ Use dots for hierarchy: `production.user.events`
- ❌ Avoid special chars: `user/events`, `events@prod`
- ❌ Avoid creating one log per user/tenant (use keys instead)

### Basic Operations

**Create a log**:
```rust
client.create_log("user-events").await?;

// With custom config
client.create_log_with_config(
    "payment-transactions",
    LogConfig {
        partition_count: 10,
        replication_factor: 3,
        retention: RetentionConfig::Time(Duration::from_days(7)),
        ..Default::default()
    },
).await?;
```

**Write to a log**:
```rust
client.produce(
    "user-events",
    Record::new(
        Some(b"user-123".to_vec()),  // Key
        b"login event".to_vec(),     // Value
    ),
).await?;
```

**Read from a log**:
```rust
let records = client.consume(
    "user-events",
    LogOffset::ZERO,
    1000,  // max records
).await?;
```

**List logs**:
```rust
let logs = client.list_logs().await?;
for log in logs {
    println!("Log: {}, Partitions: {}", log.log_id, log.partition_count);
}
```

### Log Metadata

```rust
pub struct LogMetadata {
    pub log_id: LogId,
    pub partition_count: u32,
    pub replication_factor: u32,
    pub partitioning_mode: PartitioningMode,  // Static or Dynamic
    pub retention_config: RetentionConfig,
    pub created_at: SystemTime,
    pub partitions: Vec<PartitionMetadata>,
}
```

### Common Patterns

| Pattern | Example | When to Use |
|---------|---------|-------------|
| **Per-Entity** | `users`, `orders`, `products` | Clear entity boundaries |
| **Per-Event-Type** | `user-login`, `user-logout` | Separate event streams |
| **Unified Event Log** | `events` (with type in record) | Single stream for all events |
| **Time-Partitioned** | `events-2025-01`, `events-2025-02` | Rotation by time period |

### Multi-Tenancy

**Pattern 1: Log per tenant** (low tenant count)
```rust
let log_id = LogId::new(&format!("tenant.{}.events", tenant_id));
```

**Pattern 2: Shared log with key-based isolation** (high tenant count, recommended)
```rust
let record = Record::new(
    Some(format!("{}:user-123", tenant_id).into_bytes()),  // Tenant in key
    event_data,
);
```

### Performance Guidelines

| Log Count | Status | Notes |
|-----------|--------|-------|
| 10-100 | ✅ Optimal | Recommended range |
| 100-1,000 | ⚠️ Acceptable | Requires tuning |
| 1,000+ | ❌ Excessive | Consider federation or shared logs |

**Rule of thumb**: Use fewer, larger logs with more partitions rather than many small logs.

### System Equivalents

| System | Pyralog Equivalent |
|--------|-------------------|
| Kafka | Topic → LogId |
| Pulsar | Topic → LogId |
| Kinesis | Stream → LogId |
| Azure Event Hub | Event Hub → LogId |
| RabbitMQ | Queue → LogId |

**See also**: [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md) for partition management.

---

## Partitions

### Overview

A **Partition** is a physical shard of a log that enables parallelism and scalability.

```rust
pub struct PartitionId(u32);

pub struct FullPartitionId {
    pub log_id: LogId,
    pub partition_id: u32,
}
```

### Partition Assignment

```
Log "user-events" with 4 partitions:
  ├─ Partition 0: Records where hash(key) % 4 == 0
  ├─ Partition 1: Records where hash(key) % 4 == 1
  ├─ Partition 2: Records where hash(key) % 4 == 2
  └─ Partition 3: Records where hash(key) % 4 == 3
```

### Purpose

| Benefit | Description |
|---------|-------------|
| **Parallelism** | Multiple leaders → distributed writes |
| **Scalability** | More partitions → higher throughput |
| **Ordering** | Records in same partition are ordered |
| **Distribution** | Data spread across cluster nodes |

### Partition Sizing

| Log Size | Partition Count | Mode |
|----------|----------------|------|
| Small | 1-10 | Static |
| Medium | 10-100 | Static or Dynamic |
| Large | 100-1,000 | Dynamic (auto-scale) |

### LogId vs PartitionId

| Aspect | LogId | PartitionId |
|--------|-------|-------------|
| **Type** | Logical | Physical |
| **Audience** | User-facing | Internal |
| **Scope** | Contains partitions | Single shard |
| **Config** | Has retention, replication | Inherits from log |
| **Example** | `"user-events"` | `"user-events"` partition 3 |

**See also**: [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md), [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md)

---

## Records

### Overview

A **Record** is a single message in a log.

```rust
pub struct Record {
    pub offset: LogOffset,          // Server-assigned position
    pub epoch: Epoch,                // Server-assigned epoch
    pub timestamp: SystemTime,       // Record timestamp
    pub key: Option<Bytes>,          // Optional routing key
    pub value: Bytes,                // Record payload
    pub headers: Vec<RecordHeader>,  // Optional metadata
}
```

### Record Components

| Component | Required | Purpose |
|-----------|----------|---------|
| **Key** | No (recommended) | Partitioning, ordering, compaction |
| **Value** | Yes | Actual message payload |
| **Headers** | No | Metadata (not used for routing) |
| **Offset** | Server-assigned | Position in partition |
| **Epoch** | Server-assigned | Leader generation |
| **Timestamp** | Auto (or custom) | Record creation time |

### Creating Records

```rust
// Minimal record (no key, random partition)
Record::new(None, b"Hello, Pyralog!".to_vec())

// Record with key (consistent partitioning)
Record::new(
    Some(b"user-123".to_vec()),
    b"login event".to_vec(),
)

// Record with headers
Record::new(
    Some(b"user-123".to_vec()),
    b"login event".to_vec(),
)
.with_header("source", "web-app")
.with_header("version", "1.0")
.with_timestamp(SystemTime::now())
```

### Key Guidelines

| Scenario | Key | Result |
|----------|-----|--------|
| **Ordering required** | User ID | Same user → same partition → ordered |
| **Load balancing** | Random/None | Uniform distribution across partitions |
| **Compaction** | Entity ID | Latest record per entity retained |
| **Multi-tenancy** | Tenant ID | Tenant isolation within shared log |

**See also**: [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md)

---

## Offsets

### Overview

An **Offset** is the sequential position of a record within a partition.

```rust
pub struct LogOffset(u64);

// Constants
LogOffset::ZERO     // Start of partition (offset 0)
LogOffset::new(42)  // Specific position
LogOffset::MAX      // End of partition
```

### Offset Types

| Type | Description | Example |
|------|-------------|---------|
| **Server-Assigned** | Sequential position in partition | `0, 1, 2, ...` |
| **Epoch-Offset** | Offset with epoch for failover safety | `{ epoch: 5, offset: 1000 }` |
| **Virtual LSN (VLSN)** | Client-managed sequence number | `1000, 1001, 1002, ...` |

### Server-Assigned Offset

```
Partition Timeline:
  Offset 0: First record
  Offset 1: Second record
  Offset 2: Third record
  ...
  Offset N: Current position
```

### Epoch-Offset

```rust
pub struct EpochOffset {
    pub epoch: Epoch,
    pub offset: u64,
}

// Example: Epoch 5, Offset 1000
EpochOffset { epoch: 5, offset: 1000 }
```

### Virtual LSN (VLSN)

Client-managed sequence numbers for custom ordering and routing.

**See also**: [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for VLSN patterns, [EPOCHS.md](EPOCHS.md) for epoch details.

---

## Epochs

### Overview

An **Epoch** is a generation number for partition leadership.

```rust
pub struct Epoch(u64);
```

### Purpose

| Benefit | Description |
|---------|-------------|
| **Safe Failover** | Prevent duplicate writes after leader change |
| **Fast Writes** | Decouple offset assignment from consensus |
| **Recovery** | Track which records came from which leader |

### Epoch Timeline

```
Partition Leadership:
  Leader N1, Epoch 1: Offsets 0-999
  Leader N2, Epoch 2: Offsets 1000-1999  (after failover)
  Leader N3, Epoch 3: Offsets 2000-...   (after another failover)
```

Each leader change increments the epoch, ensuring no two leaders write with the same epoch.

**See also**: [EPOCHS.md](EPOCHS.md) for comprehensive details.

---

## Consumer Groups

### Overview

A **Consumer Group** is a set of consumers that cooperatively consume from a log.

```rust
pub struct ConsumerGroupId(String);

// Example
ConsumerGroupId::new("analytics-processors")
```

### Purpose

| Benefit | Description |
|---------|-------------|
| **Load Balancing** | Partitions distributed among consumers |
| **Fault Tolerance** | Partition reassignment on consumer failure |
| **Exactly-Once** | Commit tracking per group |
| **Isolation** | Multiple groups consume independently |

### Example

```
Log "events" with 4 partitions, Consumer Group "analytics":

Initial assignment:
  Consumer A: Partitions [0, 1]
  Consumer B: Partitions [2, 3]

After Consumer A fails:
  Consumer B: Partitions [0, 1, 2, 3]  (rebalanced)

After new Consumer C joins:
  Consumer B: Partitions [0, 1]
  Consumer C: Partitions [2, 3]
```

### Consumer Group Strategies

| Strategy | Description | Use Case |
|----------|-------------|----------|
| **Per-Partition Commits** | Track offset per partition | Standard consumption |
| **VLSN Commits** | Track virtual sequence number | Custom ordering |
| **Manual Commits** | Application controls commits | Exactly-once processing |

**See also**: [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for commit patterns.

---

## Replication

### Overview

**Replication** ensures data durability by maintaining copies across multiple nodes.

```
Partition 0 with Replication Factor = 3:
  ├─ Leader:    Node 1  (accepts writes)
  ├─ Follower:  Node 2  (replica)
  └─ Follower:  Node 3  (replica)
```

### Key Terms

| Term | Description |
|------|-------------|
| **Replication Factor (RF)** | Total copies (leader + followers). Example: RF=3 = 1 leader + 2 followers |
| **In-Sync Replicas (ISR)** | Followers caught up with leader, eligible for quorum |
| **Write Quorum** | Minimum replicas for write to succeed. Example: RF=3, quorum=2 |
| **Read Quorum** | Minimum replicas for consistent read |

### Replication Strategies

| RF | ISR | Write Latency | Durability | Use Case |
|----|-----|---------------|------------|----------|
| 1 | 1 | Lowest | Lowest | Development, non-critical |
| 3 | 2 | Medium | High | Production (recommended) |
| 5 | 3 | Higher | Highest | Critical data, compliance |

### Failure Scenarios

| Scenario | RF=3, ISR=2 | Result |
|----------|-------------|--------|
| 1 follower down | 2 nodes in ISR | ✅ Writes continue |
| Leader down | Follower promoted | ✅ Writes continue after failover |
| 2 nodes down | 1 node remaining | ❌ Writes blocked (< quorum) |

**See also**: [ARCHITECTURE.md](ARCHITECTURE.md) for replication details, [CAP_THEOREM.md](CAP_THEOREM.md) for consistency tradeoffs.

---

## Segments

### Overview

A **Segment** is a physical storage file that holds a portion of a partition's data.

```rust
pub struct Segment {
    pub base_offset: LogOffset,      // First offset in segment
    pub size_bytes: u64,              // File size
    pub created_at: SystemTime,       // Creation time
    pub path: PathBuf,                // File path
}
```

### Segment Structure

```
Partition 0:
  ├─ Segment 00000000000000.log (offsets 0-999)
  ├─ Segment 00000000001000.log (offsets 1000-1999)
  ├─ Segment 00000000002000.log (offsets 2000-2999)
  └─ Segment 00000000003000.log (offsets 3000-..., active)
```

### Key Properties

| Property | Description |
|----------|-------------|
| **Immutability** | Closed segments are immutable (append-only) |
| **Naming** | Named by base offset: `00000000001000.log` |
| **Active Segment** | One active segment per partition accepting writes |
| **Rollover** | New segment created when size/time threshold reached |

### Segment Files

Each segment consists of multiple files:

| File | Purpose | Example |
|------|---------|---------|
| **Log file** | Actual record data | `00000000001000.log` |
| **Index file** | Offset → file position mapping | `00000000001000.index` |
| **Timeindex file** | Timestamp → offset mapping | `00000000001000.timeindex` |

### Segment Rollover

New segments are created when:

| Trigger | Default | Configurable |
|---------|---------|--------------|
| **Size threshold** | 1 GB | `segment.bytes` |
| **Time threshold** | 7 days | `segment.ms` |
| **Manual** | Admin command | - |

### Segment Lifecycle

```
1. CREATE (active)
   ├─ Accepting writes
   ├─ Growing in size
   └─ One per partition

2. CLOSE (immutable)
   ├─ No more writes
   ├─ Can be read
   ├─ Can be compacted
   └─ Can be deleted (retention)

3. DELETE
   └─ Removed by retention policy
```

### Performance Characteristics

| Operation | Performance | Notes |
|-----------|-------------|-------|
| **Write** | Sequential I/O | Append to active segment |
| **Read** | O(log N) | Binary search in index |
| **Scan** | Sequential | Read multiple segments |
| **Delete** | O(1) | Delete entire file |

**See also**: [STORAGE.md](STORAGE.md) for LSM-Tree details, [PPHM.md](PPHM.md) for index optimization.

---

## Retention

### Overview

**Retention** defines how long data is kept in the log before deletion.

```rust
pub enum RetentionConfig {
    Time(Duration),              // Delete after N time
    Size(u64),                   // Delete when > N bytes
    TimeAndSize(Duration, u64),  // Delete when either condition met
    Infinite,                    // Never delete
}
```

### Retention Strategies

| Strategy | Description | Example | Use Case |
|----------|-------------|---------|----------|
| **Time-based** | Delete after N days/hours | 7 days | Event logs, audit trails |
| **Size-based** | Delete when total size exceeds | 100 GB | Limited storage |
| **Time + Size** | Delete when either met | 7 days OR 100 GB | Balanced approach |
| **Infinite** | Never delete | ∞ | Append-only ledgers |

### Configuration

```rust
// Time-based retention (7 days)
LogConfig {
    retention: RetentionConfig::Time(Duration::from_days(7)),
    ..Default::default()
}

// Size-based retention (100 GB)
LogConfig {
    retention: RetentionConfig::Size(100_000_000_000),
    ..Default::default()
}

// Combined retention (7 days OR 100 GB, whichever first)
LogConfig {
    retention: RetentionConfig::TimeAndSize(
        Duration::from_days(7),
        100_000_000_000,
    ),
    ..Default::default()
}
```

### Retention Enforcement

| Aspect | Behavior |
|--------|----------|
| **Granularity** | Entire segments (not individual records) |
| **Check frequency** | Every 5 minutes (configurable) |
| **Deletion** | Deletes oldest segments first |
| **Active segment** | Never deleted (even if old) |

### Retention by Log Type

| Log Type | Recommended | Rationale |
|----------|-------------|-----------|
| **Audit logs** | 90-365 days | Compliance, investigations |
| **Event logs** | 7-30 days | Recent analysis, debugging |
| **Metrics** | 1-7 days | Short-term monitoring |
| **CDC streams** | 1-3 days | Real-time replication |
| **Ledgers** | Infinite | Immutable history |

### Storage Cost Optimization

```
Example: 1 TB/day ingestion

Retention: 1 day   → 1 TB storage
Retention: 7 days  → 7 TB storage
Retention: 30 days → 30 TB storage
Retention: 1 year  → 365 TB storage

Cost impact: Linear with retention period
```

**See also**: [STORAGE.md](STORAGE.md) for tiered storage, [OPERATIONS.md](OPERATIONS.md) for capacity planning.

---

## Compaction

### Overview

**Compaction** removes old versions of records to save storage space.

```rust
pub enum CompactionMode {
    None,              // No compaction
    DeleteMarkers,     // Remove tombstones after retention
    KeyBased,          // Keep only latest per key
    TimeSeries,        // Downsample old data
}
```

### Compaction Strategies

| Strategy | Keeps | Removes | Use Case |
|----------|-------|---------|----------|
| **None** | All records | Nothing | Event logs, audit trails |
| **Delete Markers** | All records | Tombstones after retention | General logs |
| **Key-Based** | Latest per key | Old versions | Entity state, CDC |
| **Time Series** | Downsampled data | High-resolution old data | Metrics, sensors |

### Key-Based Compaction

**Example**: User profile updates

```
Before compaction:
  Offset 10: user:123 → { name: "Alice", age: 25 }
  Offset 20: user:123 → { name: "Alice", age: 26 }  ← birthday
  Offset 30: user:123 → { name: "Alice A.", age: 26 }  ← name change
  Offset 40: user:456 → { name: "Bob", age: 30 }

After compaction:
  Offset 30: user:123 → { name: "Alice A.", age: 26 }  ← latest
  Offset 40: user:456 → { name: "Bob", age: 30 }       ← latest
  
Savings: 50% (4 records → 2 records)
```

### Tombstones (Delete Markers)

```rust
// Tombstone: null value with key
Record {
    key: Some(b"user:123".to_vec()),
    value: None,  // ← Tombstone
}

// Compaction behavior:
// 1. Keep tombstone during retention period
// 2. Delete tombstone after retention
// 3. Delete all records with that key
```

### Time Series Compaction

**Example**: Metrics downsampling

```
Original (1-minute resolution):
  Time 00:00 → 100 requests/min
  Time 00:01 → 105 requests/min
  Time 00:02 → 98 requests/min
  ...

After 1 day (10-minute resolution):
  Time 00:00-00:09 → avg 102 requests/min

After 7 days (1-hour resolution):
  Time 00:00-00:59 → avg 100 requests/min

Savings: 60× compression (1-min → 1-hour)
```

### Compaction Process

```
1. SELECT segments for compaction
   ├─ Choose segments eligible for compaction
   └─ Avoid active segment

2. READ records from segments
   ├─ Scan all selected segments
   └─ Build key → latest record map

3. WRITE compacted segment
   ├─ Write only latest per key
   └─ Create new segment

4. REPLACE old segments
   ├─ Atomically swap segments
   └─ Delete old segments
```

### Configuration

```rust
LogConfig {
    compaction_mode: CompactionMode::KeyBased,
    compaction_interval: Duration::from_hours(1),
    min_compaction_lag: Duration::from_minutes(5),  // Don't compact recent data
    ..Default::default()
}
```

### Compaction vs Retention

| Aspect | Retention | Compaction |
|--------|-----------|------------|
| **Purpose** | Delete old data | Remove duplicate data |
| **Trigger** | Time or size | Key updates |
| **Granularity** | Entire segments | Individual records |
| **Effect** | Data loss | Data deduplication |
| **Use case** | Limit storage | Optimize storage |

**See also**: [STORAGE.md](STORAGE.md) for LSM compaction, [DEDUPLICATION.md](DEDUPLICATION.md) for multi-layer strategies.

---

## Quorums

### Overview

A **Quorum** is the minimum number of replicas required for an operation to succeed.

```rust
pub struct QuorumConfig {
    pub write_quorum: u32,  // Min replicas for write ACK
    pub read_quorum: u32,   // Min replicas for consistent read
}
```

### Quorum Types

| Type | Description | Example |
|------|-------------|---------|
| **Write Quorum** | Min replicas to acknowledge write | RF=3, write quorum=2 |
| **Read Quorum** | Min replicas to read from | RF=3, read quorum=1 |
| **Sync Quorum** | Min in-sync replicas | RF=3, sync quorum=2 |

### Quorum Calculations

**Standard quorums**:

| RF | Majority Quorum | Tolerated Failures |
|----|----------------|-------------------|
| 1 | 1 | 0 (no fault tolerance) |
| 3 | 2 | 1 node down |
| 5 | 3 | 2 nodes down |
| 7 | 4 | 3 nodes down |

**Formula**: `quorum = floor(RF / 2) + 1`

### Consistency Guarantees

| Read Quorum | Write Quorum | Guarantee |
|-------------|--------------|-----------|
| 1 | W (all replicas) | Eventual consistency |
| R | W (where R + W > RF) | Strong consistency (quorum intersection) |
| RF (all) | 1 | Strong read consistency |

### Common Configurations

| Configuration | Write | Read | Latency | Durability | Use Case |
|---------------|-------|------|---------|------------|----------|
| **Fast writes** | 1 | 1 | Lowest | Lowest | Development, caching |
| **Balanced** | 2/3 | 1/3 | Medium | High | Production (default) |
| **Strong consistency** | 2/3 | 2/3 | Higher | High | Financial, critical |
| **Maximum durability** | 3/3 | 1/3 | Highest | Highest | Audit logs, compliance |

### Example Configuration

```rust
// Balanced: Write quorum = 2/3, Read quorum = 1/3
LogConfig {
    replication_factor: 3,
    min_in_sync_replicas: 2,  // Write quorum
    read_quorum: 1,            // Read quorum
    ..Default::default()
}

// Strong consistency: Write quorum = 2/3, Read quorum = 2/3
LogConfig {
    replication_factor: 3,
    min_in_sync_replicas: 2,  // Write quorum
    read_quorum: 2,            // Read quorum (ensures overlap)
    ..Default::default()
}
```

### Quorum and CAP Theorem

| Quorum Setting | CAP Position | Trade-off |
|----------------|--------------|-----------|
| **Low quorum** (1/3) | AP (availability) | Low latency, risk of stale reads |
| **High quorum** (3/3) | CP (consistency) | High latency, blocks on failures |
| **Balanced** (2/3 write, 1/3 read) | Balanced | Good latency + durability |

**See also**: [CAP_THEOREM.md](CAP_THEOREM.md) for detailed analysis, [ARCHITECTURE.md](ARCHITECTURE.md) for replication internals.

---

## Acknowledgments

### Overview

An **Acknowledgment (ACK)** confirms that a write has been successfully persisted.

```rust
pub enum AckPolicy {
    None,      // Don't wait for any ACK (fire-and-forget)
    Leader,    // Wait for leader to write to disk
    Quorum,    // Wait for quorum replicas
    All,       // Wait for all replicas
}
```

### Acknowledgment Policies

| Policy | Wait For | Latency | Durability | Data Loss Risk |
|--------|----------|---------|------------|----------------|
| **None** | Nothing | Lowest | Lowest | High (leader crash = loss) |
| **Leader** | Leader write | Low | Medium | Medium (1 node = loss) |
| **Quorum** | Majority replicas | Medium | High | Low (< quorum nodes = OK) |
| **All** | All replicas | Highest | Highest | Minimal (all nodes = loss) |

### ACK Policy Selection

| Use Case | Recommended | Rationale |
|----------|-------------|-----------|
| **High-throughput logs** | None or Leader | Minimize latency |
| **Event streams** | Leader | Balance speed + safety |
| **Financial transactions** | All | Maximum durability |
| **Audit logs** | All | Compliance requirements |
| **Metrics, telemetry** | None | Fire-and-forget OK |
| **Production default** | Quorum | Balanced approach |

### Configuration

```rust
// Fire-and-forget (fastest, least safe)
ProduceConfig {
    ack_policy: AckPolicy::None,
    timeout: Duration::from_millis(10),
    ..Default::default()
}

// Wait for leader (balanced)
ProduceConfig {
    ack_policy: AckPolicy::Leader,
    timeout: Duration::from_millis(50),
    ..Default::default()
}

// Wait for quorum (recommended for production)
ProduceConfig {
    ack_policy: AckPolicy::Quorum,
    timeout: Duration::from_millis(100),
    ..Default::default()
}

// Wait for all replicas (maximum durability)
ProduceConfig {
    ack_policy: AckPolicy::All,
    timeout: Duration::from_millis(200),
    ..Default::default()
}
```

### Write Latency Comparison

**Typical latencies** (RF=3, same datacenter):

| Policy | P50 | P99 | P99.9 |
|--------|-----|-----|-------|
| **None** | 0.5ms | 1ms | 2ms |
| **Leader** | 1ms | 3ms | 10ms |
| **Quorum** | 3ms | 10ms | 50ms |
| **All** | 10ms | 50ms | 200ms |

### ACK Failure Handling

| Scenario | None | Leader | Quorum | All |
|----------|------|--------|--------|-----|
| Leader crashes before write | ✅ Client OK | ❌ Timeout | ❌ Timeout | ❌ Timeout |
| Leader crashes after write | ✅ Client OK | ⚠️ ACK lost | ⚠️ ACK lost | ⚠️ ACK lost |
| 1 follower down (RF=3) | ✅ Client OK | ✅ Client OK | ✅ Client OK | ❌ Timeout |
| 2 followers down (RF=3) | ✅ Client OK | ✅ Client OK | ❌ Timeout | ❌ Timeout |

### Idempotent Writes

To handle ACK failures safely:

```rust
// Idempotent producer with session ID
ProduceConfig {
    ack_policy: AckPolicy::Quorum,
    enable_idempotence: true,  // Deduplicate retries
    max_retries: 3,
    ..Default::default()
}

client.produce(
    "user-events",
    Record::new(...)
        .with_dedup_id(scarab_id),  // Unique ID for deduplication
).await?;
```

### ACK vs Quorum

| Concept | Scope | Purpose |
|---------|-------|---------|
| **Quorum** | Replication policy | How many replicas must stay in sync |
| **Acknowledgment** | Write policy | When client considers write successful |

**Relationship**: ACK policy should align with quorum for consistency.

**See also**: [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) for exactly-once semantics, [PERFORMANCE.md](PERFORMANCE.md) for tuning.

---

## Concept Hierarchy

### Visual Hierarchy

```
Pyralog Cluster
  └─ Logs (LogId)
       ├─ Partitions
       │    ├─ Leader (Node 1)
       │    ├─ Follower (Node 2)
       │    └─ Follower (Node 3)
       │
       └─ Records
            ├─ Offset (position)
            ├─ Epoch (generation)
            ├─ Key (partitioning)
            └─ Value (payload)

Consumer Groups
  └─ Consumers
       └─ Assigned Partitions
```

### Concept Relationships

| Parent | Child | Relationship |
|--------|-------|--------------|
| Cluster | Logs | 1:N (cluster has many logs) |
| Log | Partitions | 1:N (log has many partitions) |
| Partition | Records | 1:N (partition has many records) |
| Partition | Leader | 1:1 (one leader per partition) |
| Partition | Followers | 1:N (multiple followers per partition) |
| Record | Offset | 1:1 (each record has one offset) |
| Record | Epoch | 1:1 (each record has one epoch) |
| Consumer Group | Consumers | 1:N (group has many consumers) |
| Consumer | Partitions | N:M (consumer assigned multiple partitions) |

---

## Quick Comparison with Other Systems

| Concept | Pyralog | Kafka | Pulsar | Kinesis |
|---------|---------|-------|--------|---------|
| Stream | LogId | Topic | Topic | Stream |
| Shard | Partition | Partition | Partition | Shard |
| Message | Record | Record | Message | Record |
| Position | Offset | Offset | Message ID | Sequence Number |
| Generation | Epoch | Leader Epoch | - | - |
| Consumers | Consumer Group | Consumer Group | Subscription | Consumer |
| Copies | Replication | Replication | Replication | Replication |

---

## Summary

### Core Concepts at a Glance

| Concept | Type | Key Property |
|---------|------|--------------|
| **LogId** | Logical | User-facing stream identifier |
| **Partition** | Physical | Parallelism and distribution unit |
| **Record** | Data | Individual message |
| **Offset** | Position | Sequential position in partition |
| **Epoch** | Generation | Leader failover tracking |
| **Consumer Group** | Coordination | Cooperative consumption |
| **Replication** | Durability | Data copies across nodes |
| **Segment** | Storage | Physical log file |
| **Retention** | Lifecycle | Data expiration policy |
| **Compaction** | Optimization | Deduplicate old records |
| **Quorum** | Consistency | Min replicas for operation |
| **Acknowledgment** | Confirmation | Write success policy |

### Key Takeaways

1. **LogId** is the top-level abstraction (like Kafka topic)
2. **Partitions** enable parallelism and scalability
3. **Records** are routed to partitions based on key hash
4. **Offsets** provide sequential ordering within partitions
5. **Epochs** enable safe, fast failover without consensus on every write
6. **Consumer Groups** provide load balancing and fault tolerance
7. **Replication** ensures durability with configurable quorums
8. **Segments** are immutable files storing partition data
9. **Retention** controls data lifecycle (time/size-based deletion)
10. **Compaction** removes old versions to save space
11. **Quorums** define consistency guarantees (write/read quorums)
12. **Acknowledgments** control when writes are considered successful

---

## Learn More

### Deep Dives
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture and internals
- [EPOCHS.md](EPOCHS.md) - Epochs in detail
- [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md) - Partition management
- [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) - Advanced partitioning strategies
- [CAP_THEOREM.md](CAP_THEOREM.md) - Consistency and availability tradeoffs
- [STORAGE.md](STORAGE.md) - Storage layer and LSM-Tree architecture
- [PPHM.md](PPHM.md) - Partitioned Perfect Hash Maps for indexing
- [DEDUPLICATION.md](DEDUPLICATION.md) - Multi-layer deduplication strategies
- [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) - Exactly-once semantics and transactions

### Practical Guides
- [QUICK_START.md](QUICK_START.md) - Get started in 5 minutes
- [EXAMPLES.md](EXAMPLES.md) - Code examples and patterns
- [OPERATIONS.md](OPERATIONS.md) - Deployment and operations
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tuning and optimization

### Reference
- [DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md) - Complete documentation index
- [FAQ.md](FAQ.md) - Frequently asked questions
- [COMPARISON.md](COMPARISON.md) - Comparison with other systems
