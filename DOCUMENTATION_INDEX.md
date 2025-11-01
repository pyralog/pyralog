# DLog Documentation Index

Complete guide to all DLog documentation.

## 📚 Quick Navigation

### Getting Started
- **[README](README.md)** - Project overview, features, quick introduction
- **[QUICK_START](QUICK_START.md)** - Get running in 5 minutes
- **[FAQ](FAQ.md)** - Frequently asked questions

### Learning DLog
- **[CORE_CONCEPTS](CORE_CONCEPTS.md)** - Fundamental concepts (LogId, Partitions, Records, Offsets, Epochs)
- **[ARCHITECTURE](ARCHITECTURE.md)** - Deep dive into system internals
- **[DESIGN](DESIGN.md)** - Design decisions and rationale
- **[PAPER](PAPER.md)** ⭐ - **Academic research paper** on DLog's novel contributions
- **[Blog Series](blog/README.md)** 🎯 - **5-part technical blog series** explaining DLog
  - [1. Introducing DLog](blog/1-introducing-dlog.md) - Why we need unified infrastructure
  - [2. Sparse Append Counter](blog/2-sparse-append-counter.md) - Novel persistent atomic primitive
  - [3. Distributed Coordinators](blog/3-distributed-coordinators.md) - Eliminating bottlenecks
  - [4. 28 Billion Ops/Sec](blog/4-28-billion-ops.md) - Architectural deep-dive
  - [5. Building in Rust](blog/5-rust-infrastructure.md) - Lessons learned
- **[DATA_PATH](DATA_PATH.md)** - Write and read path with detailed diagrams
- **[EXAMPLES](EXAMPLES.md)** - Practical code examples and patterns
- **[EPOCHS](EPOCHS.md)** - Understanding epochs and sequencers
- **[CAP_THEOREM](CAP_THEOREM.md)** - Consistency, availability, and tradeoffs
- **[DYNAMIC_PARTITIONS](DYNAMIC_PARTITIONS.md)** - Dynamic partition splitting and merging
- **[CLIENT_PARTITIONING_PATTERNS](CLIENT_PARTITIONING_PATTERNS.md)** - Client-side partitioning strategies (VLSN, hash-based, hierarchical)
- **[ADVANCED_FEATURES](ADVANCED_FEATURES.md)** - Future features from other systems

### Operations
- **[OPERATIONS](OPERATIONS.md)** - Deployment, configuration, and maintenance
- **[PERFORMANCE](PERFORMANCE.md)** - Performance tuning and optimization
- **[COMPARISON](COMPARISON.md)** - How DLog compares to alternatives
- **[TIKV_COMPARISON](TIKV_COMPARISON.md)** - Detailed comparison with TiKV

### Development
- **[CONTRIBUTING](CONTRIBUTING.md)** - How to contribute to DLog
- **[IMPLEMENTATION_PLAN](IMPLEMENTATION_PLAN.md)** - Complete implementation roadmap
- **[RUST_LIBRARIES](RUST_LIBRARIES.md)** - Recommended Rust crates and ecosystem
- **[CHANGELOG](CHANGELOG.md)** - Version history and release notes
- **[PROJECT_SUMMARY](PROJECT_SUMMARY.md)** - Complete project overview
- **[ADVANCED_FEATURES](ADVANCED_FEATURES.md)** - Advanced features roadmap

## 📖 Documentation by Topic

### Architecture & Design

#### Core Concepts
- [CORE_CONCEPTS.md](CORE_CONCEPTS.md)
  - LogId (logs/topics)
  - Partitions and sharding
  - Records and data model
  - Offsets and positioning
  - Epochs and generations
  - Consumer groups
  - Replication fundamentals
  - Naming conventions
  - Best practices
  - Multi-tenancy patterns

#### System Architecture
- [ARCHITECTURE.md](ARCHITECTURE.md)
  - Core components overview
  - Storage engine design
  - Consensus protocol (Raft)
  - Replication system
  - Network protocol
  - Performance optimizations
  - Failure scenarios

