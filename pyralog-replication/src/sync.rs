use pyralog_core::{LogOffset, Result, PyralogError};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Notify;

/// Synchronization manager for tracking replication progress
pub struct SyncManager {
    /// Track the highest offset replicated to each node
    node_offsets: Arc<RwLock<HashMap<u64, LogOffset>>>,
    
    /// Notification system for offset updates
    notifiers: Arc<RwLock<HashMap<u64, Arc<Notify>>>>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            node_offsets: Arc::new(RwLock::new(HashMap::new())),
            notifiers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update the replicated offset for a node
    pub fn update_offset(&self, node_id: u64, offset: LogOffset) {
        let mut offsets = self.node_offsets.write();
        offsets.insert(node_id, offset);
        
        // Notify any waiters
        if let Some(notifier) = self.notifiers.read().get(&node_id) {
            notifier.notify_waiters();
        }
    }

    /// Get the current offset for a node
    pub fn get_offset(&self, node_id: u64) -> Option<LogOffset> {
        self.node_offsets.read().get(&node_id).copied()
    }

    /// Get the minimum offset across all nodes (committed offset)
    pub fn get_committed_offset(&self) -> LogOffset {
        self.node_offsets
            .read()
            .values()
            .min()
            .copied()
            .unwrap_or(LogOffset::ZERO)
    }

    /// Get the maximum offset across all nodes (high watermark)
    pub fn get_high_watermark(&self) -> LogOffset {
        self.node_offsets
            .read()
            .values()
            .max()
            .copied()
            .unwrap_or(LogOffset::ZERO)
    }

    /// Wait for a specific node to reach an offset
    pub async fn wait_for_offset(&self, node_id: u64, target_offset: LogOffset) -> Result<()> {
        loop {
            // Check current offset
            if let Some(current) = self.get_offset(node_id) {
                if current >= target_offset {
                    return Ok(());
                }
            }

            // Get or create notifier for this node
            let notifier = {
                let mut notifiers = self.notifiers.write();
                notifiers
                    .entry(node_id)
                    .or_insert_with(|| Arc::new(Notify::new()))
                    .clone()
            };

            // Wait for notification
            notifier.notified().await;
        }
    }

    /// Wait for a quorum of nodes to reach an offset
    pub async fn wait_for_quorum(
        &self,
        nodes: &[u64],
        target_offset: LogOffset,
        quorum_size: usize,
    ) -> Result<()> {
        loop {
            // Count how many nodes have reached the target
            let ready_count = nodes
                .iter()
                .filter(|&&node_id| {
                    self.get_offset(node_id)
                        .map(|offset| offset >= target_offset)
                        .unwrap_or(false)
                })
                .count();

            if ready_count >= quorum_size {
                return Ok(());
            }

            // Wait for any node to update
            let notifiers: Vec<_> = nodes
                .iter()
                .filter_map(|&node_id| {
                    let notifiers = self.notifiers.read();
                    notifiers.get(&node_id).cloned()
                })
                .collect();

            if notifiers.is_empty() {
                // No notifiers available, wait a bit and retry
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }

            // Wait for any notifier
            tokio::select! {
                _ = notifiers[0].notified() => {},
            }
        }
    }

    /// Get replication lag for a node
    pub fn get_lag(&self, node_id: u64) -> Option<u64> {
        let offsets = self.node_offsets.read();
        let node_offset = offsets.get(&node_id)?;
        let max_offset = offsets.values().max()?;
        
        Some(max_offset.as_u64().saturating_sub(node_offset.as_u64()))
    }

    /// Get nodes that are in sync (within lag threshold)
    pub fn get_in_sync_nodes(&self, max_lag: u64) -> Vec<u64> {
        let offsets = self.node_offsets.read();
        let max_offset = offsets.values().max().copied().unwrap_or(LogOffset::ZERO);

        offsets
            .iter()
            .filter(|(_, &offset)| {
                max_offset.as_u64().saturating_sub(offset.as_u64()) <= max_lag
            })
            .map(|(&node_id, _)| node_id)
            .collect()
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager() {
        let manager = SyncManager::new();

        manager.update_offset(1, LogOffset::new(100));
        manager.update_offset(2, LogOffset::new(50));
        manager.update_offset(3, LogOffset::new(75));

        assert_eq!(manager.get_committed_offset(), LogOffset::new(50));
        assert_eq!(manager.get_high_watermark(), LogOffset::new(100));
        
        assert_eq!(manager.get_lag(2), Some(50));
    }
}

