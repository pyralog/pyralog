# DLog Documentation Index

**Complete guide to DLog: A platform for secure, parallel, distributed, and decentralized computing.**

DLog unifies cryptographic verification, actor-based concurrency, functional programming, multi-model databases, and decentralized consensus into a single coherent system.

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
  - [2. Obelisk Sequencer](blog/2-obelisk-sequencer.md) - Novel persistent atomic primitive
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
- **[MEMORY_ONLY_MODE](MEMORY_ONLY_MODE.md)** ⭐ **NEW: Ultra-fast ephemeral storage (10-100× faster)**
- **[COMPARISON](COMPARISON.md)** - How DLog compares to alternatives
- **[TIKV_COMPARISON](TIKV_COMPARISON.md)** - Detailed comparison with TiKV

### Advanced Computing Primitives
- **[ACTOR_MODEL](ACTOR_MODEL.md)** ⭐ **NEW: Location-transparent actors, topology-level reactivity, supervision trees**
- **[TENSOR_DATABASE](TENSOR_DATABASE.md)** ⭐ **NEW: Multi-dimensional arrays, ML/AI, vectors, embeddings**
- **[DADBS](DADBS.md)** ⭐ **NEW: Decentralized Autonomous Database Systems**
- **[MPHF](MPHF.md)** ⭐ **NEW: Partitioned Perfect Hash Maps (O(1) lookups, zero collisions)**
- **[WIREGUARD_PROTOCOL](WIREGUARD_PROTOCOL.md)** ⭐ **NEW: Quantum-resistant universal protocol (WireGuard + Rosenpass)**
- **[BATUTA](BATUTA.md)** ⭐ **NEW: DLog's programming language (Clojure + Elixir fusion, actor-first queries)**

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
  - Novel coordination primitives (Obelisk Sequencer)
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
  - **Obelisk Sequencer pattern** (novel primitive) ⭐
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
      - How Obelisk Sequencer prevents duplicate IDs
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

#### Decentralized Systems
- [DADBS.md](DADBS.md) ⭐ **NEW: Decentralized Autonomous Database Systems**
  - **Self-managing, trustless, distributed database infrastructure**
    - Decentralized (no single point of control)
    - Autonomous (self-healing, self-optimizing, self-configuring, self-protecting)
    - Trustless (cryptographic verification, Byzantine fault tolerance)
    - Economic incentives align node behavior
  - **5 Consensus Mechanisms**
    - Raft (crash fault tolerant, fast, DLog default)
    - PBFT (Practical Byzantine Fault Tolerant, malicious nodes)
    - Tendermint (BFT with fast finality, instant finality)
    - Proof of Work (Nakamoto consensus, permissionless, energy-intensive)
    - Proof of Stake (economic security, no energy waste)
    - Complete comparison table and implementation patterns
  - **Complete Architecture**
    - Layered architecture (Application → Autonomy → Consensus → Storage → Network → Crypto)
    - Node architecture with identity, storage, consensus, autonomy, networking, incentives
    - Hybrid network topology (structured + DHT + gossip)
  - **Smart Contracts for Databases**
    - Programmable constraints, triggers, policies
    - Access control, data retention, escrow, voting
  - **6 Major Use Cases**
    - Decentralized social networks, supply chain tracking, healthcare records
    - Financial settlement, voting systems, IoT data marketplace
  - **Implementation Patterns**
    - Hybrid architecture, optimistic execution, sharded DADBS
  - **Complete governance model with on-chain voting**

#### Functional Relational Algebra
- [FUNCTIONAL_RELATIONAL_ALGEBRA.md](FUNCTIONAL_RELATIONAL_ALGEBRA.md) ⭐ **NEW: Pure functional query system**
  - **Pure Function Relational Operators**
    - Select (σ), Project (π), Join (⋈), Union (∪), Difference (−)
    - Immutable operations, no side effects
    - Composable via method chaining
  - **Monad-Based Query DSL**
    - Query as monad with `flatMap`/`>>=`
    - Do-notation macro for Rust
    - Composable queries
    - Monad laws verified
  - **Applicative Functor Queries**
    - Parallel query execution
    - Independent sub-queries
    - 2-3× speedup for independent operations
  - **Lazy Evaluation**
    - Build queries without executing
    - Optimize before execution
    - 2.25× faster with fusion
    - DataFusion integration
  - **Algebraic Data Types (ADTs)**
    - QueryExpr ADT with pattern matching
    - Type-safe transformations
    - Query serialization
  - **Point-Free Style (Tacit Programming)**
    - Function composition without parameters
    - Query combinators library
    - Operator overloading (`>>` for compose)
  - **Type-Level Query Safety**
    - Compile-time schema validation
    - Typed columns and joins
    - HList for heterogeneous rows
    - Prevent column mismatches at compile time
  - **Functional Query Rewrite Rules**
    - Algebraic laws for optimization
    - 14× speedup with filter pushdown
    - Cost-based optimization
    - Provably correct transformations
  - **Complete implementation roadmap** (11-16 months)

