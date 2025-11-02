use serde::{Deserialize, Serialize};
use crate::state::LogEntry;

/// AppendEntries RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesRequest {
    /// Leader's term
    pub term: u64,
    
    /// So follower can redirect clients
    pub leader_id: u64,
    
    /// Index of log entry immediately preceding new ones
    pub prev_log_index: u64,
    
    /// Term of prev_log_index entry
    pub prev_log_term: u64,
    
    /// Log entries to store (empty for heartbeat)
    pub entries: Vec<LogEntry>,
    
    /// Leader's commit index
    pub leader_commit: u64,
}

/// AppendEntries RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendEntriesResponse {
    /// Current term, for leader to update itself
    pub term: u64,
    
    /// True if follower contained entry matching prev_log_index and prev_log_term
    pub success: bool,
    
    /// For optimization: the index of last matching entry
    pub match_index: Option<u64>,
}

/// RequestVote RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    /// Candidate's term
    pub term: u64,
    
    /// Candidate requesting vote
    pub candidate_id: u64,
    
    /// Index of candidate's last log entry
    pub last_log_index: u64,
    
    /// Term of candidate's last log entry
    pub last_log_term: u64,
}

/// RequestVote RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    /// Current term, for candidate to update itself
    pub term: u64,
    
    /// True means candidate received vote
    pub vote_granted: bool,
}

