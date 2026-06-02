//! The query specification for a windowed range query.

use crate::aggregate::Aggregation;
use crate::point::Timestamp;

/// A range query: scan `[from, to)` and fold the matching points into windows
/// of size `window`, applying `agg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuerySpec {
    pub from: Timestamp,
    pub to: Timestamp,
    pub window: u64,
    pub agg: Aggregation,
}

impl QuerySpec {
    pub fn new(from: Timestamp, to: Timestamp, window: u64, agg: Aggregation) -> Self {
        Self { from, to, window, agg }
    }
}
