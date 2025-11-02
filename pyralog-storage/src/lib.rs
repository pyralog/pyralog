//! DLog Storage - High-performance storage engine
//!
//! Features:
//! - Segment-based log storage
//! - Memory-mapped I/O for fast reads
//! - Write-ahead logging
//! - Zero-copy operations
//! - Compression support

pub mod segment;
pub mod index;
pub mod log_storage;
pub mod write_cache;
pub mod tiered;

pub use log_storage::LogStorage;
pub use segment::{Segment, SegmentConfig};
pub use write_cache::WriteCache;

