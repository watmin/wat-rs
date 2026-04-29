# Arc 093 — BACKLOG

DESIGN settled 2026-04-29 (all 11 open questions resolved).
Predecessor arcs (097 Duration helpers, 098 Clara matcher, 099 +
100 + 101 wat-cli reshape) all sealed same day. Slate is clean;
this arc is the active implementation work.

## Architecture lock — circuit pattern

The reader uses the existing `:wat::std::stream::*` circuit
pattern. Three stages, two bounded(1) channels, drop-cascade
shutdown:

```
SQL producer ──tx/rx── filter ──tx/rx── consumer
```

The SQL stage is a wat-side `stream::spawn-producer` whose
producer lambda calls Rust shims per-row to step the sqlite
cursor. No "Rust thread that returns a Stream<T>" abstraction —
the Rust shims are step-shaped, the stream-stdlib does the
threading.

Per user direction 2026-04-29:
> "sql -> filter -> consumer / same tx,rx pairs lock stepping
> each other"

## Slice 1 — read-handle + step-through stream sources + sqlite indexes — *in progress*

- **Status:** in progress 2026-04-29.
- **Adds (Rust):**
  - `:wat::telemetry::sqlite::ReadHandle` — read-only sqlite
    connection wrapping `rusqlite::Connection`. Opens with
    `journal_mode=WAL` already-set on the writer side; reader
    just opens read-only.
  - `:wat::telemetry::sqlite::LogCursor` — prepared statement
    positioned for SELECT * FROM log ORDER BY time_ns ASC.
    Thread-owned (per arc 053's discipline for stateful Rust
    types under CSP).
  - `:wat::telemetry::sqlite::MetricCursor` — same shape,
    SELECT * FROM metric ORDER BY start_time_ns ASC.
  - Step shims:
    - `(LogCursor/step! cursor) -> :Option<Event::Log>`
    - `(MetricCursor/step! cursor) -> :Option<Event::Metric>`
    Each yields one Tagged-decoded Event variant per call,
    returns `:None` when sqlite3_step says SQLITE_DONE.
- **Adds (wat):**
  - `(:wat::telemetry::sqlite/open path) -> ReadHandle`
  - `(:wat::telemetry::sqlite/log-cursor handle query) -> LogCursor`
  - `(:wat::telemetry::sqlite/metric-cursor handle query) -> MetricCursor`
  - `(:wat::telemetry::sqlite/stream-logs handle query) -> Stream<Event::Log>`
    — wat-side `define` that opens the cursor, hands it to a
    spawn-producer lambda, returns the Stream tuple.
  - `(:wat::telemetry::sqlite/stream-metrics handle query) -> Stream<Event::Metric>`
- **Adds (auto-spawn schema):**
  - Indexes on `log.time_ns`, `log.namespace`,
    `metric.start_time_ns`, `metric.namespace` — load-bearing
    for slice 2's pushdown. High-cardinality columns
    (`uuid`, `metric_name`) intentionally NOT indexed (their
    cardinality approaches row count; wat-side filters
    post-narrowing).
- **Slice-1 query is a stub.** Empty `:wat::telemetry::LogQuery` /
  `MetricQuery` types accepted; SQL is unconstrained
  `SELECT * FROM <table> ORDER BY <time_col> ASC`. Slice 2 fills
  in the WHERE clause assembly.
- **Tests:** integration test that writes a few events via the
  arc-091 writer, opens the .db with the new ReadHandle, streams
  them back, asserts shape + count + ordering.
- **Done when:** `cargo test --workspace` green; reading a
  written .db round-trips through the new stream sources.

## Slice 2 — Constraint enums + Clara-flavored builders + push-down — *ready when 1 lands*

- **Status:** ready when slice 1 ships.
- **Adds:**
  - `:wat::telemetry::LogConstraint` enum (Since / Until /
    Namespace / Caller / Uuid / Level variants).
  - `:wat::telemetry::MetricConstraint` enum (Since / Until /
    Namespace / Uuid / MetricName variants).
  - Per-field builder defines (`since`, `until`, `namespace`,
    `caller`, `uuid`, `level`, `metric-name`) — one-line wraps
    around the variant constructors.
  - Query constructors: `(log-query (vec :LogConstraint ...))`
    and `(metric-query (vec :MetricConstraint ...))` — the
    constraint vec IS the query.
  - Producer-side: cursor opens with WHERE clause assembled from
    the constraint vec. AND across constraints; each constraint
    contributes a `?N` placeholder bound at prepare time.
  - `Since` / `Until` take `:wat::time::Instant` (per arc 097)
    — converted to epoch nanos for the WHERE clause.
- **Done when:** worked-example queries from DESIGN §Worked
  examples (Grace outcomes, Grace>5.0 + cohort metrics) execute
  correctly with the indexes from slice 1 doing the heavy
  filtering before the wat-side matches? predicate runs.

## Slice 3 — Materialization helpers — *ready when 2 lands*

- **Status:** ready when slice 2 ships.
- **Adds:**
  - `(:wat::telemetry::Event::Log/data-ast e) -> :Option<HolonAST>`
    — return the raw Tagged AST from the data column. Cheap;
    pattern-match in wat against shape directly.
  - `(:wat::telemetry::Event::Log/data-value e) -> :Option<Value>`
    — full lift via `:wat::eval-ast!` to a `Value::Struct`. The
    bridge from row bytes to a struct value the Clara-style
    matcher (arc 098) consumes.
- **Done when:** `(matches? (Event::Log/data-value e) (:Foo ...))`
  works against a real .db.

## Slice 4 — Example interrogation scripts — *ready when 3 lands*

- **Status:** ready when slice 3 ships.
- **Adds:**
  - `examples/interrogate/` (TBD on exact location) — both
    worked examples from DESIGN, runnable against a real
    `runs/pulse-*.db` from the lab.
  - Each script is a complete `:user::main` that opens a .db,
    streams events, filters via matches?, prints results.
- **Sibling-arc dependency** (arc 098 Clara matcher) shipped
  2026-04-29; nothing blocking.

## Slice 5 — INSCRIPTION + 058 row — *ready when 4 lands*

- **Status:** ready when 4 ships.
- **Adds:** INSCRIPTION.md sealing the arc; 058
  FOUNDATION-CHANGELOG row in the lab repo; CIRCUIT.md update
  if the diagram needs the new reader stage drawn.

## Cross-cutting fog

- **Test fixtures** — slice 1's tests need a writer-side fixture
  that emits a known set of events. Likely: piggyback on the
  existing wat-telemetry-sqlite tests, which already write
  events via the auto-spawn pipeline. We open the resulting
  .db with the new read-handle and stream it back.
- **Connection vs cursor lifecycle** — the prepared statement
  (cursor) holds a borrow on the Connection. With the
  ThreadOwnedCell discipline, the cursor must live in the same
  thread-owned cell as the connection it borrows from. Arc 053's
  pattern handles this; we follow the same shape.
- **`:Option<Event>` step return** — the step shim returns
  `:None` on SQLITE_DONE. The producer lambda checks this and
  returns from its loop, dropping its end of the channel —
  drop-cascade kicks in.
- **WAL mode coordination** — writer side already sets
  `journal_mode=WAL`; the reader inherits (WAL persists across
  open/close cycles). No reader-side configuration needed.
- **Indexes vs writer perf** — adding indexes slows the writer
  marginally per-INSERT. Acceptable: telemetry write is already
  batched (arc 089/095); the read-time speedup from indexed
  WHERE clauses (slice 2) is the load-bearing benefit.