#### Design Philosophy
- [DESIGN.md](DESIGN.md)
  - Research summary (Redpanda + LogDevice)
  - Design philosophy
  - Key design decisions
  - Trade-offs analysis
  - Innovation points
  - Implementation phases

#### Research Paper
- [PAPER.md](PAPER.md) ⭐ **Academic research paper**
  - Abstract and contributions
  - Background and related work survey
  - Novel coordination primitives (Sparse Append Counter)
  - Distributed coordinators via Snowflake IDs
  - Dual Raft architecture
  - Configurable CopySet strategies
  - Percolator protocol integration
  - Performance evaluation and benchmarks
  - Comparison with Kafka, TiKV, Databend
  - Future research directions
  - 40+ pages, 12,000 words
  - 23 academic references

#### Epochs System
- [EPOCHS.md](EPOCHS.md)
  - What are epochs?
  - Why epochs matter
  - Implementation details
  - Usage examples
  - Failover scenarios
  - Best practices

#### Dynamic Partitions
- [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md)
  - Motivation and benefits
  - Static vs dynamic partitions
  - Partition splitting process
  - Partition merging process
  - Implementation details
  - Client-side routing
  - Configuration and policies
  - Comparison with TiKV regions
  - Migration strategy

#### Client Partitioning Patterns
- [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md)
  - Hash-based partitioning (default)
  - Virtual LSN (VLSN) partitioning
  - Hierarchical keys (multi-tenant)
  - Pattern comparison matrix
  - Implementation examples
  - Ordering guarantees
  - Performance characteristics
  - Best practices
  - **Sparse Append Counter pattern** (novel primitive) ⭐
    - **Persistent atomic counter** - Like `AtomicU64`, but crash-safe!
    - General-purpose building block for durable counters
    - Detailed comparison with mmap (performance, durability, scalability)
    - Write latency, recovery speed, disk usage analysis
    - Concurrency, portability, failure modes
    - Trade-off analysis and when to use each
    - SIGBUS risk explained
    - Could be extracted as standalone Rust crate
    - **Use Case Deep Dive: Snowflake IDs**
      - Twitter's distributed ID generator algorithm
      - 64-bit structure (timestamp + machine ID + sequence)
      - How Sparse Append Counter prevents duplicate IDs
      - 10 real-world use cases (Twitter, Discord, Instagram, etc.)
      - Discord message example with code
      - Variants (ULID, Instagram, MongoDB ObjectId)
      - Companies using Snowflake/similar approaches
  - Consumer commit strategies (per-partition vs VLSN)

#### Data Path
- [DATA_PATH.md](DATA_PATH.md)
  - Complete write path with diagrams
  - Complete read path with diagrams
  - Batch write optimization
  - Replication flow details
  - Failure scenarios
  - Performance optimizations
  - Step-by-step flows with ASCII diagrams

#### CAP Theorem
- [CAP_THEOREM.md](CAP_THEOREM.md)
  - CAP theorem explained
  - DLog's position in CAP space
  - Flexible quorums and CAP
  - Configuration examples (CP, AP, balanced)
  - Comparison with other systems
  - PACELC extension
  - Practical recommendations
  - Monitoring CAP metrics

#### Immutable Knowledge Databases
- [IMMUTABLE_KNOWLEDGE_DB.md](IMMUTABLE_KNOWLEDGE_DB.md) ⭐ **NEW: DLog for temporal knowledge systems**
  - Entity-Attribute-Value-Time (EAVT) model
  - Complete immutability and audit trails
  - Time-travel queries (query at any historical point)
  - ACID transactions for atomic fact assertions
  - Use cases: Scientific papers, legal documents, medical records, infrastructure as code
  - Comparison with Datomic, Crux, PostgreSQL
  - 50,000× faster than Datomic
  - Complete implementation patterns and query examples

