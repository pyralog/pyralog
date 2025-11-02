use pyralog_core::{LogOffset, Result, DLogError};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Index entry: maps logical offset to physical position
#[derive(Debug, Clone, Copy)]
struct IndexEntry {
    offset: LogOffset,
    position: u64,
    size: u32,
}

const INDEX_ENTRY_SIZE: usize = 20; // 8 + 8 + 4 bytes

/// An index for quickly locating records in a segment
pub struct Index {
    path: PathBuf,
    file: RwLock<File>,
    entries: RwLock<BTreeMap<u64, IndexEntry>>,
}

impl Index {
    /// Create a new index
    pub fn create(segment_path: &Path) -> Result<Self> {
        let path = segment_path.with_extension("index");
        
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        Ok(Self {
            path,
            file: RwLock::new(file),
            entries: RwLock::new(BTreeMap::new()),
        })
    }

    /// Open an existing index
    pub fn open(path: PathBuf) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        let mut entries = BTreeMap::new();
        let mut buffer = vec![0u8; INDEX_ENTRY_SIZE];

        loop {
            match file.read_exact(&mut buffer) {
                Ok(_) => {
                    let offset = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
                    let position = u64::from_le_bytes(buffer[8..16].try_into().unwrap());
                    let size = u32::from_le_bytes(buffer[16..20].try_into().unwrap());

                    entries.insert(
                        offset,
                        IndexEntry {
                            offset: LogOffset::new(offset),
                            position,
                            size,
                        },
                    );
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(DLogError::StorageError(e.to_string())),
            }
        }

        Ok(Self {
            path,
            file: RwLock::new(file),
            entries: RwLock::new(entries),
        })
    }

    /// Add an index entry
    pub fn append(&self, offset: LogOffset, position: u64, size: u32) -> Result<()> {
        let entry = IndexEntry {
            offset,
            position,
            size,
        };

        // Write to file
        let mut file = self.file.write();
        let mut buffer = [0u8; INDEX_ENTRY_SIZE];
        
        buffer[0..8].copy_from_slice(&offset.as_u64().to_le_bytes());
        buffer[8..16].copy_from_slice(&position.to_le_bytes());
        buffer[16..20].copy_from_slice(&size.to_le_bytes());

        file.write_all(&buffer)
            .map_err(|e| DLogError::StorageError(e.to_string()))?;

        // Update in-memory index
        self.entries.write().insert(offset.as_u64(), entry);

        Ok(())
    }

    /// Lookup an offset in the index
    pub fn lookup(&self, offset: LogOffset) -> Option<(u64, u32)> {
        self.entries
            .read()
            .get(&offset.as_u64())
            .map(|entry| (entry.position, entry.size))
    }

    /// Find the largest offset less than or equal to the given offset
    pub fn lookup_le(&self, offset: LogOffset) -> Option<(LogOffset, u64, u32)> {
        self.entries
            .read()
            .range(..=offset.as_u64())
            .next_back()
            .map(|(_, entry)| (entry.offset, entry.position, entry.size))
    }

    /// Get all entries in the index
    pub fn entries(&self) -> Vec<(LogOffset, u64, u32)> {
        self.entries
            .read()
            .values()
            .map(|entry| (entry.offset, entry.position, entry.size))
            .collect()
    }

    /// Sync the index to disk
    pub fn sync(&self) -> Result<()> {
        let file = self.file.read();
        file.sync_all()
            .map_err(|e| DLogError::StorageError(e.to_string()))?;
        Ok(())
    }
}

