//! The multi-series store: ingest, query, and retention across all series.

use std::collections::BTreeMap;

use crate::aggregate::{WindowResult, aggregate};
use crate::error::CoreError;
use crate::point::{DataPoint, SeriesKey, Timestamp};
use crate::query::QuerySpec;
use crate::retention;
use crate::series::Series;

/// An in-memory, time-series store holding multiple partitioned series.
#[derive(Debug, Clone)]
pub struct Store {
    partition_size: u64,
    series: BTreeMap<SeriesKey, Series>,
}

impl Store {
    /// Creates a store whose series partition into buckets of `partition_size`
    /// ticks (clamped to at least 1).
    pub fn new(partition_size: u64) -> Self {
        Self {
            partition_size: partition_size.max(1),
            series: BTreeMap::new(),
        }
    }

    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    pub fn series_keys(&self) -> Vec<SeriesKey> {
        self.series.keys().cloned().collect()
    }

    /// The number of points stored for a series (0 if the series is unknown).
    pub fn point_count(&self, key: &str) -> usize {
        self.series.get(key).map_or(0, Series::point_count)
    }

    /// Ingests a point into the named series, creating the series if needed.
    pub fn ingest(&mut self, key: impl Into<SeriesKey>, point: DataPoint) {
        let size = self.partition_size;
        self.series
            .entry(key.into())
            .or_insert_with(|| Series::new(size))
            .insert(point);
    }

    /// Runs a windowed range query against a series. An unknown series yields an
    /// empty result.
    pub fn query(&self, key: &str, spec: &QuerySpec) -> Result<Vec<WindowResult>, CoreError> {
        let Some(series) = self.series.get(key) else {
            return Ok(Vec::new());
        };
        let points = series.range(spec.from, spec.to);
        aggregate(&points, spec.window, spec.agg)
    }

    /// Applies retention across every series, dropping partitions older than
    /// `horizon` relative to `now`. Returns the total partitions dropped.
    pub fn retain(&mut self, now: Timestamp, horizon: u64) -> usize {
        let cutoff = retention::cutoff(now, horizon);
        self.series
            .values_mut()
            .map(|series| series.retain(cutoff))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::Aggregation;

    fn store_with_data() -> Store {
        let mut store = Store::new(60);
        for t in (0u64..120).step_by(10) {
            store.ingest("cpu", DataPoint::new(t, (t + 10) as f64));
            store.ingest("mem", DataPoint::new(t, 1.0));
        }
        store
    }

    #[test]
    fn tracks_multiple_series() {
        let store = store_with_data();
        assert_eq!(store.series_count(), 2);
        assert_eq!(store.point_count("cpu"), 12);
        assert_eq!(store.point_count("missing"), 0);
    }

    #[test]
    fn windowed_query_downsamples() {
        let store = store_with_data();
        // cpu value = ts + 10; window 30, avg:
        // [0,30): 10,20,30 -> 20 ; [30,60): 40,50,60 -> 50
        // [60,90): 70,80,90 -> 80 ; [90,120): 100,110,120 -> 110
        let spec = QuerySpec::new(0, 120, 30, Aggregation::Avg);
        let result = store.query("cpu", &spec).unwrap();
        let avgs: Vec<f64> = result.iter().map(|w| w.value).collect();
        assert_eq!(avgs, vec![20.0, 50.0, 80.0, 110.0]);
    }

    #[test]
    fn query_unknown_series_is_empty() {
        let store = store_with_data();
        let spec = QuerySpec::new(0, 120, 30, Aggregation::Avg);
        assert!(store.query("nope", &spec).unwrap().is_empty());
    }

    #[test]
    fn retention_applies_across_series() {
        let mut store = store_with_data();
        // now 120, horizon 60 -> cutoff 60 -> drop [0,60) in each of 2 series.
        let dropped = store.retain(120, 60);
        assert_eq!(dropped, 2);
        assert_eq!(store.point_count("cpu"), 6);
        assert_eq!(store.point_count("mem"), 6);
    }

    #[test]
    fn invalid_window_propagates() {
        let store = store_with_data();
        let spec = QuerySpec::new(0, 120, 0, Aggregation::Avg);
        assert!(store.query("cpu", &spec).is_err());
    }
}
