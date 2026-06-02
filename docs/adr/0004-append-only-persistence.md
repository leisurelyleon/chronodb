# 4. Append-only persistence behind a trait

- Status: Accepted
- Date: 2026-06

## Context

The pure core must stay testable without disk, yet real use needs durability.

## Decision

Define a `WriteAheadLog` trait. Ingested points are appended as records;
replaying the log rebuilds the store. `InMemoryWal` serves tests; `FileWal`
appends one JSON record per line for durability. The persistence crate is
decoupled from the core and logs primitive records.

## Consequences

- The core is tested with no filesystem dependency.
- Durability is append-only: simple, crash-friendly, and replayable.
- A different backend (e.g. binary framing) can implement the same trait.
