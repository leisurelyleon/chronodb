# chronodb

> An embedded time-series storage engine with windowed aggregation and retention.

`chronodb` is an in-process time-series engine: it ingests timestamped metric
points into time-partitioned series, answers time-range queries that fold points
into fixed windows (computing min/max/avg/sum/count per window), and enforces a
retention policy that drops data past a configurable horizon. It is embedded —
a library you call directly — not a networked database server.

## The Problem

Metric data arrives as a high-volume stream of `(series, timestamp, value)`
points, and the useful questions are time-ranged and downsampled: "the per-minute
average over the last hour," not "every raw point." `chronodb` stores points so
range scans are efficient, computes windowed aggregates with precise interval
semantics, and discards stale data automatically.

## Architecture

```
chronodb-core    pure engine: time-partitioned storage, range queries, windowed aggregation, retention
chronodb-store   append-only, file-backed persistence behind a write-ahead-log trait
chronodb-cli     the binary: ingest points, run windowed range queries
```

The storage and query core is pure and deterministic — no I/O, no clock — so
ingestion, windowing, and retention are unit-tested against known values.
Persistence sits behind a trait, with an in-memory fake for tests and a
file-backed write-ahead log for durability. See
[`docs/architecture.md`](docs/architecture.md) and
[`docs/query-model.md`](docs/query-model.md) for the windowing semantics.

## Build & Test

```bash
cargo build
cargo test
```

## Run

```bash
# Ingest points and run a downsampling query (per-window averages)
cargo run -p chronodb-cli -- demo

# Query a range with a window and aggregation
cargo run -p chronodb-cli -- query --series cpu --from 0 --to 600 --window 60 --agg avg data.wal
```

## License

MIT — see [LICENSE](LICENSE).
