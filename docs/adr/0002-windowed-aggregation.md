# 2. Windowed aggregation with half-open, absolutely-aligned windows

- Status: Accepted
- Date: 2026-06

## Context

Downsampling is the core time-series read. The subtle part is window boundaries:
an inconsistent convention double-counts or drops points at edges.

## Decision

Use half-open `[start, start + window)` windows aligned to absolute time
(`start = (t / window) * window`). A boundary point belongs to the upper window.
The convention is identical to the half-open range-scan convention.

## Consequences

- Windows tile the timeline with no overlap or gap.
- A point always maps to the same window regardless of query start.
- The rule is documented and tested against hand-computed values.
