//! DLog - High-Performance Distributed Log System
//!
//! DLog is a modern distributed log system built in Rust, inspired by
//! Redpanda and LogDevice. It provides:
//!
//! - High-performance, low-latency writes with write caching
//! - Flexible quorum-based replication
//! - Raft consensus for cluster coordination
//! - Kafka-compatible API
//! - Tiered storage support
//! - Zero-copy operations
//! - Memory-mapped I/O

pub mod server;
pub mod client;
pub mod cluster;
pub mod config;

pub use pyralog_core as core;
pub use pyralog_storage as storage;
pub use pyralog_consensus as consensus;
pub use pyralog_replication as replication;
pub use pyralog_protocol as protocol;

pub use server::DLogServer;
pub use client::DLogClient;
pub use config::DLogConfig;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::core::{
        LogId, LogOffset, PartitionId, Record, RecordBatch, RecordHeader, Result,
    };
    pub use crate::protocol::{
        ProduceRequest, ConsumeRequest, ProduceResponse, ConsumeResponse,
    };
    pub use crate::client::DLogClient;
    pub use crate::server::DLogServer;
}