#### Advanced Features
- [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) ⭐ **Updated with Percolator protocol**
  - **DLog's Architectural Advantages** (new section)
    - Sparse Append Counter primitive
    - Dual Raft clusters
    - Smart client pattern
    - Per-record CopySet optimization
  - **Transactions** ⭐ **NEW: Percolator protocol integration**
    - TiKV's Percolator protocol (MVCC, 2PC)
    - Distributed TSO (Snowflake-powered, 4B timestamps/sec)
    - Distributed Transaction Coordinators (4B tx/sec)
    - 8000x faster than TiKV, 40,000x faster than Kafka
    - Complete MVCC storage implementation
  - Log compaction
  - **Exactly-once semantics** ⭐ **EXPANDED: Complete deep-dive**
    - Three delivery guarantees (at-most-once, at-least-once, exactly-once)
    - Kafka's three-part solution (idempotent producers, transactions, offset commits)
    - Complete DLog implementation with Percolator + Snowflake IDs
    - Distributed session managers (4B sessions/sec)
    - Deduplication algorithm and cache management
    - Transactional writes with MVCC
    - Read committed consumers
    - End-to-end example code
    - Performance characteristics and trade-offs
  - **Stream processing** ⭐ **NEW: DataFusion + Polars integration**
    - Apache DataFusion (SQL stream processing)
    - Polars (DataFrame stream processing)
    - Apache Arrow native (zero-copy, SIMD)
    - Windowing (tumbling, sliding, session)
    - Stream-stream joins
    - Complete end-to-end examples
    - Performance benchmarks (10-100x faster than JVM)
    - State management with DLog changelog
  - Schema registry (Sparse Append Counter for schema IDs)
  - Consumer groups (Sparse Append Counter for generations)
  - Connectors framework
  - Change data capture (Sparse Append Counter for event IDs)
  - Multi-DC replication
  - **Time-travel queries** ⭐ **NEW: Hybrid Sparse + Arrow DataFusion index**
    - Two-tier indexing (segment stats + sparse checkpoints)
    - 10-100 MB RAM for billions of records
    - 2-5 ms query time
    - SQL time-travel queries via DataFusion
    - Lazy-loading from S3
    - Memory efficiency analysis
  - Observability features
  - **DLog as OpenTelemetry Backend** ⭐ **NEW: OTLP receiver + Arrow storage**
    - OTLP/gRPC receiver for traces/metrics/logs
    - Arrow schema for OpenTelemetry data
    - DataFusion SQL queries for trace analysis
    - 10-50x faster writes than Elasticsearch/Jaeger
  - **Advanced Analytics (from Databend)** ⭐ **NEW: Data warehouse features**
    - Semi-structured data (JSON querying)
    - External tables (zero-copy S3 queries)
    - Materialized views (100-1000x faster dashboards)
    - Inverted indexes (full-text search)
    - Bloom filters (1000x faster point queries)
    - Data clustering (30-50% better compression)
    - Virtual columns (zero storage overhead)
    - Zero-copy cloning (instant snapshots)
  - **Data Ingestion (from Databend)** ⭐ **NEW: High-performance data loading**
    - COPY INTO (bulk loading from S3/GCS/Azure/local files)
    - Streaming ingestion (Kafka/Kinesis/MQTT/HTTP)
    - Stage management (pre-load validation)
    - Schema inference (automatic schema detection)
    - Format support (JSON, CSV, Parquet, ORC, Avro, NdJson)
    - Compression support (gzip, snappy, zstd, lz4, brotli)
    - Data pipelines (automated ETL workflows)
    - 1-5 GB/sec bulk load throughput
    - 100K-1M events/sec streaming ingestion
  - **Universal Pattern**: Distributed Coordinators via Snowflake IDs
    - Extends to ALL coordinators in DLog
    - 28 billion ops/sec across 7 service types
  - Implementation roadmap

### User Guides

#### Getting Started
- [QUICK_START.md](QUICK_START.md)
  - Installation
  - Single node setup
  - Three node cluster
  - Docker deployment
  - Basic operations
  - Configuration
  - Troubleshooting

#### Code Examples
- [EXAMPLES.md](EXAMPLES.md)
  - Basic usage
  - Configuration
  - Advanced patterns
  - Performance tuning
  - Load testing
  - Best practices

