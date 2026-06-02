# 3. Retention and compaction at partition granularity

- Status: Accepted
- Date: 2026-06

## Context

Old data must be discarded to bound storage. Filtering individual points is
costly and leaves fragmented partitions.

## Decision

Express retention as a cutoff (`now - horizon`) and drop any partition lying
entirely before it. Compaction is therefore whole-partition removal, not
per-point deletion.

## Consequences

- Retention is O(partitions), not O(points).
- The cutoff is pure logic, tested independently.
- Granularity is the partition width: a partition is kept until all of it
  expires.
