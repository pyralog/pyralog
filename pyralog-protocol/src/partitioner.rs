use bytes::Bytes;
use pyralog_core::PartitionId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Strategy for partitioning records across partitions
#[derive(Debug, Clone, Copy)]
pub enum PartitionStrategy {
    /// Round-robin distribution
    RoundRobin,
    
    /// Hash-based partitioning using key
    KeyHash,
    
    /// Random partition selection
    Random,
    
    /// Sticky partitioning (batch records to same partition)
    Sticky,
}

/// Partitioner for distributing records across partitions
pub struct Partitioner {
    strategy: PartitionStrategy,
    partition_count: u32,
    round_robin_counter: std::sync::atomic::AtomicU32,
    sticky_partition: std::sync::atomic::AtomicU32,
}

impl Partitioner {
    pub fn new(strategy: PartitionStrategy, partition_count: u32) -> Self {
        Self {
            strategy,
            partition_count,
            round_robin_counter: std::sync::atomic::AtomicU32::new(0),
            sticky_partition: std::sync::atomic::AtomicU32::new(0),
        }
    }

    /// Determine the partition for a record
    pub fn partition(&self, key: Option<&Bytes>, value: &Bytes) -> PartitionId {
        match self.strategy {
            PartitionStrategy::RoundRobin => self.round_robin(),
            PartitionStrategy::KeyHash => self.key_hash(key),
            PartitionStrategy::Random => self.random(),
            PartitionStrategy::Sticky => self.sticky(),
        }
    }

    fn round_robin(&self) -> PartitionId {
        let counter = self.round_robin_counter.fetch_add(
            1,
            std::sync::atomic::Ordering::Relaxed,
        );
        PartitionId::new(counter % self.partition_count)
    }

    fn key_hash(&self, key: Option<&Bytes>) -> PartitionId {
        if let Some(key) = key {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let hash = hasher.finish();
            PartitionId::new((hash % self.partition_count as u64) as u32)
        } else {
            // If no key, fall back to round-robin
            self.round_robin()
        }
    }

    fn random(&self) -> PartitionId {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        PartitionId::new(rng.gen_range(0..self.partition_count))
    }

    fn sticky(&self) -> PartitionId {
        let partition = self.sticky_partition.load(std::sync::atomic::Ordering::Relaxed);
        PartitionId::new(partition)
    }

    /// Switch to a new sticky partition (used when batch is complete)
    pub fn rotate_sticky(&self) {
        let current = self.sticky_partition.load(std::sync::atomic::Ordering::Relaxed);
        let next = (current + 1) % self.partition_count;
        self.sticky_partition
            .store(next, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get the number of partitions
    pub fn partition_count(&self) -> u32 {
        self.partition_count
    }

    /// Get all partition IDs
    pub fn all_partitions(&self) -> Vec<PartitionId> {
        (0..self.partition_count)
            .map(PartitionId::new)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin_partitioner() {
        let partitioner = Partitioner::new(PartitionStrategy::RoundRobin, 3);

        let p1 = partitioner.partition(None, &Bytes::from("test"));
        let p2 = partitioner.partition(None, &Bytes::from("test"));
        let p3 = partitioner.partition(None, &Bytes::from("test"));
        let p4 = partitioner.partition(None, &Bytes::from("test"));

        assert_eq!(p1.as_u32(), 0);
        assert_eq!(p2.as_u32(), 1);
        assert_eq!(p3.as_u32(), 2);
        assert_eq!(p4.as_u32(), 0);
    }

    #[test]
    fn test_key_hash_partitioner() {
        let partitioner = Partitioner::new(PartitionStrategy::KeyHash, 3);

        let key = Bytes::from("same-key");
        let p1 = partitioner.partition(Some(&key), &Bytes::from("value1"));
        let p2 = partitioner.partition(Some(&key), &Bytes::from("value2"));

        // Same key should always go to the same partition
        assert_eq!(p1, p2);
    }
}

