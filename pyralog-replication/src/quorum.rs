use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Configuration for flexible quorums
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumConfig {
    /// Replication factor (total number of copies)
    pub replication_factor: usize,
    
    /// Number of acknowledgments required for write
    pub write_quorum: usize,
    
    /// Number of nodes that must have the data for read
    pub read_quorum: usize,
    
    /// Strategy for selecting replicas
    pub selection_strategy: ReplicaSelectionStrategy,
}

impl QuorumConfig {
    /// Create a simple majority quorum
    pub fn majority(replication_factor: usize) -> Self {
        let quorum_size = (replication_factor / 2) + 1;
        Self {
            replication_factor,
            write_quorum: quorum_size,
            read_quorum: quorum_size,
            selection_strategy: ReplicaSelectionStrategy::RoundRobin,
        }
    }

    /// Create a configuration optimized for writes
    pub fn write_optimized(replication_factor: usize) -> Self {
        Self {
            replication_factor,
            write_quorum: 1, // Acknowledge on first write
            read_quorum: replication_factor, // Must read from all replicas
            selection_strategy: ReplicaSelectionStrategy::Nearest,
        }
    }

    /// Create a configuration optimized for reads
    pub fn read_optimized(replication_factor: usize) -> Self {
        Self {
            replication_factor,
            write_quorum: replication_factor, // Must write to all replicas
            read_quorum: 1, // Can read from any replica
            selection_strategy: ReplicaSelectionStrategy::Nearest,
        }
    }

    /// Validate the quorum configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.replication_factor == 0 {
            return Err("Replication factor must be at least 1".to_string());
        }

        if self.write_quorum == 0 || self.write_quorum > self.replication_factor {
            return Err("Invalid write quorum size".to_string());
        }

        if self.read_quorum == 0 || self.read_quorum > self.replication_factor {
            return Err("Invalid read quorum size".to_string());
        }

        // Ensure read and write quorums overlap (for consistency)
        if self.write_quorum + self.read_quorum <= self.replication_factor {
            return Err("Read and write quorums must overlap for consistency".to_string());
        }

        Ok(())
    }
}

impl Default for QuorumConfig {
    fn default() -> Self {
        Self::majority(3)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicaSelectionStrategy {
    /// Select replicas in round-robin fashion
    RoundRobin,
    
    /// Select nearest replicas based on latency
    Nearest,
    
    /// Select replicas randomly
    Random,
    
    /// Select replicas in a specific datacenter first
    DatacenterAware { preferred_dc: String },
}

/// Represents a quorum set for a specific operation
#[derive(Debug, Clone)]
pub struct QuorumSet {
    /// All nodes in the replica set
    pub all_nodes: Vec<u64>,
    
    /// Nodes required for this specific operation
    pub required_nodes: HashSet<u64>,
    
    /// Number of successful responses received
    pub responses: usize,
    
    /// Number of responses needed
    pub target: usize,
}

impl QuorumSet {
    pub fn new(all_nodes: Vec<u64>, target: usize) -> Self {
        Self {
            all_nodes,
            required_nodes: HashSet::new(),
            responses: 0,
            target,
        }
    }

    /// Record a successful response from a node
    pub fn add_response(&mut self, node_id: u64) -> bool {
        if self.all_nodes.contains(&node_id) {
            self.required_nodes.insert(node_id);
            self.responses += 1;
            return true;
        }
        false
    }

    /// Check if quorum has been reached
    pub fn is_satisfied(&self) -> bool {
        self.responses >= self.target
    }

    /// Get the remaining nodes that haven't responded
    pub fn remaining_nodes(&self) -> Vec<u64> {
        self.all_nodes
            .iter()
            .filter(|&node| !self.required_nodes.contains(node))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_config_validation() {
        let config = QuorumConfig::majority(3);
        assert!(config.validate().is_ok());

        let invalid = QuorumConfig {
            replication_factor: 3,
            write_quorum: 1,
            read_quorum: 1,
            selection_strategy: ReplicaSelectionStrategy::RoundRobin,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_quorum_set() {
        let mut quorum = QuorumSet::new(vec![1, 2, 3], 2);
        assert!(!quorum.is_satisfied());

        quorum.add_response(1);
        assert!(!quorum.is_satisfied());

        quorum.add_response(2);
        assert!(quorum.is_satisfied());
    }
}

