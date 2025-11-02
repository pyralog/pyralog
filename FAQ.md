# DLog Frequently Asked Questions

## General Questions

### What is DLog?

DLog is a high-performance distributed log system built in Rust, inspired by Redpanda and LogDevice. It provides low-latency, durable, and scalable log storage for distributed systems.

### Why another distributed log system?

DLog combines the best ideas from existing systems:
- **Redpanda**: Write caching, no ZooKeeper, Kafka compatibility
- **LogDevice**: Flexible quorums, epochs, CopySet replication
- **Rust**: Memory safety, modern concurrency, zero-cost abstractions

### Is DLog production-ready?

DLog is currently in **active development**. Core features are complete, but we recommend:
- ✅ Use for development and testing
- ✅ Use for non-critical production workloads
- ⚠️ Wait for v1.0 for mission-critical systems

### How does DLog compare to Kafka?

See our [detailed comparison guide](COMPARISON.md). In short:
- **DLog advantages**: Lower latency, simpler ops, no ZooKeeper
- **Kafka advantages**: More mature, larger ecosystem, enterprise support

## Architecture Questions

### What consensus protocol does DLog use?

DLog uses **Raft** for cluster coordination and metadata management. Raft was chosen for:
- Simplicity and understandability
- Proven in production (etcd, Consul)
- Strong consistency guarantees
- Fast leader election

### What are epochs and why do they matter?

Epochs are generation numbers that track which leader wrote records. They prevent:
- Split-brain scenarios during failover
- Duplicate writes during recovery
- Ambiguity about record ordering

See [EPOCHS.md](EPOCHS.md) for details.

### How does replication work?

DLog uses **flexible quorum-based replication** inspired by LogDevice:
- Configure: Replication Factor (R), Write Quorum (W), Read Quorum (R)
- Constraint: W + R > RF (ensures consistency)
- Examples: (R=3, W=2, R=2) for balanced, (R=3, W=1, R=3) for writes

### What is CopySet replication?

CopySet replication intelligently selects replica nodes to minimize data loss probability. Instead of random placement, it:
- Groups nodes into copysets
- Tracks copyset usage
- Balances across failure domains
- Reduces combinations that can cause data loss

## Performance Questions

### What latency can I expect?

On modern hardware (AWS c5.2xlarge):
- **p99 write latency**: < 1ms (with write cache)
- **p50 write latency**: ~0.3ms
- **p99 read latency**: < 0.3ms

See [PERFORMANCE.md](PERFORMANCE.md) for detailed benchmarks.

### How do I optimize for throughput?

For maximum throughput:
1. Enable large write cache (128MB+)
2. Use batching (1000+ records per batch)
3. Increase segment size (2-4GB)
4. Use async replication (W=1)
5. Add more partitions

### How do I optimize for latency?

For minimum latency:
1. Use small write cache (8MB)
2. Set short buffer timeout (1ms)
3. Enable memory-mapped I/O
4. Use NVMe SSDs
5. Async replication (W=1)

### What affects performance?

Key factors:
- **Hardware**: NVMe > SAS SSD > SATA SSD > HDD
- **Configuration**: Cache size, quorum settings
- **Workload**: Record size, batching, access patterns
- **Network**: Bandwidth and latency between nodes

## Operational Questions

### How many nodes do I need?

**Minimum**:
- 1 node: Development only
- 3 nodes: Minimum for HA (tolerates 1 failure)

**Recommended**:
- 3 nodes: Small deployments
- 5 nodes: Medium deployments (tolerates 2 failures)
- 7+ nodes: Large deployments

### How do I deploy DLog?

Multiple options:
1. **Bare metal**: Direct binary installation
2. **Docker**: Container deployment
3. **Kubernetes**: Operator-based (coming soon)
4. **Cloud**: AWS/GCP/Azure

See [OPERATIONS.md](OPERATIONS.md) for detailed guides.

### Do I need ZooKeeper?

**No!** DLog has built-in Raft consensus. This is a major operational simplification compared to Kafka.

### How do I monitor DLog?

Monitor key metrics:
- Write/read latency (p50, p99, p999)
- Throughput (ops/sec, bytes/sec)
- Replication lag
- ISR count
- Disk/CPU/memory usage

Integration with Prometheus/Grafana coming soon.

### How do I backup DLog?

Options:
1. **Filesystem backup**: Stop node, copy data directory
2. **Tiered storage**: Automatic backup to S3/Azure/GCS
3. **Replication**: Multiple replicas provide redundancy

See [OPERATIONS.md](OPERATIONS.md) for backup strategies.

## Compatibility Questions

### Is DLog Kafka-compatible?

**Yes**, DLog provides a Kafka-compatible API. Existing Kafka clients should work with minimal changes.

**Compatibility level**:
- ✅ Core produce/consume APIs
- ✅ Partitioning model
- ⚠️ Advanced features (transactions, streams) coming soon
- ⚠️ Some ecosystem tools may not work

### Can I migrate from Kafka?

Yes! Migration path:
1. Deploy DLog cluster
2. Dual-write to both systems
3. Migrate consumers
4. Decommission Kafka

See [COMPARISON.md](COMPARISON.md) for details.

### What client libraries are available?

Currently:
- ✅ Rust (native)
- 🔜 Python (planned)
- 🔜 Go (planned)
- 🔜 Java (planned)
- ✅ Any Kafka client (via compatibility layer)

