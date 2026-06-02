# Architecture

`chronodb` is a Rust workspace implementing an embedded time-series storage
engine, organized so the storage and query logic is pure and deterministically
testable.

## Crates

```text
chronodb-core    pure engine: time-partitioned storage, range scans, windowed aggregation, retention
chronodb-store   append-only, file-backed persistence behind a write-ahead-log trait
chronodb-cli     the binary: ingest points, run windowed range queries
```

## Pure core, persistent edge

`chronodb-core` holds no file handles and reads no clock. A `Store` keeps a map
of `Series`; each `Series` partitions its points into fixed-width time buckets
(`Partition`), each kept sorted by timestamp. Range scans, windowed aggregation,
and retention are pure functions, so they are unit-tested against hand-computed
values.

`chronodb-store` is the only crate that touches the filesystem. A
`WriteAheadLog` trait abstracts durability: `InMemoryWal` (a real, disk-free
implementation for tests) and `FileWal` (one JSON record per line, appended).
The crate is decoupled from the core — it logs primitive records; the CLI
bridges replayed records into the core store.

## Data flow

1. **Ingest** — a point is routed to its series and to the time-aligned
   partition within that series, inserted in sorted order.
2. **Query** — a range scan gathers points in `[from, to)` across the relevant
   partitions, then the aggregation engine folds them into fixed windows.
3. **Retention** — partitions lying entirely before the retention cutoff are
   dropped wholesale (cheap compaction at partition granularity).
4. **Persistence** — each ingested point is appended to the WAL; replaying the
   WAL reconstructs the store.

See [`query-model.md`](query-model.md) for the precise windowing semantics.