#### FAQ
- [FAQ.md](FAQ.md)
  - General questions
  - Architecture questions
  - Performance questions
  - Operational questions
  - Compatibility questions
  - Development questions
  - Configuration questions
  - Feature questions
  - Troubleshooting
  - Security questions
  - Licensing questions
  - Community questions

### Operations

#### Deployment & Operations
- [OPERATIONS.md](OPERATIONS.md)
  - System requirements
  - OS tuning
  - Installation methods
  - Cluster deployment
  - Configuration reference
  - Monitoring setup
  - Backup and recovery
  - Scaling strategies
  - Troubleshooting guide
  - Security hardening
  - Performance tuning

#### Performance Guide
- [PERFORMANCE.md](PERFORMANCE.md)
  - Performance characteristics
  - Benchmarking methods
  - Optimization techniques
  - Configuration tuning
  - Monitoring performance
  - Common bottlenecks
  - Best practices
  - Performance checklist

### Comparisons

#### System Comparison
- [COMPARISON.md](COMPARISON.md)
  - DLog vs Kafka
  - DLog vs Redpanda
  - DLog vs LogDevice
  - DLog vs Pulsar
  - Performance comparison
  - Feature matrix
  - Migration paths
  - Decision guide

- [TIKV_COMPARISON.md](TIKV_COMPARISON.md)
  - Multi-Raft architecture comparison
  - Data model: log vs key-value
  - Transaction support differences
  - Performance characteristics
  - Use case analysis
  - When to use which system
  - Hybrid deployment patterns
  - Complementary architectures

### Development

#### Contributing
- [CONTRIBUTING.md](CONTRIBUTING.md)
  - Code of conduct
  - Getting started
  - Development setup
  - Making changes
  - Testing guide
  - Code style guidelines
  - Submitting changes
  - Review process
  - Advanced topics

#### Rust Libraries
- [RUST_LIBRARIES.md](RUST_LIBRARIES.md)
  - Core dependencies (Tokio, Serde, etc.)
  - Async runtime recommendations
  - Serialization libraries (bincode, protobuf, JSON)
  - Networking (Tonic gRPC, Hyper HTTP, Quinn QUIC)
  - Storage & I/O (memmap2, tokio-uring, RocksDB)
  - Consensus implementations (Raft, Openraft)
  - Data structures (DashMap, parking_lot, crossbeam)
  - Error handling (thiserror, anyhow)
  - Observability (tracing, prometheus, opentelemetry)
  - Testing frameworks (tokio-test, proptest, criterion, mockall)
  - Performance tools (pprof, jemalloc)
  - Utilities (clap, config, chrono)
  - Complete Cargo.toml example
  - Library selection guidelines
  - Common patterns and best practices

#### Implementation Plan
- [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md)
  - 6-phase development roadmap (16-23 weeks)
  - Phase 1: Foundation (project structure, core types)
  - Phase 2: Core Storage (segments, index, cache, mmap)
  - Phase 3: Consensus & Replication (Raft, cluster, CopySet)
  - Phase 4: Epochs & Smart Clients (high throughput, metadata)
  - Phase 5: Production Hardening (observability, recovery, tests)
  - Phase 6: Advanced Features (tiered storage, compaction)
  - Complete code examples for each component
  - Testing strategy (unit, integration, chaos)
  - Deployment strategy (Docker, Kubernetes)
  - Success criteria and milestones
  - Risk management
  - Timeline estimates

#### Release History
- [CHANGELOG.md](CHANGELOG.md)
  - Version history
  - Release notes
  - Feature timeline
  - Roadmap

### Reference

#### Project Overview
- [README.md](README.md)
  - Features overview
  - Architecture diagram
  - Core components
  - Installation
  - Usage examples
  - Roadmap
  - Contributing

#### Project Summary
- [PROJECT_SUMMARY.md](PROJECT_SUMMARY.md)
  - What was built
  - Key features
  - Architecture highlights
  - Innovation points
  - Documentation summary
  - Production readiness
  - Use cases
  - Technical achievements

## 🎯 Documentation by Use Case

