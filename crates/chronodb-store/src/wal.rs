//! The write-ahead-log abstraction, with an in-memory fake and a file-backed
//! implementation.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::codec;

/// Errors raised by the persistence layer.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("codec error: {0}")]
    Codec(#[from] serde_json::Error),
}

/// One persisted measurement: which series, when, and what value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalRecord {
    pub series: String,
    pub timestamp: u64,
    pub value: f64,
}

/// An append-only log of records, replayable to reconstruct state.
pub trait WriteAheadLog {
    /// Appends a record durably.
    fn append(&mut self, record: &WalRecord) -> Result<(), StoreError>;

    /// Replays all records, in append order.
    fn replay(&self) -> Result<Vec<WalRecord>, StoreError>;
}

/// An in-memory WAL for tests and ephemeral use. Real append semantics, no disk.
#[derive(Debug, Default)]
pub struct InMemoryWal {
    records: Vec<WalRecord>,
}

impl InMemoryWal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl WriteAheadLog for InMemoryWal {
    fn append(&mut self, record: &WalRecord) -> Result<(), StoreError> {
        self.records.push(record.clone());
        Ok(())
    }

    fn replay(&self) -> Result<Vec<WalRecord>, StoreError> {
        Ok(self.records.clone())
    }
}

/// A file-backed WAL: each record is one JSON line appended to a file.
#[derive(Debug)]
pub struct FileWal {
    path: PathBuf,
}

impl FileWal {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl WriteAheadLog for FileWal {
    fn append(&mut self, record: &WalRecord) -> Result<(), StoreError> {
        let line = codec::encode(record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }

    fn replay(&self) -> Result<Vec<WalRecord>, StoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)?;
        let mut records = Vec::new();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            records.push(codec::decode(line)?);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(ts: u64, value: f64) -> WalRecord {
        WalRecord {
            series: "cpu".into(),
            timestamp: ts,
            value,
        }
    }

    #[test]
    fn in_memory_appends_and_replays() {
        let mut wal = InMemoryWal::new();
        wal.append(&record(10, 1.0)).unwrap();
        wal.append(&record(20, 2.0)).unwrap();
        let replayed = wal.replay().unwrap();
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[1].timestamp, 20);
    }

    #[test]
    fn file_wal_roundtrips_through_disk() {
        let path = std::env::temp_dir().join("chronodb_file_wal_roundtrip.wal");
        let _ = fs::remove_file(&path); // start clean

        {
            let mut wal = FileWal::new(&path);
            wal.append(&record(10, 1.5)).unwrap();
            wal.append(&record(20, 2.5)).unwrap();
        }

        // A fresh FileWal over the same path replays what was written.
        let wal = FileWal::new(&path);
        let replayed = wal.replay().unwrap();
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].value, 1.5);
        assert_eq!(replayed[1].timestamp, 20);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn replay_of_missing_file_is_empty() {
        let path = std::env::temp_dir().join("chronodb_definitely_absent.wal");
        let _ = fs::remove_file(&path);
        let wal = FileWal::new(&path);
        assert!(wal.replay().unwrap().is_empty());
    }
}
