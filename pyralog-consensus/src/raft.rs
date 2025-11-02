use bytes::Bytes;
use pyralog_core::{Result, PyralogError, LogOffset};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::election::{ElectionTimeoutConfig, heartbeat_interval};
use crate::log::RaftLog;
use crate::rpc::{AppendEntriesRequest, AppendEntriesResponse, VoteRequest, VoteResponse};
use crate::state::{LogEntry, NodeRole, NodeState};

#[derive(Debug, Clone)]
pub struct RaftConfig {
    pub node_id: u64,
    pub cluster_nodes: Vec<u64>,
    pub data_dir: PathBuf,
    pub election_timeout: ElectionTimeoutConfig,
}

pub struct RaftNode {
    config: RaftConfig,
    state: Arc<RwLock<NodeState>>,
    log: Arc<RaftLog>,
    last_heartbeat: Arc<RwLock<Instant>>,
    peers: HashMap<u64, PeerConnection>,
}

struct PeerConnection {
    node_id: u64,
    // In production, this would hold actual network connections
}

impl RaftNode {
    pub async fn new(config: RaftConfig) -> Result<Self> {
        let log_path = config.data_dir.join(format!("raft-{}.log", config.node_id));
        let log = Arc::new(RaftLog::open(log_path)?);

        let persistent_state = log.load_state()?;
        let mut state = NodeState::new(config.node_id);
        state.persistent = persistent_state;

        let mut peers = HashMap::new();
        for &peer_id in &config.cluster_nodes {
            if peer_id != config.node_id {
                peers.insert(peer_id, PeerConnection { node_id: peer_id });
            }
        }

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            log,
            last_heartbeat: Arc::new(RwLock::new(Instant::now())),
            peers,
        })
    }

    /// Start the Raft node
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Start election timer
        let node_clone = Arc::clone(&self);
        tokio::spawn(async move {
            node_clone.run_election_timer().await;
        });

        // Start heartbeat timer (if leader)
        let node_clone = Arc::clone(&self);
        tokio::spawn(async move {
            node_clone.run_heartbeat_timer().await;
        });

        Ok(())
    }

    /// Propose a value to be committed
    pub async fn propose(&self, value: Bytes) -> Result<LogOffset> {
        let mut state = self.state.write();

        if state.role != NodeRole::Leader {
            return Err(PyralogError::NotLeader(None));
        }

        let term = state.persistent.current_term;
        let index = state.last_log_index() + 1;
        let entry = LogEntry::new(term, index, value.to_vec());

        state.persistent.log.push(entry);
        self.log.save_state(&state.persistent)?;

        // In production, replicate to followers here
        
        Ok(LogOffset::new(index))
    }

    /// Handle AppendEntries RPC
    pub async fn handle_append_entries(
        &self,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse> {
        let mut state = self.state.write();

        // Update last heartbeat
        *self.last_heartbeat.write() = Instant::now();

        // Reply false if term < currentTerm
        if request.term < state.persistent.current_term {
            return Ok(AppendEntriesResponse {
                term: state.persistent.current_term,
                success: false,
                match_index: None,
            });
        }

        // If RPC request or response contains term T > currentTerm:
        // set currentTerm = T, convert to follower
        if request.term > state.persistent.current_term {
            state.become_follower(request.term);
        }

        // Reply false if log doesn't contain an entry at prevLogIndex
        // whose term matches prevLogTerm
        if request.prev_log_index > 0 {
            if request.prev_log_index as usize > state.persistent.log.len() {
                return Ok(AppendEntriesResponse {
                    term: state.persistent.current_term,
                    success: false,
                    match_index: None,
                });
            }

            let prev_entry = &state.persistent.log[request.prev_log_index as usize - 1];
            if prev_entry.term != request.prev_log_term {
                // Delete conflicting entry and all that follow it
                state.persistent.log.truncate(request.prev_log_index as usize);
                self.log.save_state(&state.persistent)?;
                
                return Ok(AppendEntriesResponse {
                    term: state.persistent.current_term,
                    success: false,
                    match_index: None,
                });
            }
        }

        // Append any new entries not already in the log
        for entry in request.entries {
            if entry.index as usize > state.persistent.log.len() {
                state.persistent.log.push(entry);
            } else {
                // If an existing entry conflicts with a new one, delete it and all that follow
                if state.persistent.log[entry.index as usize - 1].term != entry.term {
                    state.persistent.log.truncate(entry.index as usize - 1);
                    state.persistent.log.push(entry);
                }
            }
        }

        self.log.save_state(&state.persistent)?;

        // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index of last new entry)
        if request.leader_commit > state.volatile.commit_index {
            state.volatile.commit_index = request
                .leader_commit
                .min(state.last_log_index());
        }

        Ok(AppendEntriesResponse {
            term: state.persistent.current_term,
            success: true,
            match_index: Some(state.last_log_index()),
        })
    }

    /// Handle RequestVote RPC
    pub async fn handle_vote_request(&self, request: VoteRequest) -> Result<VoteResponse> {
        let mut state = self.state.write();

        // Reply false if term < currentTerm
        if request.term < state.persistent.current_term {
            return Ok(VoteResponse {
                term: state.persistent.current_term,
                vote_granted: false,
            });
        }

        // If RPC request contains term T > currentTerm:
        // set currentTerm = T, convert to follower
        if request.term > state.persistent.current_term {
            state.become_follower(request.term);
        }

        // Grant vote if:
        // 1. votedFor is null or candidateId
        // 2. candidate's log is at least as up-to-date as receiver's log
        let can_vote = state.persistent.voted_for.is_none()
            || state.persistent.voted_for == Some(request.candidate_id);

        let log_up_to_date = request.last_log_term > state.last_log_term()
            || (request.last_log_term == state.last_log_term()
                && request.last_log_index >= state.last_log_index());

        if can_vote && log_up_to_date {
            state.persistent.voted_for = Some(request.candidate_id);
            self.log.save_state(&state.persistent)?;
            *self.last_heartbeat.write() = Instant::now();

            Ok(VoteResponse {
                term: state.persistent.current_term,
                vote_granted: true,
            })
        } else {
            Ok(VoteResponse {
                term: state.persistent.current_term,
                vote_granted: false,
            })
        }
    }

    /// Check if this node is the leader
    pub fn is_leader(&self) -> bool {
        self.state.read().role == NodeRole::Leader
    }

    /// Get the current leader ID
    pub fn leader_id(&self) -> Option<u64> {
        let state = self.state.read();
        if state.role == NodeRole::Leader {
            Some(state.node_id)
        } else {
            None
        }
    }

    /// Get the committed offset
    pub fn committed_offset(&self) -> LogOffset {
        LogOffset::new(self.state.read().volatile.commit_index)
    }

    /// Run the election timer
    async fn run_election_timer(self: Arc<Self>) {
        loop {
            let timeout = self.config.election_timeout.generate_timeout();
            sleep(timeout).await;

            let last_heartbeat = *self.last_heartbeat.read();
            let elapsed = last_heartbeat.elapsed();

            let role = self.state.read().role;
            
            // Start election if we're a follower or candidate and haven't heard from leader
            if role != NodeRole::Leader && elapsed >= timeout {
                self.start_election().await;
            }
        }
    }

    /// Run the heartbeat timer (for leaders)
    async fn run_heartbeat_timer(self: Arc<Self>) {
        loop {
            sleep(heartbeat_interval()).await;

            if self.is_leader() {
                self.send_heartbeats().await;
            }
        }
    }

    /// Start a new election
    async fn start_election(&self) {
        let mut state = self.state.write();
        state.become_candidate();
        
        let term = state.persistent.current_term;
        let last_log_index = state.last_log_index();
        let last_log_term = state.last_log_term();
        let candidate_id = state.node_id;
        
        drop(state);

        self.log.save_state(&self.state.read().persistent).ok();

        // Vote for self
        let mut votes = 1;
        let majority = (self.config.cluster_nodes.len() / 2) + 1;

        // Request votes from all peers
        // In production, this would send actual RPC requests
        // For now, we'll simulate winning the election if we're the first node
        if candidate_id == self.config.cluster_nodes[0] {
            votes = majority;
        }

        if votes >= majority {
            let mut state = self.state.write();
            if state.role == NodeRole::Candidate && state.persistent.current_term == term {
                state.become_leader(self.config.cluster_nodes.len());
                self.log.save_state(&state.persistent).ok();
            }
        }
    }

    /// Send heartbeats to all followers
    async fn send_heartbeats(&self) {
        let state = self.state.read();
        
        if state.role != NodeRole::Leader {
            return;
        }

        let request = AppendEntriesRequest {
            term: state.persistent.current_term,
            leader_id: state.node_id,
            prev_log_index: state.last_log_index(),
            prev_log_term: state.last_log_term(),
            entries: Vec::new(), // Heartbeat has no entries
            leader_commit: state.volatile.commit_index,
        };

        // In production, send to all peers
        // For now, this is a placeholder
    }
}

