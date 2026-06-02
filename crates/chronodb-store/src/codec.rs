//! Encoding records to and from the append-only log (one JSON object per line).

use crate::wal::{StoreError, WalRecord};

/// Encodes a record to a single JSON line (no trailing newline).
pub fn encode(record: &WalRecord) -> Result<String, StoreError> {
    Ok(serde_json::to_string(record)?)
}

/// Decodes a record from a single JSON line.
pub fn decode(line: &str) -> Result<WalRecord, StoreError> {
    Ok(serde_json::from_str(line)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_a_record() {
        let record = WalRecord {
            series: "cpu".into(),
            timestamp: 42,
            value: 3.5,
        };
        let line = encode(&record).unwrap();
        assert_eq!(decode(&line).unwrap(), record);
    }

    #[test]
    fn decode_rejects_garbage() {
        assert!(decode("not json").is_err());
    }
}
