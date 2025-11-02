//! DLog Replication - Flexible quorum-based replication
//!
//! Inspired by LogDevice's flexible quorums for high availability
//! and low latency replication.

pub mod quorum;
pub mod replicator;
pub mod copyset;
pub mod sync;

pub use quorum::{QuorumConfig, QuorumSet};
pub use replicator::{ReplicationManager, ReplicationConfig};
pub use copyset::CopySet;