## Development Questions

### What language is DLog written in?

**Rust** - chosen for:
- Memory safety without GC
- Zero-cost abstractions
- Excellent concurrency support
- Growing ecosystem
- Modern tooling

### How can I contribute?

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code of conduct
- Development setup
- Coding guidelines
- Submission process

### How do I build from source?

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/dlog/dlog.git
cd dlog
cargo build --release
```

### How do I run tests?

```bash
# All tests
cargo test

# Specific module
cargo test --package dlog-storage

# With output
cargo test -- --nocapture
```

## Configuration Questions

### What configuration options are available?

Key options:
- **Node**: ID, data directory, cluster members
- **Network**: Listen addresses, timeouts
- **Storage**: Segment size, caching, mmap
- **Replication**: Factor, quorums

See example config in [QUICK_START.md](QUICK_START.md).

### How do I tune for my workload?

Three profiles:
1. **Low latency**: Small cache, fast flush, async replication
2. **High throughput**: Large cache, batching, large segments
3. **High durability**: Sync writes, all replicas, no cache

See [PERFORMANCE.md](PERFORMANCE.md) for configurations.

### Can I change configuration without restart?

Some settings:
- ✅ Log retention policies
- ✅ Cache settings
- ❌ Node ID, cluster membership
- ❌ Data directory

## Feature Questions

### Does DLog support transactions?

**Not yet.** Transactions are planned for Phase 3. Currently available:
- Atomic batch writes
- Idempotent produces (via epochs)

### Does DLog support log compaction?

**Not yet.** Compaction is planned for Phase 3. Currently available:
- Time-based retention
- Size-based retention

### Does DLog support stream processing?

**Not yet.** Stream processing is planned for Phase 4. Currently available:
- Basic consume API
- Consumer groups (simple implementation)

### Does DLog support tiered storage?

**Yes!** DLog supports tiered storage to:
- S3 (AWS)
- Azure Blob Storage
- Google Cloud Storage
- Local filesystem

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

## Troubleshooting Questions

### Why is write latency high?

Common causes:
1. Disk saturation (use `iostat`)
2. Write cache disabled
3. Sync on every write
4. Network latency
5. Too many replicas

See [PERFORMANCE.md](PERFORMANCE.md) for solutions.

### Why is replication lagging?

Common causes:
1. Slow follower disk
2. Network bandwidth saturation
3. Overloaded follower CPU
4. Large record batches

### How do I recover from node failure?

DLog handles this automatically:
1. Cluster detects failure
2. New leader elected
3. Replicas catch up
4. Normal operation resumes

Typical recovery time: < 30 seconds

### Where are the logs?

Default locations:
- Application logs: `/var/log/dlog/dlog.log`
- Data: `/var/lib/dlog/`
- Configuration: `/etc/dlog/config.json`

## Security Questions

### Does DLog support authentication?

**Not yet.** Authentication is planned for Phase 2. Coming soon:
- mTLS for node-to-node
- Token-based for clients
- RBAC for access control

### Does DLog support encryption?

**Partial**. Currently:
- ✅ TLS for client connections (planned)
- ❌ Encryption at rest (planned)
- ❌ End-to-end encryption (planned)

### How do I secure my cluster?

Current best practices:
1. Run in private network
2. Use firewall rules
3. Restrict internal port (9093)
4. Use VPN for remote access
5. Regular security updates

## Licensing Questions

### What license is DLog under?

**Dual licensed**: MIT-0 (code) and CC0-1.0 (documentation)

- **Code** (Rust source): MIT-0 (MIT No Attribution) - the most permissive possible
- **Documentation** (Markdown, papers): CC0-1.0 (Public Domain) - no restrictions whatsoever

This provides maximum freedom and flexibility for both commercial and open-source use.

### Can I use DLog commercially?

**Yes!** Both MIT-0 and CC0-1.0 are maximally permissive licenses. You can use DLog in any commercial product without attribution requirements.

### Do I need to open-source my application?

**No!** Neither MIT-0 nor CC0-1.0 are copyleft licenses. You can use DLog in proprietary software without any obligation to share your code or even mention DLog.

## Community Questions

### How do I get help?

Multiple channels:
- **GitHub Issues**: Bug reports and features
- **GitHub Discussions**: Questions and ideas
- **Discord**: Real-time chat
- **Email**: dev@dlog.io

### How do I report a bug?

1. Check if already reported
2. Create GitHub issue
3. Include:
   - DLog version
   - Configuration
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs

### How do I request a feature?

1. Check if already requested
2. Create GitHub issue
3. Describe:
   - Use case
   - Proposed solution
   - Alternatives considered
   - Impact

### Where is the roadmap?

See [README.md](README.md#roadmap) for planned features and timeline.

## Glossary

- **Epoch**: Generation number for tracking leader changes
- **ISR**: In-Sync Replicas (up-to-date followers)
- **LSN**: Log Sequence Number (epoch + offset)
- **Quorum**: Number of nodes required for operation
- **Partition**: Independent, ordered log segment
- **Segment**: Physical file containing log records
- **Sequencer**: Leader that assigns offsets

---

**Don't see your question?** Ask on [GitHub Discussions](https://github.com/dlog/dlog/discussions) or [Discord](https://discord.gg/dlog).

