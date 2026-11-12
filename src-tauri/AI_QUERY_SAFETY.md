# AI Query Safety

## Principle

AI-generated SQL must never receive an unbounded opportunity to consume database
resources. The model proposes a query; the application decides whether it can run, how
long it may run, and how much data it may return. When safety cannot be established,
execution fails closed and the investigation continues from existing evidence.

User approval permits a write but never disables resource limits, cancellation,
concurrency controls, or circuit breaking.

## Mandatory execution path

```text
model tool call
  -> SQL safety validation
  -> single-statement classification
  -> approval gate for writes
  -> bounded EXPLAIN risk preflight for unapproved reads
  -> global and per-connection scheduler
  -> database-enforced read-only execution for unapproved reads
  -> timeout and database-task cancellation
  -> row and serialized-byte limits
  -> audit result and circuit-breaker update
```

No AI tool or transport may bypass this application-layer path.

## Hard invariants

1. Unapproved SQL runs only through `DatabaseDriver::execute_read_only_query`.
   PostgreSQL, MySQL, and SQLite enforce read-only state in the database and always
   roll it back. If cancellation interrupts cleanup, the physical connection is
   discarded instead of returning to the pool.
2. Automated SQL is one statement. Locking reads, transaction control, delay and
   benchmark functions, recursive row generators, Cartesian joins, random full-table
   sorts, excessive joins or nesting, and known unbounded operations are rejected.
3. Every AI database workload holds global and per-connection scheduler permits.
   Waiting for capacity is bounded.
4. A ChatDBA query returns at most 50 rows and at most 2 MiB of serialized result data.
   Built-in drivers stop retaining rows as soon as the byte budget is exhausted; the
   application checks the completed envelope again. A model-provided limit may lower
   these ceilings but cannot raise them.
5. Query execution is time-bounded. Timeout or user stop aborts the registered database
   task; a read-only driver must not return an in-flight connection to its pool.
6. Repeated failures open a per-connection AI circuit breaker. Normal editor traffic
   remains independent from the AI scheduler.
7. ChatDBA does not expose `EXPLAIN ANALYZE`, because it executes the statement.
8. Safety policy belongs in shared application services so desktop and web transports
   behave identically.
9. Before an unapproved read executes, ChatDBA runs plain `EXPLAIN` with its own timeout.
   Unrecognized plans, Cartesian joins, extreme cardinality estimates, large full scans,
   and large scan-plus-sort plans fail closed.
10. Each turn also has cumulative ceilings: 20 database queries, 30 seconds of
    database tool time, 8 MiB of tool results, 20 query preflights, and two consecutive queries taking at
    least eight seconds.
11. Automated reads use a separate one-connection pool identity. Editor, backup, and
    other user-initiated workloads retain their normal pools.
12. Drivers explicitly declare database read-only, statement timeout, lock timeout,
    safe cancellation, streaming byte limit, EXPLAIN preflight, and isolated-pool
    support. The default declares nothing and disables unattended reads.
13. SQLite installs an engine progress handler that interrupts CPU-bound statements
    after ten seconds, independently of the outer Rust task timeout.

## Current limits

| Resource | Limit |
| --- | ---: |
| Chat rows per query | 50 |
| Serialized chat result | 2 MiB |
| Chat query execution | 10 seconds |
| PostgreSQL/MySQL/MariaDB statement timeout | 10 seconds |
| PostgreSQL/SQLite lock wait | 1 second |
| Scheduler wait | 5 seconds |
| Concurrent AI queries globally | 4 |
| AI pool connections per connection | 1 |
| Database queries per turn | 20 |
| Cumulative database tool time per turn | 30 seconds |
| Cumulative tool-result bytes per turn | 8 MiB |
| EXPLAIN preflights per turn | 20 |
| Consecutive queries at least 8 seconds | 2 |
| Consecutive failures before circuit opens | 3 |
| Circuit cooldown | 30 seconds |

Constants in `src/application/ai_query_guard.rs`, `src/application/queries.rs`,
and `src/application/chat/tools.rs` are authoritative. Update this document and any UI
labels when those limits change.

## Fail-closed behaviour

- Unknown or unsupported statement classifications are denied.
- Plugin drivers without database-enforced read-only execution rely on classification
  and must not claim the stronger built-in-driver guarantee.
- Oversized results are discarded and never forwarded to the model.
- Timeout, cancellation, scheduler saturation, an open circuit, or cleanup failure ends
  the tool call without automatically retrying SQL.

## Planned strengthening

1. Add connection-health and latency-based circuit breaking.
4. Display the effective timeout, row cap, byte cap, read-only guarantee, and rejection
   reason in the tool-call card and audit export.
5. Add fault-injection tests proving that timeout and cancellation discard connections
   when rollback cannot complete.

Future changes may make the policy stricter. They must never create a path that lets a
model or provider increase a hard resource ceiling.
