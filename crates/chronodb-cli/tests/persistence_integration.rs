//! End-to-end integration: write points through the real file-backed WAL, then
//! replay them into the core store and run windowed queries — proving the
//! persistence layer and the query engine fit together through actual disk I/O.

use std::fs;
use std::path::PathBuf;

use chronodb_core::{Aggregation, DataPoint, QuerySpec, Store};
use chronodb_store::{FileWal, WalRecord, WriteAheadLog};

/// A unique temp path per test, so parallel runs never collide.
fn temp_wal(tag: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("chronodb_it_{tag}.wal"));
    let _ = fs::remove_file(&path);
    path
}

/// Replays a WAL file into a freshly built store.
fn store_from_wal(path: &PathBuf, partition_size: u64) -> Store {
    let wal = FileWal::new(path);
    let records = wal.replay().expect("replay should succeed");
    let mut store = Store::new(partition_size);
    for record in &records {
        store.ingest(
            record.series.clone(),
            DataPoint::new(record.timestamp, record.value),
        );
    }
    store
}

#[test]
fn wal_replay_feeds_windowed_query() {
    let path = temp_wal("replay_query");

    // Write cpu points (value = ts + 10) every 10 ticks over [0, 120) to disk.
    {
        let mut wal = FileWal::new(&path);
        for t in (0u64..120).step_by(10) {
            wal.append(&WalRecord {
                series: "cpu".into(),
                timestamp: t,
                value: (t + 10) as f64,
            })
            .expect("append should succeed");
        }
    }

    // Replay from disk into a store and downsample to 30-tick averages.
    let store = store_from_wal(&path, 3600);
    assert_eq!(store.point_count("cpu"), 12);

    let spec = QuerySpec::new(0, 120, 30, Aggregation::Avg);
    let windows = store.query("cpu", &spec).unwrap();
    let avgs: Vec<f64> = windows.iter().map(|w| w.value).collect();
    // [0,30):10,20,30->20 ; [30,60):40,50,60->50 ; [60,90):70,80,90->80 ; [90,120):100,110,120->110
    assert_eq!(avgs, vec![20.0, 50.0, 80.0, 110.0]);

    let _ = fs::remove_file(&path);
}

#[test]
fn appends_accumulate_across_handles() {
    let path = temp_wal("accumulate");

    // Two separate WAL sessions append to the same file.
    {
        let mut wal = FileWal::new(&path);
        wal.append(&WalRecord {
            series: "mem".into(),
            timestamp: 0,
            value: 1.0,
        })
        .unwrap();
    }
    {
        let mut wal = FileWal::new(&path);
        wal.append(&WalRecord {
            series: "mem".into(),
            timestamp: 60,
            value: 2.0,
        })
        .unwrap();
    }

    // Both points survive: the second handle appended, not overwrote.
    let store = store_from_wal(&path, 60);
    assert_eq!(store.point_count("mem"), 2);

    let spec = QuerySpec::new(0, 120, 60, Aggregation::Sum);
    let windows = store.query("mem", &spec).unwrap();
    assert_eq!(windows.len(), 2); // one point in [0,60), one in [60,120)
    assert_eq!(windows[0].value, 1.0);
    assert_eq!(windows[1].value, 2.0);

    let _ = fs::remove_file(&path);
}

#[test]
fn multi_series_wal_queries_independently() {
    let path = temp_wal("multi_series");

    {
        let mut wal = FileWal::new(&path);
        for t in (0u64..60).step_by(10) {
            wal.append(&WalRecord {
                series: "cpu".into(),
                timestamp: t,
                value: 100.0,
            })
            .unwrap();
            wal.append(&WalRecord {
                series: "mem".into(),
                timestamp: t,
                value: 5.0,
            })
            .unwrap();
        }
    }

    let store = store_from_wal(&path, 3600);
    assert_eq!(store.series_count(), 2);

    let spec = QuerySpec::new(0, 60, 60, Aggregation::Avg);
    assert_eq!(store.query("cpu", &spec).unwrap()[0].value, 100.0);
    assert_eq!(store.query("mem", &spec).unwrap()[0].value, 5.0);

    let _ = fs::remove_file(&path);
}
