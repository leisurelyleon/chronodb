//! Core error type.

/// Errors produced by the pure storage and query engine.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("window size must be positive")]
    InvalidWindow,
}
