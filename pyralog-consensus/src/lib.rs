//! DLog Consensus - Raft-based consensus protocol
//!
//! This module implements a robust Raft consensus algorithm for
//! distributed log coordination and metadata management.

pub mod raft;
pub mod state;
pub mod log;
pub mod rpc;
pub mod election;

pub use raft::{RaftNode, RaftConfig};
pub use state::{NodeState, NodeRole};
pub use rpc::{AppendEntriesRequest, AppendEntriesResponse, VoteRequest, VoteResponse};

