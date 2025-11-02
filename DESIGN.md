# Pyralog Design Document

## Executive Summary

Pyralog is a high-performance distributed log system built in Rust, combining the best ideas from **Redpanda** and **LogDevice** to create a modern, fast, and reliable foundation for distributed systems.

## Research Summary

### Redpanda Insights

From the research, Redpanda's key innovations include:

1. **Thread-per-Core Architecture**: Eliminates lock contention and maximizes CPU utilization
2. **Write Caching**: Acknowledges data in memory before persisting to disk, reducing write latency by up to 98%
3. **Zero External Dependencies**: Built-in Raft consensus eliminates the need for ZooKeeper
4. **Kafka Protocol Compatibility**: Drop-in replacement for existing Kafka deployments
5. **Modern C++ Implementation**: Leverages modern language features for performance

### LogDevice Insights

From the research, LogDevice's key innovations include:

1. **Flexible Quorums**: Configurable read/write quorums for availability/consistency tradeoffs
2. **CopySet Replication**: Reduces probability of data loss through intelligent replica placement
3. **Non-deterministic Record Placement**: Maintains high availability during failures
4. **Hierarchical Storage**: Multi-tier storage for cost optimization
5. **Paxos-based Consensus**: Strong consistency with flexible quorum sizes

## Pyralog's Design Philosophy

Pyralog synthesizes these ideas into a cohesive design:

### 1. Performance First

- **Write Caching** (from Redpanda): In-memory buffering with configurable durability
- **Zero-Copy I/O**: Memory-mapped files for efficient data access
- **Async Architecture**: Built on Tokio for maximum concurrency
- **Batch Processing**: Amortize overhead across multiple operations

### 2. Reliability and Consistency

- **Raft Consensus** (from Redpanda): Simple, understandable consistency model
- **Flexible Quorums** (from LogDevice): Configurable consistency/availability tradeoffs
- **CopySet Replication** (from LogDevice): Intelligent replica placement
- **Write-Ahead Logging**: Durability guarantees

### 3. Operational Simplicity

- **No External Dependencies**: Everything in one binary
- **Self-Healing**: Automatic recovery from failures
- **Observable**: Rich metrics and monitoring
- **Cloud-Native**: Kubernetes-ready

### 4. Scalability

- **Horizontal Scaling**: Add nodes to increase capacity
- **Tiered Storage**: Offload cold data to object storage
- **Partitioning**: Parallel processing across partitions
- **Load Balancing**: Even distribution of work

## Key Design Decisions

### Storage Engine

**Decision**: Segment-based log-structured storage with sparse indexes

**Rationale**:
- Sequential writes are faster than random writes on all storage media
- Immutable segments enable safe concurrent reads
- Sparse indexes balance memory usage and lookup performance
- Memory-mapped I/O provides zero-copy access

**Inspired by**: Both Redpanda and LogDevice use log-structured storage

### Consensus Protocol

**Decision**: Raft consensus for cluster coordination

**Rationale**:
- Simpler to understand and implement than Paxos
- Proven in production (etcd, Consul)
- Strong consistency guarantees
- Fast leader election

**Inspired by**: Redpanda's Raft-based approach

### Replication Strategy

**Decision**: Flexible quorums with CopySet selection

**Rationale**:
- Allows tuning for different use cases (write-heavy, read-heavy)
- CopySet replication reduces data loss probability
- ISR tracking maintains high availability
- Compatible with multiple durability levels

**Inspired by**: LogDevice's flexible quorum model

### Write Path Optimization

**Decision**: Configurable write caching with batching

**Rationale**:
- Dramatically reduces write latency for latency-sensitive applications
- Allows users to choose durability/latency tradeoff
- Batching amortizes I/O overhead
- Compatible with strong durability when needed

**Inspired by**: Redpanda's write caching feature

### Protocol Design

**Decision**: Kafka-compatible API with native binary protocol

**Rationale**:
- Kafka is the de facto standard for distributed logs
- Enables easy migration from Kafka
- Binary protocol is more efficient than text-based protocols
- Extensible for future features

**Inspired by**: Redpanda's Kafka compatibility

## Performance Targets

Based on the research and modern hardware capabilities:

| Metric | Target | Inspiration |
|--------|--------|-------------|
| Write Latency (p99) | < 1ms | Redpanda: sub-ms with caching |
| Read Latency (p99) | < 0.5ms | Memory-mapped I/O |
| Throughput | > 1M msg/sec/node | Redpanda achieves this |
| Replication Lag | < 100ms | LogDevice targets |
| Leader Election | < 300ms | Raft standard |

## Trade-offs

### 1. Consistency vs. Availability

**Choice**: Configurable
- Strong consistency with W=R=RF
- High availability with W=1, R=RF
- Balanced with majority quorums

### 2. Latency vs. Durability

**Choice**: Configurable via write cache
- Ultra-low latency: Large write cache, async flush
- Strong durability: Small cache, sync writes
- Balanced: Medium cache with periodic sync

### 3. Memory vs. Disk

**Choice**: Memory-mapped I/O when beneficial
- Enable mmap for read-heavy workloads
- Disable for write-heavy workloads
- OS page cache handles most cases well

### 4. Complexity vs. Features

**Choice**: Start simple, add complexity when needed
- Core features first (MVP)
- Advanced features later (compaction, transactions)
- Keep architecture extensible

## Innovation Points

While inspired by existing systems, Pyralog adds:

1. **Rust Safety**: Memory safety without garbage collection overhead
2. **Modern Async**: Native async/await for cleaner code
3. **Composable Architecture**: Clean module boundaries
4. **Cloud-First**: Designed for containerized deployments from day one
5. **Observable by Default**: Metrics and tracing built-in

## Implementation Phases

### Phase 1: Core Foundation ✅
- Basic log abstraction
- Storage engine
- Raft consensus
- Simple replication
- Protocol layer

### Phase 2: Production Readiness
- Network protocol implementation
- Full Kafka compatibility
- Monitoring and metrics
- Administration tools
- Performance tuning

### Phase 3: Advanced Features
- Log compaction
- Transactions
- Multi-DC replication
- Tiered storage (production)
- Schema registry

### Phase 4: Ecosystem
- Client SDKs
- Kubernetes operator
- Cloud integrations
- Monitoring dashboards
- Migration tools

## Success Criteria

Pyralog will be considered successful when:

1. **Performance**: Matches or exceeds Kafka/Redpanda benchmarks
2. **Reliability**: 99.99% uptime in production
3. **Adoption**: Used in production by multiple organizations
4. **Compatibility**: Seamless Kafka migration experience
5. **Community**: Active contributor base

## Conclusion

Pyralog represents a synthesis of proven distributed systems techniques (Redpanda, LogDevice) implemented in a modern systems language (Rust). By focusing on performance, reliability, and operational simplicity, Pyralog aims to be the distributed log of choice for the next generation of applications.

The modular architecture ensures that each component can be optimized independently while maintaining clean interfaces. The flexible configuration options allow users to tune the system for their specific workload characteristics.

Most importantly, Pyralog is built on solid theoretical foundations (Raft, flexible quorums, log-structured storage) that have been proven in large-scale production systems.