### "I want to learn about DLog"
1. Start with [README.md](README.md) for overview
2. Read [CORE_CONCEPTS.md](CORE_CONCEPTS.md) to understand fundamentals (LogId, Partitions, etc.)
3. Read [DESIGN.md](DESIGN.md) for philosophy
4. Check [ARCHITECTURE.md](ARCHITECTURE.md) for details
5. Review [COMPARISON.md](COMPARISON.md) vs alternatives

### "I want to deploy DLog"
1. Read [QUICK_START.md](QUICK_START.md) for basics
2. Follow [OPERATIONS.md](OPERATIONS.md) for production
3. Check [PERFORMANCE.md](PERFORMANCE.md) for tuning
4. Refer to [FAQ.md](FAQ.md) for common issues

### "I want to develop with DLog"
1. Start with [QUICK_START.md](QUICK_START.md)
2. Study [EXAMPLES.md](EXAMPLES.md) for patterns
3. Read API documentation (inline docs)
4. Check [FAQ.md](FAQ.md) for questions

### "I want to contribute to DLog"
1. Read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) to understand internals
3. Check [CHANGELOG.md](CHANGELOG.md) for roadmap
4. Study existing code and tests

### "I want to migrate from Kafka"
1. Read [COMPARISON.md](COMPARISON.md) for differences
2. Check [FAQ.md](FAQ.md) for compatibility
3. Review [OPERATIONS.md](OPERATIONS.md) for deployment
4. Follow migration guide in [COMPARISON.md](COMPARISON.md)

### "I want to optimize performance"
1. Study [PERFORMANCE.md](PERFORMANCE.md) comprehensively
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) for internals
3. Check [OPERATIONS.md](OPERATIONS.md) for tuning
4. Try examples from [EXAMPLES.md](EXAMPLES.md)

### "I want to implement DLog"
1. Read [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) for complete roadmap
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) to understand the design
3. Check [RUST_LIBRARIES.md](RUST_LIBRARIES.md) for recommended crates
4. Follow phase-by-phase approach with code examples
5. Use [CONTRIBUTING.md](CONTRIBUTING.md) for development setup

