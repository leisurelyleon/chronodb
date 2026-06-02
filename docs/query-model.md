# Query Model

A query scans a half-open time range and folds the matching points into
fixed-size windows, applying one aggregation per window.

## Time ranges are half-open

A range `[from, to)` includes points at `from` and excludes points at `to`.
This makes adjacent ranges tile cleanly: `[0, 60)` and `[60, 120)` partition the
timeline with no overlap and no gap.

## Window alignment

Windows are aligned to absolute time, not to the query start. A point at
timestamp `t` belongs to the window beginning at:

```text
window_start = (t / window) * window
```

spanning the half-open interval `[window_start, window_start + window)`. A point
lying exactly on a boundary belongs to the **upper** window: with `window = 30`,
a point at `t = 30` falls in `[30, 60)`, not `[0, 30)`.

Absolute alignment means the same point always lands in the same window
regardless of the query's `from`, so results from overlapping queries agree.

## Aggregations

Within each window, one of: `min`, `max`, `sum`, `avg`, `count`. Only windows
containing at least one point are returned, in ascending order of window start.

## Worked example

Points (timestamp, value): (0,1) (10,2) (20,3) (30,4) (40,5) (50,6), window 30:

- `[0, 30)` holds 1, 2, 3 — avg 2, sum 6, min 1, max 3, count 3
- `[30, 60)` holds 4, 5, 6 — avg 5, sum 15, min 4, max 6, count 3
