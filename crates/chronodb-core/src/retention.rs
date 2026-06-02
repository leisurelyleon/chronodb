//! Retention policy — pure decisions about which partitions have expired.

use crate::point::Timestamp;

/// The oldest timestamp still retained, given the current logical time and a
/// retention horizon. Saturates at zero.
pub fn cutoff(now: Timestamp, horizon: u64) -> Timestamp {
    now.saturating_sub(horizon)
}

/// Whether a partition spanning `[start, start + size)` lies entirely before the
/// cutoff and may therefore be dropped.
pub fn is_expired(start: Timestamp, size: u64, cutoff: Timestamp) -> bool {
    start.saturating_add(size) <= cutoff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoff_saturates() {
        assert_eq!(cutoff(100, 60), 40);
        assert_eq!(cutoff(30, 60), 0);
    }

    #[test]
    fn partition_entirely_before_cutoff_expires() {
        // [0,60) with cutoff 60: end == cutoff -> expired (half-open).
        assert!(is_expired(0, 60, 60));
        // [60,120) with cutoff 60: not expired.
        assert!(!is_expired(60, 60, 60));
        // [0,60) with cutoff 30: end 60 > 30 -> not expired.
        assert!(!is_expired(0, 60, 30));
    }
}