#### Multi-Model Database & Category Theory
- [MULTI_MODEL_DATABASE.md](MULTI_MODEL_DATABASE.md) ⭐ **NEW: MultiCategory-inspired features**
  - **Multi-Model Data Support** (5 data models in single backend)
    - Relational (SQL tables)
    - Document (JSON/XML hierarchies)
    - Property Graph (Cypher queries, nodes + edges)
    - Key-Value (dictionary storage)
    - RDF Graph (SPARQL queries, semantic web)
    - Unified storage in Apache Arrow
  - **Category Theory Foundation**
    - Schema as category (objects + morphisms)
    - Instance as functor (Set → Set)
    - Mathematical rigor for correctness
    - Composable transformations
  - **Fold-Function Based Queries**
    - foldLeft, foldRight, reduce, scan primitives
    - Functional programming paradigm
    - Composable operations
    - Parallel folds with Rayon
    - Integration with DataFusion
  - **Multi-Model Joins**
    - Join relational with graph data
    - Join documents with tables
    - Graph-to-graph joins
    - RDF-to-relational joins
    - Category-theoretic pullback semantics
    - 10-50× faster than ETL approach
  - **Schema Categories**
    - Type-safe schema evolution
    - Migration as functors
    - Functor laws verification
    - Reversible migrations
  - **Native Graph Data Model**
    - Property graphs (Cypher queries)
    - RDF graphs (SPARQL queries)
    - Graph algorithms (PageRank, shortest path, communities)
    - 10-50× faster than Neo4j for analytics
  - **Hierarchical Data Model**
    - JSONPath queries
    - XPath queries
    - Tree fold operations
    - Nested Arrow structures
  - **ML-Driven Query Optimization**
    - Cost prediction with decision trees
    - Adaptive query execution
    - Workload analysis
    - Index suggestions
    - Reinforcement learning optimizer
  - **Complete implementation roadmap** (15-20 months)
  - **Inspiration**: [MultiCategory Project](https://multicategory.github.io/)

#### Cryptographic Verification & Zero-Trust
- [CRYPTOGRAPHIC_VERIFICATION.md](CRYPTOGRAPHIC_VERIFICATION.md) ⭐ **NEW: immudb-inspired features**
  - **Merkle Trees** (segment-level + partition-level)
    - Tamper-proof verification with BLAKE3 cryptographic hashing
    - BLAKE3: 10× faster than SHA256 (3 GB/s vs 300 MB/s)
    - 33× faster on multi-core, SIMD optimized
    - Efficient inclusion proofs (O(log N) size)
    - Root hashes stored in Raft
  - **Zero-Trust Architecture**
    - Client-side verification (don't trust server)
    - State signatures with HSM support
    - Byzantine fault tolerance
  - **Notarization API**
    - Timestamp external data (files, events, documents)
    - Cryptographic receipts
    - Copyright protection, legal timestamps
  - **Auditor Mode**
    - Independent read-only verification nodes
    - Continuous tamper detection
    - Regulatory compliance (SEC, HIPAA, SOC2)
  - **Cryptographic Receipts**
    - Non-repudiation (prove "I wrote X at time T")
    - Merkle proofs + state signatures
    - Financial transactions, supply chain tracking
  - **Multi-Signature Transactions**
    - M-of-N approval workflows
    - Compliance workflows (require legal + finance)
    - Treasury systems, configuration changes
  - **HSM Integration**
    - Hardware key protection (FIPS 140-2 Level 3)
    - PKCS#11 support
    - YubiHSM, AWS CloudHSM, Azure Key Vault
  - **Blockchain-Style Chaining**
    - Each record includes prev_hash
    - Dual verification (Merkle + chain)
    - Sequential integrity
  - **Complete implementation roadmap** (9-12 months)
  - **Performance**: 490M writes/sec with BLAKE3 (4,900× faster than immudb)
  - **BLAKE3 advantage**: +36M writes/sec vs SHA256, 10× faster hashing
  - **Use cases**: Finance, healthcare, government, supply chain, IoT

#### Tensor Database & ML/AI
- [TENSOR_DATABASE.md](TENSOR_DATABASE.md) ⭐ **NEW: Multi-dimensional arrays for ML/AI workloads**
  - **Native Tensor Storage & Operations**
    - Multi-dimensional arrays (tensors) as first-class citizens
    - Tensor algebra, decomposition, category theory foundations
    - Zero-copy tensor exchange via DLPack (PyTorch, TensorFlow, JAX, ONNX, Hugging Face)
    - Arrow storage format with chunking, compression, mmap, Flight protocol
  - **ML Framework Integration**
    - DLPack for seamless framework interop
    - Distributed training support (data, model, pipeline, 3D parallelism)
    - Checkpointing, fault tolerance, gradient synchronization
  - **GPU Memory Management**
    - Unified memory, pinned memory, multi-GPU coordination
    - Memory pooling, CUDA graphs, defragmentation, monitoring
  - **Zarr Format Support**
    - Cloud-native chunked N-dimensional arrays
    - v2/v3 support, S3/GCS optimization, compression codecs
    - Import/export, metadata, comparison with HDF5/NetCDF
  - **Polystore Tensor Data Model**
    - Mathematical theory (tensor algebra, category theory, formal transformations)
    - Query semantics, optimization theory, complexity analysis
    - 5 detailed cross-model SQL queries with execution plans
  - **Complete ML/AI feature set**
    - Vector embeddings, ANN search, ML feature store, model registry
    - Time-series tensors, image/video storage, probabilistic tensors, graph embeddings

#### Data Structures & Algorithms
- [MPHF.md](MPHF.md) ⭐ **NEW: Partitioned Perfect Hash Maps**
  - **Merging Multiple Perfect Hash Maps**
    - Deterministic, streaming, parallelizable algorithm
    - O(1) guaranteed lookup, zero collisions
    - Complete build pipeline (sample → partition → reduce → build)
    - 6 deduplication strategies (LWW, First-Wins, Max-Value, Priority, Timestamp, Custom)
    - PHF builders comparison (BBHash, RecSplit, PTHash, CHD, BDZ)
    - Complete file format specification with memory-mapped loading
    - Real benchmarks: 18s build for 100M keys, 45ns p50 lookup, 15.8× parallelism speedup
    - ~200 lines of Rust implementation with fluent API
    - 5 real-world use cases (LSM compaction, DHT, routing, genomics, config management)
    - Advanced optimizations (SIMD, zero-copy, batch prefetch, Bloom filters)

#### Networking & Security
- [WIREGUARD_PROTOCOL.md](WIREGUARD_PROTOCOL.md) ⭐ **NEW: WireGuard as Universal Protocol**
  - **Why WireGuard Over TLS**
    - 10× less handshake complexity (1-RTT vs 2-RTT)
    - 4,000 lines of code vs 100,000+ (OpenSSL/BoringSSL)
    - Cryptokey routing (no IP-based trust)
    - No cipher negotiation (sensible defaults)
    - NAT traversal built-in
  - **Architecture & Communication Paths**
    - Client→Cluster (user applications)
    - Node→Node (internal cluster, Raft, replication)
    - Cluster→Cluster (multi-datacenter replication)
    - Admin→Cluster (secure administration)
    - Universal protocol for all DLog communication
  - **Security Model**
    - Curve25519 (ECDH), ChaCha20 (encryption), Poly1305 (authentication), BLAKE2s (hashing)
    - Zero-trust architecture with peer isolation
    - Automatic key rotation every 2 minutes
    - Perfect forward secrecy
  - **DPI (Deep Packet Inspection) Resistance**
    - Silent protocol (no handshake pattern)
    - Cryptographic camouflage
    - Random padding, traffic shaping, port hopping
    - Decoy traffic generation
    - obfs4-style obfuscation
    - Success rates: 95% GFW bypass, 99% corporate networks
  - **Quantum Resistance** ⭐
    - Current vulnerabilities (Curve25519: vulnerable to Shor's algorithm)
    - Rosenpass integration (post-quantum key exchange)
    - Kyber1024 (NIST PQC standard) for key encapsulation
    - Dilithium for digital signatures
    - Hybrid cryptography (classical + post-quantum)
    - Security guarantee: attacker must break BOTH classical AND PQ
    - Minimal overhead: ~0.1ms handshake latency
    - Migration strategy (hybrid → PQ-preferred → PQ-only)
    - NIST PQC compliant, CNSA 2.0 requirements
    - Timeline: quantum-resistant by 2030
  - **Performance Characteristics**
    - 9.5 Gbps throughput (kernel module)
    - 6-8 Gbps throughput (BoringTun userspace)
    - 1-RTT handshake (~0.2ms)
    - Sub-millisecond latency overhead
  - **BoringTun: Userspace WireGuard in Rust**
    - Cloudflare's implementation (~8,000 lines Rust)
    - Cross-platform (Linux, macOS, Windows, BSD)
    - No kernel module required
    - Memory-safe Rust implementation
    - Container-friendly (no NET_ADMIN capability)
    - Hybrid deployment strategy
  - **Cross-Platform Support**
    - Linux kernel module (production), FreeBSD kernel, OpenBSD kernel
    - Windows kernel (wireguard-nt), macOS (wireguard-go)
    - BoringTun userspace (all platforms)
    - Platform comparison table with performance metrics
  - **Configuration & Deployment**
    - Automatic bootstrap, TOML configuration
    - Kubernetes, Docker Compose examples
    - Key management (generation, distribution, rotation)
    - Health checks and connection tracking
  - **Standards Compliance**
    - NIST Post-Quantum Cryptography (Kyber, Dilithium, SPHINCS+)
    - CNSA 2.0 (quantum-resistant by 2030)
    - Future: Native WireGuard v2 with PQ support (2025-2027)
  - **Complete implementation roadmap with Rust code examples**
  - **~2,780 lines of comprehensive documentation**

#### Programming Language
- [BATUTA.md](BATUTA.md) ⭐ **NEW: DLog's Dynamic Programming Language**
  - **Clojure + Elixir Fusion**
    - Lisp S-expressions with full macro system
    - Elixir-style pattern matching and pipe operators
    - Best of both worlds: code-as-data + modern syntax
  - **Actor-First Architecture**
    - Queries execute as distributed actors
    - Automatic parallelism across cluster
    - Location-transparent remote actors
    - Supervision trees for fault tolerance
  - **Immutable Data Structures**
    - Persistent vectors, maps, sets (O(log N) updates)
    - Structural sharing (no deep copying)
    - Clojure-style data structures in Rust
  - **Pattern Matching**
    - Destructure lists, maps, nested structures
    - Match actor messages with guards
    - Elegant control flow
  - **Lisp Macros**
    - Full macro system for DSLs
    - Query language implemented as macros
    - Syntax extension at compile time
  - **Fault Tolerance**
    - "Let it crash" philosophy
    - Supervision trees (one-for-one, one-for-all, rest-for-one)
    - Links and monitors
    - Self-healing systems
  - **Distributed Execution**
    - Flocks (auto-discovery via mDNS/gossip)
    - Deploy-* operators (deploy-map, deploy-reduce)
    - Remote actors on any cluster node
    - Distributed queries
  - **Gradual Typing**
    - Optional type annotations
    - Type inference
    - Spec-based validation
    - Actor protocols (typed messages)
  - **Performance**
    - Compiles to Rust (native code)
    - 2-3× faster than Clojure
    - 1.5× faster than Elixir
    - 50ms startup time (vs 2s for Clojure)
  - **REPL & Interactive Development**
    - Live data exploration
    - Hot code reloading
    - Actor introspection
    - Time-travel debugging
  - **DLog Native Integration**
    - Snowflake IDs, Obelisk Sequencers
    - Merkle tree verification
    - Multi-model queries (SQL, Cypher, SPARQL, JSONPath)
    - Tensor operations, cryptographic primitives
  - **Complete Language Specification**
    - 20 major sections
    - Syntax, data types, functions, actors, queries
    - Pattern matching, macros, fault tolerance
    - 6 detailed examples
    - 13-18 month roadmap to production
  - **~1,000 lines of comprehensive specification**

#### Advanced Features
- [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) ⭐ **Updated with Percolator protocol**
  - **DLog's Architectural Advantages** (new section)
    - Obelisk Sequencer primitive
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
  - Schema registry (Obelisk Sequencer for schema IDs)
  - Consumer groups (Obelisk Sequencer for generations)
  - Connectors framework
  - Change data capture (Obelisk Sequencer for event IDs)
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
3. Review [CLIENT_PARTITIONING_PATTERNS.md](CLIENT_PARTITIONING_PATTERNS.md) for Obelisk Sequencer details
4. Check [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) for distributed coordinators pattern
5. Read [TIKV_COMPARISON.md](TIKV_COMPARISON.md) and [COMPARISON.md](COMPARISON.md) for performance comparisons

### "I want to build an immutable knowledge database"
1. Read [IMMUTABLE_KNOWLEDGE_DB.md](IMMUTABLE_KNOWLEDGE_DB.md) ⭐ - complete guide
2. Study [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) for transactions and time-travel
3. Review [EXAMPLES.md](EXAMPLES.md) for implementation patterns
4. Check [ARCHITECTURE.md](ARCHITECTURE.md) for MVCC and storage details

### "I need cryptographic verification and tamper-proof logs"
1. Read [CRYPTOGRAPHIC_VERIFICATION.md](CRYPTOGRAPHIC_VERIFICATION.md) ⭐ - complete guide
2. Study Merkle trees, zero-trust architecture, and HSM integration
3. Review [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) for transactions and exactly-once semantics
4. Check auditor mode for regulatory compliance
5. Implement multi-signature workflows for approval processes

### "I want a multi-model database with graph, document, and relational support"
1. Read [MULTI_MODEL_DATABASE.md](MULTI_MODEL_DATABASE.md) ⭐ - complete guide
2. Study category theory foundation for schema evolution
3. Review fold-function queries for functional programming style
4. Check multi-model joins for combining data models
5. Explore graph queries (Cypher, SPARQL) and hierarchical queries (JSONPath, XPath)
6. Learn about ML-driven query optimization

### "I want pure functional programming with type-safe queries"
1. Read [FUNCTIONAL_RELATIONAL_ALGEBRA.md](FUNCTIONAL_RELATIONAL_ALGEBRA.md) ⭐ - complete guide
2. Study pure function operators (select, project, join)
3. Review monad-based DSL for composable queries
4. Check applicative functors for parallel execution
5. Explore lazy evaluation and query optimization
6. Learn about type-level query safety and compile-time validation

### "I want to build decentralized autonomous systems"
1. Read [DADBS.md](DADBS.md) ⭐ - complete guide to Decentralized Autonomous Database Systems
2. Study 5 consensus mechanisms (Raft, PBFT, Tendermint, PoW, PoS)
3. Review autonomy layers (self-healing, self-optimizing, self-configuring, self-protecting)
4. Check smart contracts for databases (constraints, triggers, ACLs)
5. Explore economic incentives and token economics
6. Learn about governance models and on-chain voting

### "I need perfect hash maps for read-heavy workloads"
1. Read [MPHF.md](MPHF.md) ⭐ - complete algorithm specification
2. Study deterministic build pipeline (sample → partition → reduce → build)
3. Review 6 deduplication strategies and conflict resolution
4. Check PHF builders comparison (BBHash, RecSplit, PTHash, CHD, BDZ)
5. Explore file format specification and memory-mapped loading
6. Learn about SIMD optimizations and batch prefetching

### "I want to build ML/AI applications with tensor support"
1. Read [TENSOR_DATABASE.md](TENSOR_DATABASE.md) ⭐ - complete tensor database guide
2. Study DLPack integration for zero-copy framework interop (PyTorch, TensorFlow, JAX)
3. Review distributed training support (data, model, pipeline parallelism)
4. Check GPU memory management and CUDA optimization
5. Explore Zarr format for cloud-native arrays
6. Learn about polystore tensor model with mathematical foundations

### "I want fault-tolerant, location-transparent actor systems"
1. Read [ACTOR_MODEL.md](ACTOR_MODEL.md) ⭐ - complete actor model guide
2. Study theoretical foundations (formal semantics, session types, category theory)
3. Review actor-based query execution, partition management, stream processing
4. Check topology-level reactivity (flocks, deploy-* operators, peer discovery)
5. Explore supervision trees (let-it-crash, self-healing hierarchies)
6. Learn about capability-based security and typed actors
7. Review Stella-inspired actor-reactor unification

### "I need quantum-resistant networking with DPI evasion"
1. Read [WIREGUARD_PROTOCOL.md](WIREGUARD_PROTOCOL.md) ⭐ - complete WireGuard guide
2. Study quantum resistance with Rosenpass (Kyber1024 + Dilithium)
3. Review hybrid cryptography (classical + post-quantum)
4. Check DPI resistance features (obfuscation, traffic shaping, port hopping)
5. Explore BoringTun userspace implementation in Rust
6. Learn about cross-platform deployment (Linux, Windows, macOS, BSD)
7. Review NIST PQC compliance and CNSA 2.0 requirements

### "I want to query and process data with a modern language"
1. Read [BATUTA.md](BATUTA.md) ⭐ - complete Batuta language specification
2. Study Lisp S-expressions and macro system
3. Review Elixir-style pattern matching and pipe operators
4. Learn actor-first query execution (queries as distributed actors)
5. Explore immutable data structures (persistent vectors, maps, sets)
6. Check fault tolerance with supervision trees
7. Try REPL-driven development with live data exploration
8. Review DLog native integration (Snowflake IDs, Merkle trees, multi-model queries)

## 📊 Documentation Statistics

- **Total Documents**: 41 markdown files
  - 35 main documentation files
  - 6 blog posts
- **Total Lines**: ~74,000+ lines of documentation
- **Coverage**:
  - ✅ Architecture and design
  - ✅ Research contributions and academic paper
  - ✅ Actor model with theoretical foundations (formal semantics, session types, category theory)
  - ✅ Topology-level reactivity (flocks, deploy-* operators, Stella-inspired)
  - ✅ Functional relational algebra and pure functional programming
  - ✅ Immutable knowledge databases and temporal systems
  - ✅ Multi-model databases with category theory
  - ✅ Tensor database (ML/AI, vectors, embeddings, polystore)
  - ✅ Cryptographic verification and zero-trust architecture
  - ✅ Decentralized autonomous database systems (DADBS)
  - ✅ Partitioned perfect hash maps (MPHF)
  - ✅ WireGuard universal protocol (quantum resistance, DPI evasion)
  - ✅ Batuta programming language (Clojure + Elixir, actor-first queries)
  - ✅ Memory-only mode (ephemeral storage, caching)
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
| Actor Model | [ACTOR_MODEL.md](ACTOR_MODEL.md) | [ARCHITECTURE.md](ARCHITECTURE.md), [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) |
| DADBS | [DADBS.md](DADBS.md) | [ARCHITECTURE.md](ARCHITECTURE.md), [CRYPTOGRAPHIC_VERIFICATION.md](CRYPTOGRAPHIC_VERIFICATION.md) |
| Tensor Database | [TENSOR_DATABASE.md](TENSOR_DATABASE.md) | [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) |
| Perfect Hash Maps | [MPHF.md](MPHF.md) | [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) |
| WireGuard Protocol | [WIREGUARD_PROTOCOL.md](WIREGUARD_PROTOCOL.md) | [CRYPTOGRAPHIC_VERIFICATION.md](CRYPTOGRAPHIC_VERIFICATION.md), [OPERATIONS.md](OPERATIONS.md) |
| Batuta Language | [BATUTA.md](BATUTA.md) | [ACTOR_MODEL.md](ACTOR_MODEL.md), [ADVANCED_FEATURES.md](ADVANCED_FEATURES.md) |

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

**Last Updated**: 2025-11-02 (includes BATUTA.md - DLog's programming language)

**Maintainers**: DLog Team

**License**: MIT-0 (code) & CC0-1.0 (documentation)

*Found an issue with the documentation? Please [open an issue](https://github.com/dlog/dlog/issues) or submit a PR!*

