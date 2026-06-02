//! Append-only, file-backed persistence for `chronodb`.
//!
//! A [`WriteAheadLog`] records [`WalRecord`]s durably and replays them to
//! reconstruct state. [`InMemoryWal`] is a real, disk-free implementation for
//! tests; [`FileWal`] appends one JSON line per record to a file.

pub mod codec;
pub mod wal;

pub use wal::{FileWal, InMemoryWal, StoreError, WalRecord, WriteAheadLog};
