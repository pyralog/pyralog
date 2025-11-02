use pyralog_core::{Result, DLogError};
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

use crate::state::{PersistentState, LogEntry};

/// Persistent log storage for Raft
pub struct RaftLog {
    path: PathBuf,
    file: RwLock<File>,
}

impl RaftLog {
    pub fn open(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        Ok(Self {
            path,
            file: RwLock::new(file),
        })
    }

    /// Save persistent state to disk
    pub fn save_state(&self, state: &PersistentState) -> Result<()> {
        let data = bincode::serialize(state)
            .map_err(|e| DLogError::SerializationError(e.to_string()))?;

        let mut file = self.file.write();
        file.seek(SeekFrom::Start(0))
            .map_err(|e| DLogError::StorageError(e.to_string()))?;
        file.write_all(&data)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;
        file.sync_all()
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// Load persistent state from disk
    pub fn load_state(&self) -> Result<PersistentState> {
        let mut file = self.file.write();
        file.seek(SeekFrom::Start(0))
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        if buffer.is_empty() {
            return Ok(PersistentState::default());
        }

        bincode::deserialize(&buffer)
            .map_err(|e| DLogError::SerializationError(e.to_string()))
    }
}