### "I want to understand DLog's research contributions"
1. Read [PAPER.md](PAPER.md) ⭐ - comprehensive research paper
2. Study [ARCHITECTURE.md](ARCHITECTURE.md) for architectural innovations
3. Review [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for Sparse Append Counter details
4. Check [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) for distributed coordinators pattern
5. Read [TIKV_COMPARISON.md](TIKV_COMPARISON.md) and [COMPARISON.md](COMPARISON.md) for performance comparisons

### "I want to build an immutable knowledge database"
1. Read [IMMUTABLE_KNOWLEDGE_DB.md](IMMUTABLE_KNOWLEDGE_DB.md) ⭐ - complete guide
2. Study [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) for transactions and time-travel
3. Review [EXAMPLES.md](EXAMPLES.md) for implementation patterns
4. Check [ARCHITECTURE.md](ARCHITECTURE.md) for MVCC and storage details

## 📊 Documentation Statistics

- **Total Documents**: 31 markdown files
  - 25 main documentation files
  - 6 blog posts
- **Total Lines**: ~35,000+ lines of documentation
- **Coverage**:
  - ✅ Architecture and design
  - ✅ Research contributions and academic paper
  - ✅ Immutable knowledge databases and temporal systems
  - ✅ User guides and tutorials
  - ✅ Operations and deployment
  - ✅ Performance and optimization
  - ✅ Comparisons and migration
  - ✅ Development and contributing
  - ✅ FAQ and troubleshooting

## 🔍 Finding Information

### Search Tips

1. **By keyword**: Use your editor's search across all .md files
2. **By topic**: Use this index to find relevant documents
3. **By question**: Start with [FAQ.md](FAQ.md)
4. **By use case**: See "Documentation by Use Case" above

### Common Topics

| Topic | Primary Doc | Related Docs |
|-------|-------------|--------------|
| Core Concepts | [CORE_CONCEPTS.md](CORE_CONCEPTS.md) | [ARCHITECTURE.md](ARCHITECTURE.md), [EXAMPLES.md](EXAMPLES.md) |
| Research Paper | [PAPER.md](PAPER.md) ⭐ | [ARCHITECTURE.md](ARCHITECTURE.md), [DESIGN.md](DESIGN.md), [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) |
| Installation | [QUICK_START.md](QUICK_START.md) | [OPERATIONS.md](OPERATIONS.md) |
| Architecture | [ARCHITECTURE.md](ARCHITECTURE.md) | [DESIGN.md](DESIGN.md), [PAPER.md](PAPER.md) |
| Performance | [PERFORMANCE.md](PERFORMANCE.md) | [OPERATIONS.md](OPERATIONS.md) |
| Comparison | [COMPARISON.md](COMPARISON.md) | [FAQ.md](FAQ.md), [TIKV_COMPARISON.md](TIKV_COMPARISON.md) |
| TiKV Comparison | [TIKV_COMPARISON.md](TIKV_COMPARISON.md) | [ARCHITECTURE.md](ARCHITECTURE.md), [COMPARISON.md](COMPARISON.md) |
| Examples | [EXAMPLES.md](EXAMPLES.md) | [QUICK_START.md](QUICK_START.md) |
| Data Path | [DATA_PATH.md](DATA_PATH.md) | [ARCHITECTURE.md](ARCHITECTURE.md) |
| CAP Theorem | [CAP_THEOREM.md](CAP_THEOREM.md) | [DESIGN.md](DESIGN.md) |
| Advanced Features | [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) | [DESIGN.md](DESIGN.md) |
| Epochs | [EPOCHS.md](EPOCHS.md) | [ARCHITECTURE.md](ARCHITECTURE.md) |
| Dynamic Partitions | [DYNAMIC_PARTITIONS.md](DYNAMIC_PARTITIONS.md) | [ARCHITECTURE.md](ARCHITECTURE.md), [TIKV_COMPARISON.md](TIKV_COMPARISON.md) |
| Client Partitioning | [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) | [EXAMPLES.md](EXAMPLES.md), [DATA_PATH.md](DATA_PATH.md) |
| Rust Libraries | [RUST_LIBRARIES.md](RUST_LIBRARIES.md) | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Implementation Plan | [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md) | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Contributing | [CONTRIBUTING.md](CONTRIBUTING.md) | [CHANGELOG.md](CHANGELOG.md) |

## 📝 Documentation Quality

All documentation follows these principles:
- ✅ **Clear**: Easy to understand
- ✅ **Complete**: Covers all aspects
- ✅ **Accurate**: Technically correct
- ✅ **Current**: Up to date
- ✅ **Examples**: Code samples included
- ✅ **Organized**: Logical structure

## 🆘 Getting Help

If you can't find what you're looking for:

1. **Check FAQ**: [FAQ.md](FAQ.md) answers common questions
2. **Search docs**: Use text search across all markdown files
3. **Ask community**:
   - GitHub Discussions
   - Discord server
   - GitHub Issues (for bugs)
4. **Email**: dev@dlog.io

## 🔄 Keeping Documentation Updated

Documentation is maintained alongside code:
- Updated with each feature
- Reviewed in pull requests
- Versioned with releases
- Community contributions welcome

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to help improve documentation.

## 📚 External Resources

### Rust Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Tokio Documentation](https://tokio.rs/tokio/tutorial)

### Distributed Systems
- [Raft Paper](https://raft.github.io/raft.pdf)
- [Designing Data-Intensive Applications](https://dataintensive.net/)
- [LogDevice Paper](https://engineering.fb.com/2017/08/31/core-infra/logdevice-a-distributed-data-store-for-logs/)

### Related Projects
- [Redpanda](https://redpanda.com/)
- [Apache Kafka](https://kafka.apache.org/)
- [LogDevice](https://logdevice.io/)
- [TiKV](https://tikv.org/) - Distributed key-value store with multi-Raft
- [Databend](https://databend.rs/) - Cloud-native data warehouse with advanced analytics features

---

**Last Updated**: 2025-11-01

**Maintainers**: DLog Team

**License**: MIT OR Apache-2.0

*Found an issue with the documentation? Please [open an issue](https://github.com/dlog/dlog/issues) or submit a PR!*

