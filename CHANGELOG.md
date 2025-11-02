# Changelog

All notable changes to Pyralog will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete distributed log implementation with Raft consensus
- High-performance storage engine with memory-mapped I/O
- Write caching for sub-millisecond latencies (Redpanda-inspired)
- Flexible quorum-based replication (LogDevice-inspired)
- Epoch system for safe failover (LogDevice-inspired)
- CopySet replication for optimal replica placement
- Kafka-compatible protocol layer
- Partitioning strategies (key-hash, round-robin, sticky, random)
- Tiered storage support (S3, Azure, GCS, local)
- Comprehensive documentation suite

### Features in Detail

#### Storage Engine (`pyralog-storage`)
- Segment-based log-structured storage
- Sparse indexes for O(log n) lookups
- Memory-mapped files for zero-copy reads
- Write caching with configurable size and timeout
- Compression support (gzip, snappy, lz4, zstd)
- Tiered storage for cold data archival

#### Consensus (`pyralog-consensus`)
- Raft consensus protocol implementation
- Leader election with randomized timeouts
- Log replication with AppendEntries RPC
- Persistent state management
- Fast failover (< 300ms target)

#### Replication (`pyralog-replication`)
- Flexible quorum configuration
- CopySet-based replica selection
- ISR (In-Sync Replica) tracking
- Replication lag monitoring
- Datacenter-aware placement

#### Protocol (`pyralog-protocol`)
- Kafka-compatible API
- Multiple partitioning strategies
- Request/response wire format
- Error code mapping
- Kafka API compatibility layer

#### Core (`pyralog-core`)
- Epoch tracking and management
- Sequencer for epoch coordination
- Record and batch types
- Offset management
- Partition abstractions

### Documentation
- README with project overview
- ARCHITECTURE deep dive
- DESIGN document with rationale
- EXAMPLES with code samples
- QUICK_START guide
- EPOCHS explanation
- CONTRIBUTING guidelines
- OPERATIONS guide
- PERFORMANCE tuning guide
- COMPARISON with other systems
- FAQ for common questions
- PROJECT_SUMMARY

## [0.1.0] - 2025-01-01 (Planned)

### Target for Initial Release
- [ ] Complete network protocol implementation
- [ ] Full Kafka wire protocol compatibility
- [ ] Metrics and monitoring integration
- [ ] Administration CLI tools
- [ ] Integration test suite
- [ ] Production deployment guide
- [ ] Performance benchmarks baseline

## Future Releases

### [0.2.0] - Phase 2 (Q1 2026)
- [ ] Authentication (mTLS, token-based)
- [ ] Encryption (at rest, in transit)
- [ ] RBAC for access control
- [ ] Monitoring dashboards
- [ ] Kubernetes operator
- [ ] Python client SDK
- [ ] Go client SDK

### [0.3.0] - Phase 3 (Q2 2026)
- [ ] Log compaction
- [ ] Transactions support
- [ ] Multi-datacenter replication
- [ ] Schema registry
- [ ] Stream processing framework

### [1.0.0] - Phase 4 (Q3 2026)
- [ ] Production-ready declaration
- [ ] Performance guarantees
- [ ] Stability guarantees
- [ ] API stability
- [ ] Full documentation
- [ ] Enterprise support options

## Version History Template

### [X.Y.Z] - YYYY-MM-DD

#### Added
- New features

#### Changed
- Changes in existing functionality

#### Deprecated
- Soon-to-be removed features

#### Removed
- Removed features

#### Fixed
- Bug fixes

#### Security
- Security fixes

---

## Versioning

We use [SemVer](http://semver.org/) for versioning:

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality
- **PATCH**: Backwards-compatible bug fixes

## Release Process

1. Update CHANGELOG.md
2. Update version in Cargo.toml
3. Create git tag: `git tag -a vX.Y.Z -m "Release X.Y.Z"`
4. Push tag: `git push origin vX.Y.Z`
5. GitHub Actions builds and publishes release
6. Announce on Discord and website

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how to contribute to Pyralog.

