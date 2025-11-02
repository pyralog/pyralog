//! Pyralog Core - Fundamental abstractions for distributed logging
//!
//! This module provides the core types, traits, and interfaces for the Pyralog distributed log system.

pub mod error;
pub mod log;
pub mod offset;
pub mod epoch;
pub mod sequencer;
pub mod record;
pub mod partition;
pub mod traits;

pub use error::{PyralogError, Result};
pub use log::{LogId, LogMetadata};
pub use offset::{LogOffset, OffsetRange};
pub use epoch::{Epoch, EpochOffset, EpochMetadata, EpochStore};
pub use sequencer::Sequencer;
pub use record::{Record, RecordBatch, RecordHeader};
pub use partition::{Partition, PartitionId};

