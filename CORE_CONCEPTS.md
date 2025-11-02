# Pyralog Core Concepts

Fundamental concepts and terminology in Pyralog.

## Table of Contents

1. [Log ID](#log-id)
2. [Partitions](#partitions)
3. [Records](#records)
4. [Offsets](#offsets)
5. [Epochs](#epochs)
6. [Consumer Groups](#consumer-groups)
7. [Replication](#replication)

---

## Log ID

### Overview

A **LogId** is the unique identifier for a log in Pyralog, similar to a "topic" in Kafka or a "stream" in other systems.

```rust
pub struct LogId(String);

// Examples:
let log_id = LogId::new("user-events");
let log_id = LogId::new("payment-transactions");
let log_id = LogId::new("system-metrics");
```

### Purpose

A LogId represents a **logical stream of records** that:
- Has its own set of partitions
- Has independent configuration (replication, retention, etc.)
- Can be consumed independently from other logs
- Provides namespace isolation

### Naming Convention

**Best practices:**

```rust
// ✅ Good: Descriptive, kebab-case
LogId::new("user-events")
LogId::new("payment-transactions")
LogId::new("sensor-readings")

// ✅ Good: Hierarchical (for organization)
LogId::new("production.user.events")
LogId::new("staging.payment.transactions")

// ⚠️  Acceptable: Underscores
LogId::new("user_events")

// ❌ Avoid: Too generic
LogId::new("events")
LogId::new("data")

// ❌ Avoid: Special characters that complicate URLs/paths
LogId::new("user/events")  // Slash problematic
LogId::new("events@prod")  // @ problematic
```

**Recommended patterns:**

```
Environment-based:
  production.orders
  staging.orders
  development.orders

Team-based:
  analytics.clickstream
  backend.audit-log
  frontend.user-actions

Service-based:
  auth-service.login-events
  payment-service.transactions
  inventory-service.stock-updates
```

### Log Hierarchy

```
┌─────────────────────────────────────────────────┐
│   Pyralog Cluster                                  │
├─────────────────────────────────────────────────┤
│                                                 │
│  Log: "user-events"                             │
│    ├─ Partition 0                               │
│    ├─ Partition 1                               │
│    ├─ Partition 2                               │
│    └─ Partition 3                               │
│                                                 │
│  Log: "payment-transactions"                    │
│    ├─ Partition 0                               │
│    ├─ Partition 1                               │
│    └─ Partition 2                               │
│                                                 │
│  Log: "system-metrics"                          │
│    └─ Partition 0 (single partition)            │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Log Metadata

Each LogId has associated metadata:

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

// Example metadata
LogMetadata {
    log_id: LogId::new("user-events"),
    partition_count: 10,
    replication_factor: 3,
    partitioning_mode: PartitioningMode::Static,
    retention_config: RetentionConfig::Time(Duration::from_days(7)),
    created_at: SystemTime::now(),
    partitions: vec![...],
}
```

### Creating a Log

**Via client:**

```rust
use dlog_client::PyralogClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = PyralogClient::connect("localhost:9092").await?;
    
    // Create log with default configuration
    client.create_log("user-events").await?;
    
    // Create log with custom configuration
    client.create_log_with_config(
        "payment-transactions",
        LogConfig {
            partition_count: 10,
            replication_factor: 3,
            retention: RetentionConfig::Size(10_000_000_000), // 10GB
            ..Default::default()
        },
    ).await?;
    
    Ok(())
}
```

**Via configuration file:**

```toml
# dlog.toml

[logs.user_events]
partition_count = 10
replication_factor = 3
retention_bytes = 10_000_000_000
retention_hours = 168  # 7 days

[logs.payment_transactions]
partition_count = 5
replication_factor = 3
retention_bytes = 100_000_000_000
retention_hours = 720  # 30 days
min_in_sync_replicas = 2

[logs.system_metrics]
partition_count = 1
replication_factor = 1
retention_hours = 24
```

### Using LogId in Client Code

**Writing records:**

```rust
// Write to specific log
client.produce(
    "user-events",  // LogId (converted to LogId internally)
    Record::new(
        Some(b"user-123".to_vec()),
        b"login event".to_vec(),
    ),
).await?;

// Or use LogId explicitly
let log_id = LogId::new("user-events");
client.produce_to_log(
    log_id,
    Record::new(
        Some(b"user-123".to_vec()),
        b"login event".to_vec(),
    ),
).await?;
```

**Reading records:**

```rust
// Consume from log
let records = client.consume(
    "user-events",  // LogId
    LogOffset::ZERO,
    1000,  // max records
).await?;

// Consume from specific partition
let records = client.consume_from_partition(
    "user-events",  // LogId
    5,  // partition_id
    LogOffset::ZERO,
    1000,
).await?;
```

### Log Lifecycle

```
┌─────────────────────────────────────────────────┐
│   Log Lifecycle                                 │
├─────────────────────────────────────────────────┤
│                                                 │
│  1. CREATE                                      │
│     client.create_log("user-events")            │
│     → Allocate partitions                       │
│     → Assign to nodes                           │
│     → Create storage directories                │
│                                                 │
│  2. ACTIVE                                      │
│     → Accepting writes                          │
│     → Serving reads                             │
│     → Replicating data                          │
│                                                 │
│  3. MODIFY (optional)                           │
│     client.update_log_config(...)               │
│     → Change retention                          │
│     → Add/remove partitions (dynamic mode)      │
│                                                 │
│  4. PAUSE (optional)                            │
│     client.pause_log("user-events")             │
│     → Reject writes                             │
│     → Continue serving reads                    │
│                                                 │
│  5. DELETE                                      │
│     client.delete_log("user-events")            │
│     → Mark for deletion                         │
│     → Stop writes/reads                         │
│     → Clean up storage (async)                  │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Log Discovery

**List all logs in cluster:**

```rust
// Get all log IDs
let logs = client.list_logs().await?;
for log in logs {
    println!("Log: {}", log.log_id);
    println!("  Partitions: {}", log.partition_count);
    println!("  Replication: {}", log.replication_factor);
}

// Check if log exists
if client.log_exists("user-events").await? {
    println!("Log exists!");
}

// Get detailed metadata
let metadata = client.get_log_metadata("user-events").await?;
println!("Created: {:?}", metadata.created_at);
println!("Retention: {:?}", metadata.retention_config);
```

### Multi-Tenancy with LogId

**Pattern 1: Log per tenant**

```rust
// Each tenant gets their own log
let tenant_id = "acme-corp";
let log_id = LogId::new(&format!("tenant.{}.events", tenant_id));

client.create_log(&log_id).await?;
client.produce(&log_id, record).await?;
```

**Pattern 2: Shared log with key-based isolation**

```rust
// All tenants share one log, tenant in key
let log_id = LogId::new("multi-tenant-events");

let record = Record::new(
    Some(format!("{}:user-123", tenant_id).into_bytes()),  // Key includes tenant
    event_data,
);

client.produce(log_id, record).await?;
```

### Log vs Partition

**Key differences:**

```
┌─────────────────────────────────────────────────────────┐
│   LogId vs PartitionId                                  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  LogId:                                                 │
│    - Logical concept (like Kafka topic)                │
│    - User-facing identifier                            │
│    - Contains multiple partitions                      │
│    - Has configuration (retention, replication)        │
│    - Example: "user-events"                            │
│                                                         │
│  PartitionId:                                           │
│    - Physical concept                                  │
│    - Internal identifier (usually numeric)             │
│    - Single shard of data                              │
│    - Has leader and replicas                           │
│    - Example: "user-events" partition 3                │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### LogId in Different Languages

**Rust:**
```rust
let log_id = LogId::new("user-events");
client.produce(&log_id, record).await?;
```

**Python:**
```python
from dlog import PyralogClient

client = PyralogClient("localhost:9092")
client.produce("user-events", record)
```

**Java:**
```java
PyralogClient client = new PyralogClient("localhost:9092");
client.produce("user-events", record);
```

**Go:**
```go
client := dlog.NewClient("localhost:9092")
client.Produce("user-events", record)
```

### Performance Considerations

**Number of logs:**

```
Recommended:
  - 10-100 logs per cluster: Optimal
  - 100-1000 logs: Acceptable with tuning
  - 1000+ logs: Consider federation

Per-log overhead:
  - Metadata: ~1KB per log
  - Monitoring: Metrics per log
  - Admin: Management complexity

Rule of thumb:
  - Use fewer, larger logs over many small logs
  - Group related data in one log
  - Use partitions for parallelism, not multiple logs
```

**Partition count per log:**

```
Static mode:
  Small log: 1-10 partitions
  Medium log: 10-100 partitions
  Large log: 100-1000 partitions

Dynamic mode:
  Start small: 1-10 initial partitions
  Auto-scale: Split as needed
  No practical limit
```

### Common Patterns

**1. Per-Entity Log**
```rust
// One log per business entity
LogId::new("users")
LogId::new("orders")
LogId::new("products")
LogId::new("inventory")
```

**2. Per-Event-Type Log**
```rust
// One log per event type
LogId::new("user-login-events")
LogId::new("user-logout-events")
LogId::new("user-purchase-events")
LogId::new("user-signup-events")
```

**3. Unified Event Log**
```rust
// All events in one log, type in record
LogId::new("events")

// Records include event type
Record {
    key: Some(b"user-123".to_vec()),
    value: json!({
        "type": "login",
        "timestamp": "...",
        "data": {...}
    }).to_string().into_bytes(),
}
```

**4. Time-Partitioned Log**
```rust
// One log per time period (for retention)
LogId::new("events-2025-01")
LogId::new("events-2025-02")
LogId::new("events-2025-03")

// Rotate logs monthly/daily
```

### Best Practices

**DO:**
- ✅ Use descriptive, meaningful names
- ✅ Group related data in one log
- ✅ Use partitions for scalability, not multiple logs
- ✅ Plan retention strategy upfront
- ✅ Document log purpose and schema
- ✅ Use consistent naming convention

**DON'T:**
- ❌ Create too many logs (thousands)
- ❌ Use logs for temporary data (use TTL)
- ❌ Change log config frequently
- ❌ Delete and recreate logs with same name
- ❌ Use special characters in names
- ❌ Create log per user/tenant (use keys instead)

### Comparison with Other Systems

```
┌─────────────────────────────────────────────────┐
│   LogId Equivalents                             │
├─────────────────────────────────────────────────┤
│                                                 │
│  Pyralog:       LogId                              │
│  Kafka:      Topic                              │
│  Pulsar:     Topic                              │
│  Kinesis:    Stream                             │
│  EventHub:   Event Hub                          │
│  RabbitMQ:   Queue                              │
│                                                 │
└─────────────────────────────────────────────────┘
```

### Example: Complete Log Setup

```rust
use dlog_client::{PyralogClient, LogConfig, RetentionConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let client = PyralogClient::connect("localhost:9092").await?;
    
    // 1. Create log
    let log_id = "user-events";
    client.create_log_with_config(
        log_id,
        LogConfig {
            partition_count: 10,
            replication_factor: 3,
            retention: RetentionConfig::Time(Duration::from_days(7)),
            min_in_sync_replicas: 2,
            partitioning_mode: PartitioningMode::Dynamic,
            ..Default::default()
        },
    ).await?;
    
    // 2. Write to log
    for i in 0..1000 {
        client.produce(
            log_id,
            Record::new(
                Some(format!("user-{}", i % 100).into_bytes()),
                format!("event-{}", i).into_bytes(),
            ),
        ).await?;
    }
    
    // 3. Read from log
    let records = client.consume(log_id, LogOffset::ZERO, 100).await?;
    println!("Read {} records", records.len());
    
    // 4. Get log stats
    let metadata = client.get_log_metadata(log_id).await?;
    println!("Log has {} partitions", metadata.partition_count);
    
    // 5. Monitor log
    let stats = client.get_log_stats(log_id).await?;
    println!("Total records: {}", stats.record_count);
    println!("Total size: {} bytes", stats.size_bytes);
    println!("Write rate: {} records/sec", stats.write_rate);
    
    Ok(())
}
```

---

## Partitions

### Overview

A **Partition** is a physical shard of a log. Each log has one or more partitions.

```
Log "user-events" with 4 partitions:
  ├─ Partition 0: Records with hash(key) % 4 == 0
  ├─ Partition 1: Records with hash(key) % 4 == 1
  ├─ Partition 2: Records with hash(key) % 4 == 2
  └─ Partition 3: Records with hash(key) % 4 == 3
```

### Purpose

Partitions provide:
- **Parallelism**: Multiple partitions → multiple leaders → distributed writes
- **Scalability**: More partitions → higher throughput
- **Ordering**: Records in same partition are ordered
- **Distribution**: Data spread across cluster

### Partition Identifier

```rust
pub struct PartitionId(u32);

// Full identifier includes log
pub struct FullPartitionId {
    pub log_id: LogId,
    pub partition_id: u32,
}

// Example: "user-events" partition 3
FullPartitionId {
    log_id: LogId::new("user-events"),
    partition_id: 3,
}
```

See [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md) for details on partition management.

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

### Key Properties

**Key:**
- Optional but recommended
- Used for partitioning (hash routing)
- Used for ordering (same key → same partition → ordered)
- Used for compaction (in key-based compaction)

**Value:**
- Required
- Arbitrary byte array
- Can be any format (JSON, Protobuf, Avro, etc.)

**Headers:**
- Optional metadata
- Key-value pairs
- Not used for routing

### Creating Records

```rust
// Minimal record (no key)
let record = Record::new(
    None,
    b"Hello, Pyralog!".to_vec(),
);

// Record with key
let record = Record::new(
    Some(b"user-123".to_vec()),
    b"login event".to_vec(),
);

// Record with headers
let record = Record::new(
    Some(b"user-123".to_vec()),
    b"login event".to_vec(),
)
.with_header("source", "web-app")
.with_header("version", "1.0")
.with_timestamp(SystemTime::now());
```

See [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for advanced partitioning strategies.

---

## Offsets

### Overview

An **Offset** is the position of a record within a partition.

```rust
pub struct LogOffset(u64);

// Examples:
LogOffset::ZERO     // Start of partition
LogOffset::new(42)  // Specific position
LogOffset::MAX      // End of partition
```

### Types of Offsets

**1. Server-Assigned Offset (Primary)**
```
Partition has sequential offsets:
  Offset 0: First record
  Offset 1: Second record
  Offset 2: Third record
  ...
```

**2. Epoch-Offset (EpochOffset)**
```rust
pub struct EpochOffset {
    pub epoch: Epoch,
    pub offset: u64,
}

// Example: Epoch 5, Offset 1000
EpochOffset { epoch: 5, offset: 1000 }
```

**3. Virtual LSN (VLSN)**
```
Client-managed sequence number:
  VLSN 1000, 1001, 1002...
  Used for routing and ordering
```

See [EPOCHS.md](EPOCHS.md) for epoch details.

---

## Epochs

### Overview

An **Epoch** is a generation number for partition leadership.

```rust
pub struct Epoch(u64);
```

Each time a partition gets a new leader, the epoch increments:
```
Leader N1, Epoch 1: Offsets 0-999
Leader N2, Epoch 2: Offsets 1000-1999  (after failover)
Leader N3, Epoch 3: Offsets 2000-...   (after another failover)
```

### Purpose

Epochs enable:
- **Safe failover**: Prevent duplicate writes
- **Fast writes**: Decouple offset assignment from consensus
- **Recovery**: Know which records are from which leader

See [EPOCHS.md](EPOCHS.md) for comprehensive details.

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

Consumer groups provide:
- **Load balancing**: Partitions distributed among consumers
- **Fault tolerance**: Partition reassignment on consumer failure
- **Exactly-once**: Commit tracking per group

### Example

```
Log "events" with 4 partitions, Consumer Group "analytics":
  Consumer A: Assigned partitions [0, 1]
  Consumer B: Assigned partitions [2, 3]
  
If Consumer A fails:
  Consumer B: Reassigned all partitions [0, 1, 2, 3]
```

See [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for commit patterns.

---

## Replication

### Overview

**Replication** ensures data durability by maintaining copies across multiple nodes.

```
Partition 0 (RF=3):
  Leader: Node 1     (accepts writes)
  Follower: Node 2   (replica)
  Follower: Node 3   (replica)
```

### Key Concepts

**Replication Factor (RF):**
- Number of copies (including leader)
- Example: RF=3 means 1 leader + 2 followers

**In-Sync Replicas (ISR):**
- Followers that are caught up with leader
- Used for quorum calculations

**Quorums:**
- Write quorum: Minimum replicas for write to succeed
- Read quorum: Minimum replicas for consistent read

See [ARCHITECTURE.md](ARCHITECTURE.md) for replication details.

---

## Summary

**Core concepts hierarchy:**

```
Cluster
  └─ Logs (LogId)
       └─ Partitions
            └─ Records (with offsets)
                 └─ Replicated across nodes
```

**Key takeaways:**

1. **LogId** = Logical stream (like Kafka topic)
2. **Partition** = Physical shard for parallelism
3. **Record** = Individual message
4. **Offset** = Position within partition
5. **Epoch** = Leader generation number
6. **Consumer Group** = Cooperative consumption
7. **Replication** = Data durability

**Learn more:**
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [EXAMPLES.md](EXAMPLES.md) - Code examples
- [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) - Advanced patterns
- [EPOCHS.md](EPOCHS.md) - Epochs in detail
- [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md) - Partition management

