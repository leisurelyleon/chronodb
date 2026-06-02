//! The fundamental data types.

/// A point's timestamp, in abstract integer ticks (caller chooses the unit).
pub type Timestamp = u64;

/// A measured value.
pub type Value = f64;

/// A series identifier (e.g. a metric name).
pub type SeriesKey = String;

/// A single timestamped measurement within a series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataPoint {
    pub timestamp: Timestamp,
    pub value: Value,
}

impl DataPoint {
    pub fn new(timestamp: Timestamp, value: Value) -> Self {
        Self { timestamp, value }
    }
}
