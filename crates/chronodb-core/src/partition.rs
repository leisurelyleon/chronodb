//! A time-bucketed partition of points, kept sorted by timestamp.

use crate::point::{DataPoint, Timestamp};

/// A partition covering `[start, start + size)`, holding its points sorted by
/// timestamp for efficient range scans.
#[derive(Debug, Clone)]
pub struct Partition {
    start: Timestamp,
    size: u64,
    points: Vec<DataPoint>,
}

impl Partition {
    pub fn new(start: Timestamp, size: u64) -> Self {
        Self {
            start,
            size,
            points: Vec::new(),
        }
    }

    pub fn start(&self) -> Timestamp {
        self.start
    }

    /// The exclusive end of the partition's time span.
    pub fn end(&self) -> Timestamp {
        self.start.saturating_add(self.size)
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Inserts a point, preserving ascending timestamp order. Points with equal
    /// timestamps retain insertion order (stable).
    pub fn insert(&mut self, point: DataPoint) {
        let idx = self
            .points
            .partition_point(|p| p.timestamp <= point.timestamp);
        self.points.insert(idx, point);
    }

    /// The points whose timestamps lie in the half-open range `[from, to)`.
    pub fn range(&self, from: Timestamp, to: Timestamp) -> &[DataPoint] {
        let lo = self.points.partition_point(|p| p.timestamp < from);
        let hi = self.points.partition_point(|p| p.timestamp < to);
        &self.points[lo..hi]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_keeps_sorted() {
        let mut partition = Partition::new(0, 100);
        partition.insert(DataPoint::new(30, 3.0));
        partition.insert(DataPoint::new(10, 1.0));
        partition.insert(DataPoint::new(20, 2.0));
        let all = partition.range(0, 100);
        let timestamps: Vec<u64> = all.iter().map(|p| p.timestamp).collect();
        assert_eq!(timestamps, vec![10, 20, 30]);
    }

    #[test]
    fn range_is_half_open() {
        let mut partition = Partition::new(0, 100);
        for t in [0u64, 10, 20, 30, 40] {
            partition.insert(DataPoint::new(t, t as f64));
        }
        // [10, 30): includes 10 and 20, excludes 30.
        let slice = partition.range(10, 30);
        let timestamps: Vec<u64> = slice.iter().map(|p| p.timestamp).collect();
        assert_eq!(timestamps, vec![10, 20]);
    }

    #[test]
    fn end_is_start_plus_size() {
        assert_eq!(Partition::new(60, 60).end(), 120);
    }
}
