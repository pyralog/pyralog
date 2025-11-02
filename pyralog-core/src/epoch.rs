use serde::{Deserialize, Serialize};
use std::fmt;

/// Epoch number for tracking log generations
/// 
/// Inspired by LogDevice's epoch system. Epochs are monotonically increasing
/// numbers that identify which sequencer (leader) wrote records. When a 
/// sequencer fails and a new one takes over, it gets a new epoch number.
/// This prevents ambiguity during recovery and ensures ordering guarantees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Epoch(pub u64);

impl Epoch {
    pub const INVALID: Epoch = Epoch(0);
    pub const FIRST: Epoch = Epoch(1);
    pub const MAX: Epoch = Epoch(u64::MAX);

    #[inline]
    pub fn new(epoch: u64) -> Self {
        Epoch(epoch)
    }

    #[inline]
    pub fn next(&self) -> Self {
        Epoch(self.0.saturating_add(1))
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

impl From<u64> for Epoch {
    fn from(epoch: u64) -> Self {
        Epoch(epoch)
    }
}

/// Epoch-based offset that combines epoch and offset within epoch
/// 
/// Format: [Epoch (32 bits)][Offset within epoch (32 bits)]
/// This is similar to LogDevice's LSN (Log Sequence Number) format
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EpochOffset {
    pub epoch: Epoch,
    pub offset: u32,
}

impl EpochOffset {
    pub const INVALID: EpochOffset = EpochOffset {
        epoch: Epoch::INVALID,
        offset: 0,
    };

    pub fn new(epoch: Epoch, offset: u32) -> Self {
        EpochOffset { epoch, offset }
    }

    /// Create from a 64-bit LSN value
    pub fn from_lsn(lsn: u64) -> Self {
        let epoch = Epoch::new((lsn >> 32) as u64);
        let offset = (lsn & 0xFFFFFFFF) as u32;
        EpochOffset { epoch, offset }
    }

    /// Convert to a 64-bit LSN value
    pub fn to_lsn(&self) -> u64 {
        (self.epoch.0 << 32) | (self.offset as u64)
    }

    pub fn next(&self) -> Self {
        EpochOffset {
            epoch: self.epoch,
            offset: self.offset.saturating_add(1),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.epoch.is_valid()
    }
}

impl fmt::Display for EpochOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.epoch, self.offset)
    }
}

/// Epoch metadata stored per log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochMetadata {
    /// Current epoch number
    pub current_epoch: Epoch,
    
    /// Sequencer node for this epoch
    pub sequencer_node: u64,
    
    /// Start offset of this epoch in the global log
    pub start_offset: u64,
    
    /// Whether this epoch is sealed (no more writes)
    pub sealed: bool,
    
    /// Last known offset in this epoch
    pub last_known_offset: Option<u32>,
}

impl EpochMetadata {
    pub fn new(epoch: Epoch, sequencer_node: u64, start_offset: u64) -> Self {
        Self {
            current_epoch: epoch,
            sequencer_node,
            start_offset,
            sealed: false,
            last_known_offset: None,
        }
    }

    /// Seal this epoch (no more writes allowed)
    pub fn seal(&mut self, last_offset: u32) {
        self.sealed = true;
        self.last_known_offset = Some(last_offset);
    }

    /// Check if this epoch can accept writes
    pub fn can_write(&self) -> bool {
        !self.sealed
    }
}

/// Epoch store for tracking epoch metadata
#[derive(Debug, Clone)]
pub struct EpochStore {
    epochs: Vec<EpochMetadata>,
}

impl EpochStore {
    pub fn new() -> Self {
        Self {
            epochs: Vec::new(),
        }
    }

    /// Start a new epoch
    pub fn start_epoch(&mut self, sequencer_node: u64, start_offset: u64) -> Epoch {
        let epoch = if let Some(last) = self.epochs.last() {
            last.current_epoch.next()
        } else {
            Epoch::FIRST
        };

        self.epochs.push(EpochMetadata::new(epoch, sequencer_node, start_offset));
        epoch
    }

    /// Get metadata for an epoch
    pub fn get_epoch(&self, epoch: Epoch) -> Option<&EpochMetadata> {
        self.epochs
            .iter()
            .find(|e| e.current_epoch == epoch)
    }

    /// Get mutable metadata for an epoch
    pub fn get_epoch_mut(&mut self, epoch: Epoch) -> Option<&mut EpochMetadata> {
        self.epochs
            .iter_mut()
            .find(|e| e.current_epoch == epoch)
    }

    /// Get the current (latest) epoch
    pub fn current_epoch(&self) -> Option<Epoch> {
        self.epochs.last().map(|e| e.current_epoch)
    }

    /// Seal an epoch
    pub fn seal_epoch(&mut self, epoch: Epoch, last_offset: u32) -> bool {
        if let Some(metadata) = self.get_epoch_mut(epoch) {
            metadata.seal(last_offset);
            true
        } else {
            false
        }
    }

    /// Find which epoch contains a global offset
    pub fn epoch_for_offset(&self, global_offset: u64) -> Option<Epoch> {
        for metadata in self.epochs.iter().rev() {
            if global_offset >= metadata.start_offset {
                return Some(metadata.current_epoch);
            }
        }
        None
    }

    /// Convert global offset to epoch offset
    pub fn to_epoch_offset(&self, global_offset: u64) -> Option<EpochOffset> {
        let epoch = self.epoch_for_offset(global_offset)?;
        let metadata = self.get_epoch(epoch)?;
        let offset_in_epoch = (global_offset - metadata.start_offset) as u32;
        Some(EpochOffset::new(epoch, offset_in_epoch))
    }

    /// Convert epoch offset to global offset
    pub fn to_global_offset(&self, epoch_offset: EpochOffset) -> Option<u64> {
        let metadata = self.get_epoch(epoch_offset.epoch)?;
        Some(metadata.start_offset + epoch_offset.offset as u64)
    }
}

impl Default for EpochStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_ordering() {
        let e1 = Epoch::new(1);
        let e2 = Epoch::new(2);
        assert!(e1 < e2);
        assert_eq!(e1.next(), e2);
    }

    #[test]
    fn test_epoch_offset() {
        let eo = EpochOffset::new(Epoch::new(5), 100);
        assert_eq!(eo.epoch, Epoch::new(5));
        assert_eq!(eo.offset, 100);
        
        let lsn = eo.to_lsn();
        let eo2 = EpochOffset::from_lsn(lsn);
        assert_eq!(eo, eo2);
    }

    #[test]
    fn test_epoch_store() {
        let mut store = EpochStore::new();
        
        // Start first epoch
        let e1 = store.start_epoch(1, 0);
        assert_eq!(e1, Epoch::FIRST);
        
        // Start second epoch
        let e2 = store.start_epoch(2, 1000);
        assert_eq!(e2, Epoch::new(2));
        
        // Test offset conversion
        let global_offset = 1050;
        let epoch_offset = store.to_epoch_offset(global_offset).unwrap();
        assert_eq!(epoch_offset.epoch, e2);
        assert_eq!(epoch_offset.offset, 50);
        
        // Convert back
        let global = store.to_global_offset(epoch_offset).unwrap();
        assert_eq!(global, global_offset);
    }

    #[test]
    fn test_epoch_sealing() {
        let mut store = EpochStore::new();
        let epoch = store.start_epoch(1, 0);
        
        assert!(store.get_epoch(epoch).unwrap().can_write());
        
        store.seal_epoch(epoch, 999);
        
        assert!(!store.get_epoch(epoch).unwrap().can_write());
        assert_eq!(store.get_epoch(epoch).unwrap().last_known_offset, Some(999));
    }
}

