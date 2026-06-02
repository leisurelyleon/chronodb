# chronodb
An embedded time-series storage engine in Rust. Ingests timestamped metric points into time-partitioned series, answers range queries with windowed aggregation (min/max/avg/sum, downsampling), and enforces retention policies — with a pure, deterministic storage core, an append-only file-backed persistence layer, and a query CLI.
