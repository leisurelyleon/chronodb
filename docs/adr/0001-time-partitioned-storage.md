# 1. Time-partitioned storage

- Status: Accepted
- Date: 2026-06

## Context

Time-series data is written in roughly time order, queried by time range, and
expired oldest-first. A flat sorted list makes range queries easy but retention
and locality poor.

## Decision

Partition each series into fixed-width time buckets, each holding its points
sorted by timestamp. A range query touches only the partitions overlapping the
range; retention drops whole partitions.

## Consequences

- Range scans skip irrelevant partitions.
- Retention is cheap: drop a partition rather than filter points.
- Partition width trades off granularity against per-partition overhead.
