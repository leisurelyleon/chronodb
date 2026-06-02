# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial workspace scaffold: chronodb-core, chronodb-store, chronodb-cli.

## [0.1.0] - TBD

### Added
- Time-partitioned storage core for timestamped metric points.
- Range queries with windowed aggregation (min/max/avg/sum/count) and downsampling.
- Retention policy dropping partitions past a configurable horizon.
- Append-only, file-backed persistence behind a write-ahead-log trait.
- CLI to ingest points and run windowed range queries.

[Unreleased]: https://github.com/leisurelyleon/chronodb/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/leisurelyleon/chronodb/releases/tag/v0.1.0
