//! Windowed aggregation — the heart of the time-series query engine.
//!
//! Points are folded into fixed-size windows aligned to absolute time: a point
//! at timestamp `t` belongs to the window starting at `(t / window) * window`,
//! which spans the half-open interval `[start, start + window)`. A point lying
//! exactly on a boundary belongs to the *upper* window.

use std::collections::BTreeMap;

use crate::error::CoreError;
use crate::point::{DataPoint, Timestamp};

/// The aggregation applied within each window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregation {
    Min,
    Max,
    Sum,
    Avg,
    Count,
}

/// The aggregated result for one window.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowResult {
    /// The window's aligned start timestamp.
    pub start: Timestamp,
    /// The aggregated value.
    pub value: f64,
    /// How many points fell in this window.
    pub count: u64,
}

/// Folds `points` into windows of size `window`, applying `agg` to each.
/// Only windows that contain at least one point are returned, in ascending
/// order of window start. Returns an error if `window` is zero.
pub fn aggregate(
    points: &[DataPoint],
    window: u64,
    agg: Aggregation,
) -> Result<Vec<WindowResult>, CoreError> {
    if window == 0 {
        return Err(CoreError::InvalidWindow);
    }

    // Group values by their aligned window start. BTreeMap keeps windows in
    // ascending order deterministically.
    let mut groups: BTreeMap<Timestamp, Vec<f64>> = BTreeMap::new();
    for point in points {
        let start = (point.timestamp / window) * window;
        groups.entry(start).or_default().push(point.value);
    }

    let mut results = Vec::with_capacity(groups.len());
    for (start, values) in groups {
        let count = values.len() as u64;
        let value = match agg {
            Aggregation::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
            Aggregation::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            Aggregation::Sum => values.iter().copied().sum::<f64>(),
            Aggregation::Avg => values.iter().copied().sum::<f64>() / count as f64,
            Aggregation::Count => count as f64,
        };
        results.push(WindowResult { start, value, count });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points() -> Vec<DataPoint> {
        // ts:  0  10  20  30  40  50
        // val: 1   2   3   4   5   6
        vec![
            DataPoint::new(0, 1.0),
            DataPoint::new(10, 2.0),
            DataPoint::new(20, 3.0),
            DataPoint::new(30, 4.0),
            DataPoint::new(40, 5.0),
            DataPoint::new(50, 6.0),
        ]
    }

    #[test]
    fn zero_window_is_rejected() {
        assert!(matches!(aggregate(&points(), 0, Aggregation::Avg), Err(CoreError::InvalidWindow)));
    }

    #[test]
    fn avg_per_thirty_tick_window() {
        // [0,30): 1,2,3 -> avg 2.0 ; [30,60): 4,5,6 -> avg 5.0
        let result = aggregate(&points(), 30, Aggregation::Avg).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], WindowResult { start: 0, value: 2.0, count: 3 });
        assert_eq!(result[1], WindowResult { start: 30, value: 5.0, count: 3 });
    }

    #[test]
    fn min_max_sum_count() {
        let min = aggregate(&points(), 30, Aggregation::Min).unwrap();
        assert_eq!(min[0].value, 1.0);
        assert_eq!(min[1].value, 4.0);

        let max = aggregate(&points(), 30, Aggregation::Max).unwrap();
        assert_eq!(max[0].value, 3.0);
        assert_eq!(max[1].value, 6.0);

        let sum = aggregate(&points(), 30, Aggregation::Sum).unwrap();
        assert_eq!(sum[0].value, 6.0); // 1+2+3
        assert_eq!(sum[1].value, 15.0); // 4+5+6

        let count = aggregate(&points(), 30, Aggregation::Count).unwrap();
        assert_eq!(count[0].value, 3.0);
        assert_eq!(count[0].count, 3);
    }

    #[test]
    fn boundary_point_falls_in_upper_window() {
        // A single point exactly on the boundary t=30 with window 30 must land
        // in window [30,60), not [0,30).
        let result = aggregate(&[DataPoint::new(30, 9.0)], 30, Aggregation::Sum).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start, 30);
    }

    #[test]
    fn single_wide_window_holds_everything() {
        let result = aggregate(&points(), 1000, Aggregation::Count).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 6);
        assert_eq!(result[0].start, 0);
    }
}
