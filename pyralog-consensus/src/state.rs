use pyralog_core::LogOffset;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

/// Persistent state on all nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// Latest term server has seen
    pub current_term: u64,
    
    /// Candidate ID that received vote in current term
    pub voted_for: Option<u64>,
    
    /// Log entries
    pub log: Vec<LogEntry>,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
        }
    }
}

/// Volatile state on all nodes
#[derive(Debug, Clone)]
pub struct VolatileState {
    /// Index of highest log entry known to be committed
    pub commit_index: u64,
    
    /// Index of highest log entry applied to state machine
    pub last_applied: u64,
}

impl Default for VolatileState {
    fn default() -> Self {
        Self {
            commit_index: 0,
            last_applied: 0,
        }
    }
}

/// Volatile state on leaders
#[derive(Debug, Clone)]
pub struct LeaderState {
    /// For each server, index of next log entry to send
    pub next_index: Vec<u64>,
    
    /// For each server, index of highest log entry known to be replicated
    pub match_index: Vec<u64>,
}

impl LeaderState {
    pub fn new(cluster_size: usize, last_log_index: u64) -> Self {
        Self {
            next_index: vec![last_log_index + 1; cluster_size],
            match_index: vec![0; cluster_size],
        }
    }
}

/// Complete node state
pub struct NodeState {
    pub node_id: u64,
    pub role: NodeRole,
    pub persistent: PersistentState,
    pub volatile: VolatileState,
    pub leader: Option<LeaderState>,
}

impl NodeState {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            role: NodeRole::Follower,
            persistent: PersistentState::default(),
            volatile: VolatileState::default(),
            leader: None,
        }
    }

    pub fn become_follower(&mut self, term: u64) {
        self.role = NodeRole::Follower;
        self.persistent.current_term = term;
        self.persistent.voted_for = None;
        self.leader = None;
    }

    pub fn become_candidate(&mut self) {
        self.role = NodeRole::Candidate;
        self.persistent.current_term += 1;
        self.persistent.voted_for = Some(self.node_id);
        self.leader = None;
    }

    pub fn become_leader(&mut self, cluster_size: usize) {
        self.role = NodeRole::Leader;
        let last_log_index = self.persistent.log.len() as u64;
        self.leader = Some(LeaderState::new(cluster_size, last_log_index));
    }

    pub fn last_log_index(&self) -> u64 {
        self.persistent.log.len().saturating_sub(1) as u64
    }

    pub fn last_log_term(&self) -> u64 {
        self.persistent
            .log
            .last()
            .map(|entry| entry.term)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: u64,
    pub index: u64,
    pub data: Vec<u8>,
}

impl LogEntry {
    pub fn new(term: u64, index: u64, data: Vec<u8>) -> Self {
        Self { term, index, data }
    }
}

