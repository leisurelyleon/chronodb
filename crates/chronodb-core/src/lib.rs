//! Pure time-series storage and query engine for `chronodb`.
//!
//! A [`Store`] ingests [`DataPoint`]s into time-partitioned [`Series`], answers
//! windowed range queries via [`QuerySpec`], and enforces retention. Everything
//! here is pure — no I/O, no clock — so windowing and retention are unit-tested
//! against hand-computed values. Persistence lives in `chronodb-store`.

pub mod aggregate;
pub mod error;
pub mod partition;
pub mod point;
pub mod query;
pub mod retention;
pub mod series;
pub mod store;

pub use aggregate::{Aggregation, WindowResult, aggregate};
pub use error::CoreError;
pub use partition::Partition;
pub use point::{DataPoint, SeriesKey, Timestamp, Value};
pub use query::QuerySpec;
pub use series::Series;
pub use store::Store;
