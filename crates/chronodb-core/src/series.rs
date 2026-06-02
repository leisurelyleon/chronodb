//! A single series: time-partitioned points with range scans and retention.

use std::collections::BTreeMap;

use crate::partition::Partition;
use crate::point::{DataPoint, Timestamp};
use crate::retention;

/// One metric series, partitioned into fixed-width time buckets.
#[derive(Debug, Clone)]
pub struct Series {
    partition_size: u64,
    partitions: BTreeMap<Timestamp, Partition>,
}

impl Series {
    pub fn new(partition_size: u64) -> Self {
        Self {
            partition_size: partition_size.max(1),
            partitions: BTreeMap::new(),
        }
    }

    fn partition_start(&self, ts: Timestamp) -> Timestamp {
        (ts / self.partition_size) * self.partition_size
    }

    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    pub fn point_count(&self) -> usize {
        self.partitions.values().map(Partition::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.point_count() == 0
    }

    /// Ingests a point into its time-aligned partition.
    pub fn insert(&mut self, point: DataPoint) {
        // Resolve the partition start and size first so neither borrows `self`
        // across the mutable `entry` call below.
        let start = self.partition_start(point.timestamp);
        let size = self.partition_size;
        self.partitions
            .entry(start)
            .or_insert_with(|| Partition::new(start, size))
            .insert(point);
    }

    /// Collects all points in the half-open range `[from, to)`, in ascending
    /// timestamp order.
    pub fn range(&self, from: Timestamp, to: Timestamp) -> Vec<DataPoint> {
        if from >= to {
            return Vec::new();
        }
        let aligned_from = self.partition_start(from);
        let mut out = Vec::new();
        for partition in self.partitions.range(aligned_from..to).map(|(_, p)| p) {
            out.extend_from_slice(partition.range(from, to));
        }
        out
    }

    /// Drops partitions that lie entirely before `cutoff`. Returns how many were
    /// removed.
    pub fn retain(&mut self, cutoff: Timestamp) -> usize {
        let size = self.partition_size;
        let expired: Vec<Timestamp> = self
            .partitions
            .iter()
            .filter(|(start, _)| retention::is_expired(**start, size, cutoff))
            .map(|(start, _)| *start)
            .collect();
        for start in &expired {
            self.partitions.remove(start);
        }
        expired.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series_with_points() -> Series {
        let mut series = Series::new(60);
        for t in (0u64..120).step_by(10) {
            series.insert(DataPoint::new(t, t as f64));
        }
        series
    }

    #[test]
    fn points_land_in_correct_partitions() {
        let series = series_with_points();
        // partition_size 60 -> partitions [0,60) and [60,120).
        assert_eq!(series.partition_count(), 2);
        assert_eq!(series.point_count(), 12);
    }

    #[test]
    fn range_spans_partition_boundaries() {
        let series = series_with_points();
        // [50, 70) crosses the partition boundary: ts 50 and 60.
        let scanned = series.range(50, 70);
        let timestamps: Vec<u64> = scanned.iter().map(|p| p.timestamp).collect();
        assert_eq!(timestamps, vec![50, 60]);
    }

    #[test]
    fn range_returns_ascending_order() {
        let series = series_with_points();
        let all = series.range(0, 120);
        let mut sorted = all.clone();
        sorted.sort_by_key(|p| p.timestamp);
        let a: Vec<u64> = all.iter().map(|p| p.timestamp).collect();
        let b: Vec<u64> = sorted.iter().map(|p| p.timestamp).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn retention_drops_old_partitions() {
        let mut series = series_with_points();
        // cutoff 60 -> partition [0,60) expires, [60,120) stays.
        let dropped = series.retain(60);
        assert_eq!(dropped, 1);
        assert_eq!(series.partition_count(), 1);
        assert_eq!(series.point_count(), 6); // ts 60..110
    }
}
