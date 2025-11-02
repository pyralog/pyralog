//! DLog Protocol - Protocol abstraction layer
//!
//! Provides protocol implementations including Kafka-compatible API

pub mod kafka;
pub mod api;
pub mod partitioner;
pub mod request;
pub mod response;

pub use api::{ProtocolHandler, ProduceRequest, ConsumeRequest, ProduceResponse, ConsumeResponse};
pub use partitioner::{Partitioner, PartitionStrategy};

