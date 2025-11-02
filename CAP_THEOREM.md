# CAP Theorem and Pyralog

Understanding consistency, availability, and partition tolerance in Pyralog's design.

## Table of Contents

1. [CAP Theorem Overview](#cap-theorem-overview)
2. [Pyralog's Position in CAP Space](#dlogs-position-in-cap-space)
3. [Flexible Quorums and CAP](#flexible-quorums-and-cap)
4. [Practical Tradeoffs](#practical-tradeoffs)
5. [Configuration Examples](#configuration-examples)
6. [Comparison with Other Systems](#comparison-with-other-systems)
7. [PACELC Extension](#pacelc-extension)

---

## CAP Theorem Overview

### The CAP Theorem

Formulated by Eric Brewer in 2000, the CAP theorem states that a distributed system can provide at most **two** of the following three guarantees:

**C - Consistency**: Every read receives the most recent write or an error
**A - Availability**: Every request receives a response (without guarantee it's the latest)
**P - Partition Tolerance**: System continues despite network partitions

### Visual Representation

```
        Consistency
           /\
          /  \
         /    \
        /  CP  \
       /   |    \
      /    |     \
     /     |      \
    /------|-------\
   /   CA  |   AP   \
  /________|_________\
Availability    Partition
                Tolerance
```

### The Reality

In distributed systems, **network partitions are inevitable**. Therefore, the real choice is between:
- **CP**: Consistency + Partition Tolerance (sacrifice availability)
- **AP**: Availability + Partition Tolerance (sacrifice consistency)

### Important Clarifications

1. **CAP is about 100% guarantees**: You can have "pretty good" availability and consistency
2. **Choice is per-operation**: Different operations can make different tradeoffs
3. **Partition tolerance is mandatory**: Networks fail, so P is not optional
4. **The choice is: C or A during partitions**: Normal operation can have both

---

## Pyralog's Position in CAP Space

### Configurable CAP Profile

Pyralog's unique approach: **Let users choose their position on the CAP spectrum** through flexible quorums.

```
Traditional Systems:
- Kafka: CP (strong consistency)
- Cassandra: AP (eventual consistency)

Pyralog:
- Configurable: From CP to AP based on quorum settings
```

### Key Design Principles

1. **Flexible Quorums**: Configure W (write quorum) and R (read quorum)
2. **Tunable Consistency**: Choose consistency level per operation
3. **Graceful Degradation**: System adapts to failures
4. **Epoch-based Safety**: Prevents split-brain regardless of CAP choice

### Pyralog's CAP Triangle

```
         Strong Consistency
              /\
             /  \
            /    \
           / W=RF \
          /  R=RF  \
         /          \
        /   Pyralog     \
       /  (tunable)   \
      /                \
     /   W=majority    \
    /    R=majority     \
   /____________________\
High Availability    Low Latency
  (W=1, R=RF)         (W=1, R=1)
```

---

## Flexible Quorums and CAP

### Quorum Formula

```
W + R > RF

Where:
- W  = Write quorum (nodes that must acknowledge write)
- R  = Read quorum (nodes that must respond to read)
- RF = Replication factor (total copies)
```

**When W + R > RF**: Guaranteed overlap, ensuring consistency
**When W + R ≤ RF**: No guaranteed overlap, eventual consistency

### CAP Configurations

#### 1. Strong Consistency (CP)

```rust
QuorumConfig {
    replication_factor: 3,
    write_quorum: 3,      // All replicas
    read_quorum: 1,       // Any replica
}

// W + R = 4 > RF = 3 ✓ (Consistent)
```

**Guarantees**:
- ✅ Reads always see latest write
- ✅ Linearizable consistency
- ❌ Writes block if any node unavailable
- ❌ Lower availability during failures

**Use cases**:
- Financial transactions
- Inventory management
- Critical configuration data

#### 2. High Availability (AP)

```rust
QuorumConfig {
    replication_factor: 3,
    write_quorum: 1,      // Any replica
    read_quorum: 1,       // Any replica
}

// W + R = 2 < RF = 3 ✗ (Eventually consistent)
```

**Guarantees**:
- ✅ Writes succeed as long as one node available
- ✅ Reads succeed as long as one node available
- ✅ High availability
- ❌ Reads may return stale data
- ❌ Eventual consistency only

**Use cases**:
- Logging and metrics
- Social media feeds
- Analytics data
- Non-critical events

#### 3. Balanced (Majority)

```rust
QuorumConfig {
    replication_factor: 3,
    write_quorum: 2,      // Majority
    read_quorum: 2,       // Majority
}

// W + R = 4 > RF = 3 ✓ (Consistent)
```

**Guarantees**:
- ✅ Consistent reads
- ✅ Tolerates one node failure
- ✅ Good balance of consistency and availability
- ⚠️ Requires majority available

**Use cases**:
- Most production workloads
- E-commerce transactions
- User state management
- General-purpose logging

---

## Practical Tradeoffs

### Consistency Levels

Pyralog supports multiple consistency levels through quorum configuration:

| Level | W | R | Latency | Availability | Consistency |
|-------|---|---|---------|--------------|-------------|
| **Strong** | RF | 1 | High | Low | Linearizable |
| **Majority** | ⌈RF/2⌉ | ⌈RF/2⌉ | Medium | Medium | Strong |
| **One** | 1 | 1 | Low | High | Eventual |
| **Quorum** | ⌈RF/2⌉ | ⌈RF/2⌉ | Medium | Medium | Quorum |

### During Network Partition

#### Scenario: 3-node cluster, network splits 2 vs 1

**With Majority Quorums (W=2, R=2)**:
```
Partition A: [Node 1, Node 2] - Has majority
Partition B: [Node 3]         - No majority

Result:
- Partition A: ✅ Accepts writes, ✅ Accepts reads
- Partition B: ❌ Rejects writes, ❌ Rejects reads
- Behavior: CP (Consistency preserved, Availability sacrificed in minority)
```

**With W=1, R=1 (High Availability)**:
```
Partition A: [Node 1, Node 2]
Partition B: [Node 3]

Result:
- Partition A: ✅ Accepts writes, ✅ Accepts reads
- Partition B: ✅ Accepts writes, ✅ Accepts reads
- Behavior: AP (Availability preserved, Consistency sacrificed)
- Issue: Split-brain! Both partitions accept writes
```

### Pyralog's Split-Brain Prevention

Even with AP configuration, Pyralog uses **epochs** to prevent true split-brain:

```rust
pub struct Record {
    pub offset: LogOffset,
    pub epoch: Epoch,    // Prevents ambiguity
    // ...
}
```

During partition with W=1, R=1:
1. Old leader (in minority) tries to write
2. Epoch system detects it's not current leader
3. Write rejected even though quorum would allow it
4. New leader (in majority) can write with new epoch
5. When partition heals, old writes are discarded

**Result**: AP-like availability with CP-like safety guarantees!

---

## Configuration Examples

### Example 1: Financial System (CP)

```rust
// Maximum consistency, tolerant of failures
let config = QuorumConfig {
    replication_factor: 5,
    write_quorum: 5,        // All replicas
    read_quorum: 1,         // Any replica
};

// Guarantees:
// - Every write goes to all 5 replicas
// - If any replica is down, writes fail
// - Reads are fast (any replica)
// - No stale reads possible
```

**CAP Profile**: CP
**Availability**: 99.9% (5 nines require all nodes)
**Consistency**: Linearizable

### Example 2: Logging System (AP)

```rust
// Maximum availability, eventual consistency
let config = QuorumConfig {
    replication_factor: 3,
    write_quorum: 1,        // Any replica
    read_quorum: 1,         // Any replica
};

// Guarantees:
// - Writes succeed if any node is up
// - Reads succeed if any node is up
// - May read stale data
// - Eventually consistent
```

**CAP Profile**: AP (with epoch safety)
**Availability**: 99.99% (only need 1 of 3 nodes)
**Consistency**: Eventual

### Example 3: E-commerce (Balanced)

```rust
// Balanced consistency and availability
let config = QuorumConfig {
    replication_factor: 3,
    write_quorum: 2,        // Majority
    read_quorum: 2,         // Majority
};

// Guarantees:
// - Tolerates 1 node failure
// - Consistent reads
// - Good availability
// - Reasonable latency
```

**CAP Profile**: CP (with good availability)
**Availability**: 99.95% (need 2 of 3 nodes)
**Consistency**: Strong

---

## Comparison with Other Systems

### Kafka

**CAP Position**: CP

```
- Uses ISR (In-Sync Replicas)
- min.insync.replicas = minimum ISR size
- Writes fail if ISR drops below minimum
- Strong consistency guarantees
- May sacrifice availability
```

**Pyralog Equivalent**:
```rust
QuorumConfig {
    replication_factor: 3,
    write_quorum: 2,        // Similar to min.insync.replicas=2
    read_quorum: 1,
}
```

### Cassandra

**CAP Position**: AP

```
- Designed for availability
- Eventual consistency by default
- Tunable consistency levels
- Never rejects writes (in theory)
- May return stale data
```

**Pyralog Equivalent**:
```rust
QuorumConfig {
    replication_factor: 3,
    write_quorum: 1,
    read_quorum: 1,
}
```

### LogDevice

**CAP Position**: Tunable (like Pyralog)

```
- Flexible quorums
- Configurable W and R
- Epoch-based safety
- Similar to Pyralog's approach
```

**Direct Mapping**: Pyralog's quorum system inspired by LogDevice

### Redis

**CAP Position**: CP or AP depending on mode

```
- Single-master: CP (strong consistency)
- Redis Cluster: AP (async replication)
- Redis Sentinel: CP with eventual failover
```

### Pyralog vs Others

| System | CAP | Configurable | Epochs | Quorums |
|--------|-----|--------------|--------|---------|
| **Pyralog** | **Tunable** | **✅** | **✅** | **Flexible** |
| Kafka | CP | ⚠️ Limited | ❌ | ISR-based |
| Cassandra | AP | ✅ | ❌ | Flexible |
| LogDevice | Tunable | ✅ | ✅ | Flexible |
| Pulsar | CP | ⚠️ Limited | ❌ | Quorum |

---

## PACELC Extension

CAP theorem has been extended to **PACELC**:

```
PAC: During Partition, choose between Availability and Consistency
ELC: Else (no partition), choose between Latency and Consistency
```

### Pyralog's PACELC Profile

**During Partition (PA/C)**:
```rust
// Choose based on quorum config
if write_quorum + read_quorum > replication_factor {
    // PC: Favor Consistency
    return Err("Quorum not available");
} else {
    // PA: Favor Availability
    return Ok(write_with_eventual_consistency);
}
```

**Else, Normal Operation (EL/C)**:
```rust
// Pyralog favors Latency via write caching
if cache_enabled {
    // EL: Low Latency
    cache.buffer(record);
    return Ok(offset);
} else {
    // EC: Strong Consistency
    storage.sync_write(record).await?;
    return Ok(offset);
}
```

### PACELC Configuration

#### PA/EL (Availability + Latency)

```rust
PyralogConfig {
    replication: QuorumConfig {
        replication_factor: 3,
        write_quorum: 1,
        read_quorum: 1,
    },
    storage: StorageConfig {
        cache_enabled: true,
        cache_size: 64 * 1024 * 1024,
        sync_on_write: false,
    },
}
```

**Profile**: Maximum availability and lowest latency
**Use case**: High-throughput logging, metrics

#### PC/EC (Consistency + Consistency)

```rust
PyralogConfig {
    replication: QuorumConfig {
        replication_factor: 3,
        write_quorum: 3,
        read_quorum: 1,
    },
    storage: StorageConfig {
        cache_enabled: false,
        sync_on_write: true,
    },
}
```

**Profile**: Maximum consistency
**Use case**: Financial transactions, critical data

#### PC/EL (Consistency + Latency) - **Pyralog's Sweet Spot**

```rust
PyralogConfig {
    replication: QuorumConfig {
        replication_factor: 3,
        write_quorum: 2,    // Consistency during partition
        read_quorum: 2,
    },
    storage: StorageConfig {
        cache_enabled: true,     // Latency during normal operation
        cache_size: 16 * 1024 * 1024,
        sync_on_write: false,
    },
}
```

**Profile**: Consistent yet fast
**Use case**: Most production workloads

---

## Recommendations

### By Use Case

#### Financial/Billing Systems
```rust
✅ Choose: CP (Strong Consistency)
QuorumConfig::strong_consistency(replication_factor: 5)

Rationale:
- Accuracy > Availability
- Can't afford stale reads
- Acceptable to reject requests during failures
```

#### Logging/Monitoring
```rust
✅ Choose: AP (High Availability)
QuorumConfig::high_availability(replication_factor: 3)

Rationale:
- Availability > Consistency
- Some data loss acceptable
- Must not block application
```

#### E-commerce
```rust
✅ Choose: Balanced (Majority Quorums)
QuorumConfig::majority(replication_factor: 3)

Rationale:
- Need consistency for orders
- Need availability for user experience
- Good balance
```

#### Real-time Analytics
```rust
✅ Choose: AP or Balanced
QuorumConfig::read_optimized(replication_factor: 3)

Rationale:
- Slight staleness acceptable
- Need high read throughput
- Eventually consistent is fine
```

### By SLA Requirements

| Availability SLA | Recommended Config | CAP |
|------------------|-------------------|-----|
| 99.9% (3 nines) | RF=3, W=3, R=1 | CP |
| 99.95% | RF=3, W=2, R=2 | CP |
| 99.99% (4 nines) | RF=3, W=1, R=1 | AP |
| 99.999% (5 nines) | RF=5, W=1, R=1 | AP |

### By Latency Requirements

| Latency Target | Write Cache | Sync | Quorum |
|---------------|-------------|------|--------|
| < 1ms (p99) | Large (64MB) | No | W=1 |
| < 5ms (p99) | Medium (16MB) | No | W=2 |
| < 10ms (p99) | Small (8MB) | No | W=2 |
| < 50ms (p99) | Disabled | Yes | W=3 |

---

## Advanced Topics

### Linearizability

**Definition**: Operations appear to execute atomically in some order consistent with real-time

**Pyralog Achieves Linearizability** when:
```rust
write_quorum == replication_factor && read_quorum == 1
// OR
write_quorum == 1 && read_quorum == replication_factor
```

**Example**:
```rust
let config = QuorumConfig {
    replication_factor: 3,
    write_quorum: 3,  // Write to all
    read_quorum: 1,   // Read from any
};

// This guarantees:
// 1. All writes are on all replicas before acknowledgment
// 2. Any read sees the latest write
// 3. Linearizable consistency
```

### Causal Consistency

**Definition**: If operation A causally precedes B, all nodes see A before B

**Pyralog Provides Causal Consistency** through epochs:
```rust
pub struct Record {
    pub epoch: Epoch,
    pub offset: LogOffset,
    // ...
}

// Epoch establishes causality:
// If record_a.epoch < record_b.epoch
// Then record_a causally precedes record_b
```

### Session Consistency

**Definition**: A client's reads reflect its writes

**Pyralog Guarantees** through sticky sessions:
```rust
// Client tracks its latest written offset
pub struct ConsistentClient {
    last_write_offset: LogOffset,
}

impl ConsistentClient {
    pub async fn read(&self) -> Result<Record> {
        // Only read from replicas at or past last_write_offset
        self.client.read_from(self.last_write_offset).await
    }
}
```

---

## Monitoring CAP Metrics

### Key Metrics

```rust
// Consistency metrics
dlog_stale_reads_total         // Reads that returned old data
dlog_consistency_violations    // Detected inconsistencies

// Availability metrics
dlog_write_failures_total      // Failed writes
dlog_read_failures_total       // Failed reads
dlog_quorum_unavailable        // Quorum not available

// Latency metrics
dlog_write_latency_seconds     // Write latency
dlog_read_latency_seconds      // Read latency
```

### Alerting

```yaml
# High stale read rate (consistency issue)
- alert: HighStaleReads
  expr: rate(dlog_stale_reads_total[5m]) > 0.01
  
# Quorum frequently unavailable (availability issue)
- alert: QuorumUnavailable
  expr: rate(dlog_quorum_unavailable[5m]) > 0.05
  
# High write latency (latency issue)
- alert: HighWriteLatency
  expr: histogram_quantile(0.99, dlog_write_latency_seconds) > 0.1
```

---

## Conclusion

### Key Takeaways

1. **CAP is a spectrum, not binary**: Pyralog lets you choose your position
2. **P is mandatory**: Networks partition, so choose C or A
3. **Flexible quorums are powerful**: One size doesn't fit all
4. **Epochs add safety**: Even AP config gets split-brain protection
5. **Configuration matters**: Choose based on your requirements

### Pyralog's Philosophy

```
"Don't force users into one CAP box.
 Give them the tools to make informed tradeoffs.
 Provide safety guardrails (epochs).
 Let them optimize for their use case."
```

### Decision Framework

```
1. What's more important: Consistency or Availability?
2. Can you tolerate stale reads?
3. What latency is acceptable?
4. What's your availability target?
5. Configure quorums accordingly
```

### The Pyralog Advantage

Unlike systems that force you into CP (Kafka) or AP (Cassandra), **Pyralog adapts to your needs** while maintaining safety through epochs and flexible quorums inspired by LogDevice.

---

## Further Reading

- [Brewer's CAP Theorem (2000)](https://www.cs.berkeley.edu/~brewer/cs262b-2004/PODC-keynote.pdf)
- [CAP Twelve Years Later (2012)](https://www.infoq.com/articles/cap-twelve-years-later-how-the-rules-have-changed/)
- [PACELC Theorem](https://en.wikipedia.org/wiki/PACELC_theorem)
- [Pyralog Flexible Quorums](DESIGN.md#flexible-quorums)
- [Pyralog Epochs](EPOCHS.md)
- [LogDevice Consistency](https://engineering.fb.com/2017/08/31/core-infra/logdevice-a-distributed-data-store-for-logs/)

---

**Last Updated**: 2025-01-01
**Version**: 1.0
**Status**: Complete

*For questions about CAP tradeoffs in Pyralog, see [FAQ.md](FAQ.md) or open a GitHub discussion.*

