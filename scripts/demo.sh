#!/usr/bin/env bash
# Run the in-memory demonstration: ingest a metric, downsample it, apply
# retention, and re-query to show compaction.
set -euo pipefail

cargo build --release

echo "== chronodb demonstration =="
./target/release/chronodb demo
