use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;

/// CopySet represents a set of nodes that store a copy of data
/// Inspired by LogDevice's copyset replication
#[derive(Debug, Clone)]
pub struct CopySet {
    pub nodes: Vec<u64>,
    pub leader: u64,
}

impl CopySet {
    pub fn new(nodes: Vec<u64>, leader: u64) -> Self {
        Self { nodes, leader }
    }

    pub fn contains(&self, node_id: u64) -> bool {
        self.nodes.contains(&node_id)
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }
}

/// CopySet selector for efficient replica placement
pub struct CopySetSelector {
    all_nodes: Vec<u64>,
    replication_factor: usize,
    /// Track copyset usage for load balancing
    copyset_usage: HashMap<Vec<u64>, usize>,
}

impl CopySetSelector {
    pub fn new(all_nodes: Vec<u64>, replication_factor: usize) -> Self {
        Self {
            all_nodes,
            replication_factor,
            copyset_usage: HashMap::new(),
        }
    }

    /// Select a copyset for storing data
    /// Uses copyset replication to reduce the probability of data loss
    pub fn select_copyset(&mut self) -> Option<CopySet> {
        if self.all_nodes.len() < self.replication_factor {
            return None;
        }

        let mut rng = rand::thread_rng();
        
        // Select nodes for the copyset
        let mut nodes = self.all_nodes.clone();
        nodes.shuffle(&mut rng);
        nodes.truncate(self.replication_factor);
        nodes.sort_unstable();

        // Track copyset usage
        *self.copyset_usage.entry(nodes.clone()).or_insert(0) += 1;

        // Select a leader from the copyset
        let leader = *nodes.choose(&mut rng)?;

        Some(CopySet { nodes, leader })
    }

    /// Select a copyset with datacenter awareness
    pub fn select_copyset_dc_aware(
        &mut self,
        datacenter_map: &HashMap<u64, String>,
        preferred_dc: &str,
    ) -> Option<CopySet> {
        let mut rng = rand::thread_rng();

        // First, try to get at least one node from preferred DC
        let preferred_nodes: Vec<u64> = self
            .all_nodes
            .iter()
            .filter(|&node| {
                datacenter_map
                    .get(node)
                    .map(|dc| dc == preferred_dc)
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        let other_nodes: Vec<u64> = self
            .all_nodes
            .iter()
            .filter(|&node| {
                datacenter_map
                    .get(node)
                    .map(|dc| dc != preferred_dc)
                    .unwrap_or(true)
            })
            .copied()
            .collect();

        let mut selected_nodes = Vec::new();

        // Add at least one node from preferred DC if available
        if !preferred_nodes.is_empty() {
            let node = *preferred_nodes.choose(&mut rng)?;
            selected_nodes.push(node);
        }

        // Fill remaining slots
        let mut remaining_nodes = preferred_nodes
            .iter()
            .chain(other_nodes.iter())
            .filter(|&node| !selected_nodes.contains(node))
            .copied()
            .collect::<Vec<_>>();

        remaining_nodes.shuffle(&mut rng);

        while selected_nodes.len() < self.replication_factor && !remaining_nodes.is_empty() {
            selected_nodes.push(remaining_nodes.remove(0));
        }

        if selected_nodes.len() < self.replication_factor {
            return None;
        }

        selected_nodes.sort_unstable();
        let leader = *selected_nodes.choose(&mut rng)?;

        Some(CopySet {
            nodes: selected_nodes,
            leader,
        })
    }

    /// Get copyset statistics for monitoring
    pub fn get_stats(&self) -> CopySetStats {
        let total_copysets = self.copyset_usage.len();
        let total_usage: usize = self.copyset_usage.values().sum();
        
        let max_usage = self.copyset_usage.values().max().copied().unwrap_or(0);
        let min_usage = self.copyset_usage.values().min().copied().unwrap_or(0);

        CopySetStats {
            total_copysets,
            total_usage,
            max_usage,
            min_usage,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CopySetStats {
    pub total_copysets: usize,
    pub total_usage: usize,
    pub max_usage: usize,
    pub min_usage: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copyset_selection() {
        let nodes = vec![1, 2, 3, 4, 5];
        let mut selector = CopySetSelector::new(nodes, 3);

        let copyset = selector.select_copyset().unwrap();
        assert_eq!(copyset.size(), 3);
        assert!(copyset.contains(copyset.leader));
    }
}

