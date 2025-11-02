use crate::epoch::{Epoch, EpochStore};
use crate::partition::PartitionId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Sequencer manages epoch numbers and record ordering for partitions
/// 
/// Inspired by LogDevice's sequencer concept. The sequencer is responsible for:
/// - Assigning epoch numbers to partitions
/// - Sealing old epochs during failover
/// - Ensuring no duplicate writes during recovery
#[derive(Clone)]
pub struct Sequencer {
    node_id: u64,
    partition_epochs: Arc<RwLock<HashMap<PartitionId, EpochStore>>>,
}

impl Sequencer {
    pub fn new(node_id: u64) -> Self {
        Self {
            node_id,
            partition_epochs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Activate sequencer for a partition (become leader)
    pub fn activate(&self, partition: PartitionId, start_offset: u64) -> Epoch {
        let mut epochs = self.partition_epochs.write();
        let store = epochs.entry(partition).or_insert_with(EpochStore::new);
        store.start_epoch(self.node_id, start_offset)
    }

    /// Get current epoch for a partition
    pub fn current_epoch(&self, partition: PartitionId) -> Option<Epoch> {
        self.partition_epochs
            .read()
            .get(&partition)
            .and_then(|store| store.current_epoch())
    }

    /// Seal an epoch (during failover or graceful shutdown)
    pub fn seal_epoch(&self, partition: PartitionId, epoch: Epoch, last_offset: u32) -> bool {
        self.partition_epochs
            .write()
            .get_mut(&partition)
            .map(|store| store.seal_epoch(epoch, last_offset))
            .unwrap_or(false)
    }

    /// Check if sequencer can write to this epoch
    pub fn can_write(&self, partition: PartitionId, epoch: Epoch) -> bool {
        self.partition_epochs
            .read()
            .get(&partition)
            .and_then(|store| store.get_epoch(epoch))
            .map(|metadata| metadata.can_write())
            .unwrap_or(false)
    }

    /// Get the epoch store for a partition
    pub fn get_epoch_store(&self, partition: PartitionId) -> Option<EpochStore> {
        self.partition_epochs
            .read()
            .get(&partition)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequencer_activation() {
        let sequencer = Sequencer::new(1);
        let partition = PartitionId::new(0);

        let epoch = sequencer.activate(partition, 0);
        assert_eq!(epoch, Epoch::FIRST);
        assert_eq!(sequencer.current_epoch(partition), Some(epoch));
    }

    #[test]
    fn test_epoch_sealing() {
        let sequencer = Sequencer::new(1);
        let partition = PartitionId::new(0);

        let epoch = sequencer.activate(partition, 0);
        assert!(sequencer.can_write(partition, epoch));

        sequencer.seal_epoch(partition, epoch, 999);
        assert!(!sequencer.can_write(partition, epoch));
    }

    #[test]
    fn test_failover() {
        let sequencer1 = Sequencer::new(1);
        let sequencer2 = Sequencer::new(2);
        let partition = PartitionId::new(0);

        // Node 1 activates
        let epoch1 = sequencer1.activate(partition, 0);
        assert_eq!(epoch1, Epoch::FIRST);

        // Node 1 seals its epoch
        sequencer1.seal_epoch(partition, epoch1, 999);

        // Node 2 activates with new epoch
        let epoch2 = sequencer2.activate(partition, 1000);
        assert_eq!(epoch2, Epoch::new(2));
        assert!(epoch2 > epoch1);
    }
}

