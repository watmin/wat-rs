# Arc 093 — `:wat::telemetry::*` reader / interrogation — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate gained a pry/gdb-style debugging UX over frozen
telemetry `.db` files. A wat script opens a `runs/pulse-*.db`
read-only, streams Event rows through the substrate's `:wat::std::
stream::*` circuit, narrows by time at the SQL layer, filters
in-app via Clara `:wat::form::matches?` predicates over lifted
struct values, and prints / collects what it finds. End-to-end
running concretely in `examples/interrogate/` — six trades
written, two query results matching the designed predicate.

```scheme
(:wat::std::stream::filter
  (:wat::telemetry::sqlite/stream-logs handle
    (:wat::core::vec :wat::telemetry::TimeConstraint
      (:wat::telemetry::since (:wat::time::hours-ago 1))))
  (:wat::core::lambda ((e :wat::telemetry::Event) -> :bool)
    (:wat::core::match (:wat::telemetry::Event::Log/data-value e)
      -> :bool
      ((Some trade)
        (:wat::form::matches? trade
          (:demo::Trade
            (= ?side :side)
            (= ?qty  :qty)
            (= ?side "buy")
            (> ?qty 10))))
      (:None false))))
```

Five slices over one day. Every stage of the arc-093 DESIGN
shipped, plus three substrate completions surfaced along the way.

**Predecessors:**
- [arc 091](../091-wat-measure/INSCRIPTION.md) — telemetry
  writer + sqlite schema + struct→form / eval-ast! foundation.
- [arc 095](../095-service-paired-channels/INSCRIPTION.md) —
  Service<E,G> paired channels (writer side; reader doesn't
  ride on it but the connection-pool concepts settled there).
- [arc 097](../097-wat-time-duration/INSCRIPTION.md) — Duration
  + ActiveSupport-flavored helpers (`hours-ago` etc.). Time-
  range constraint enums depend on these.
- [arc 098](../098-wat-form-matches/INSCRIPTION.md) — Clara-
  style `:wat::form::matches?` matcher. The filter-predicate
  surface arc 093 was designed around.

**Surfaced by:** user direction 2026-04-29:

> "the first real caller is us debugging the program. we have a
> time indexed edn store - i want to do filter evaluations on
> this time indexed data..."

> "to be more direct here - we'll do some run and then we'll
> interrogate the system - we'll build up script templates and
> treat wat like a ruby of sorts - we'll build queries and run
> them. sql to reduce the candidate space and edn into a filter
> func to do pruning"

The arc closed when slice 4's `cargo run -p interrogate-example`
printed the expected "hits: 2" and `cargo test --workspace`
came back green.

---

## What shipped

### Slice 1a — low-cardinality SQL indexes

Auto-spawn schema-install (arc 085's enum-derived path) gains
`CREATE INDEX IF NOT EXISTS` statements alongside its
`CREATE TABLE`. After two revisions during the slice, the line
in the sand was: index TIME columns ONLY. For
`:wat::telemetry::Event` this yields exactly two indexes:
`log.time_ns`, `metric.start_time_ns`. High-cardinality columns
(`uuid`, `metric_name`) intentionally NOT indexed — their
cardinality approaches row count, the index storage dwarfs the
data, the planner can't range-scan them effectively, and the
wat-side matcher post-narrowing is faster than an index seek
would have been over a few hundred rows.

Per user direction:
> "honestly... i think we just do time filter in sqlite and do
> all other content filtering in our stream... just draw the
> line in the sand."

### Slice 1b — `:rust::sqlite::ReadHandle` (in `wat-sqlite`, generic)

Read-only sibling of `:rust::sqlite::Db`. Opens with
`SQLITE_OPEN_READ_ONLY`; capability-honest by construction (the
type system enforces no `execute` / `pragma` / `begin` calls on
a reader). Lives in the **generic** `wat-sqlite` crate, not
`wat-telemetry-sqlite` — telemetry is one consumer of the
generic primitive. Per user correction mid-slice:
> "generic read ops need to be wat-sqlite — telemetry is a user
> of the generic tooling."

`(:wat::sqlite::open-readonly path) -> :wat::sqlite::ReadHandle`
plus a `path()` accessor for thread-crossing (cursors re-open
inside the producer thread from the captured path; thread-owned
cells can't follow the spawn boundary).

### Slice 1c — cursor types + producer threads + per-row reify

`:rust::telemetry::sqlite::LogCursor` /
`:rust::telemetry::sqlite::MetricCursor` opaque types. Each wraps
a Rust producer thread that owns the rusqlite trinity (Connection
+ Statement + Rows) on its stack and ships reified Event variants
through a bounded(1) crossbeam channel. `step!` shims pull one
event per call; `:None` signals exhaustion (channel disconnect).

Per-row reify decodes every column to its declared field type:
- `:i64` columns: direct typed reads.
- `:String` columns (uuid): direct reads.
- `:wat::edn::NoTag` columns (namespace, caller, level,
  metric_name, metric_value, metric_unit) → `read_holon_ast_natural`
  → wrap as `Value::Struct(":wat::edn::NoTag", [HolonAST])`.
- `:wat::edn::Tagged` columns (data) → `read_holon_ast_tagged`
  → wrap similarly.
- `:HashMap<HolonAST,HolonAST>` (tags) → tagless EDN map parse
  + per-entry HolonAST decode → `Value::wat__std__HashMap`.

### Slice 1d — wat surface: cursors + stream sources

`Reader.wat` ships `:wat::telemetry::sqlite::LogCursor` /
`MetricCursor` typealiases plus the substrate `step!` accessors.
Stream sources compose `:wat::std::stream::spawn-producer` around
the cursor: `(stream-logs handle constraints)` /
`(stream-metrics handle constraints)`. The producer lambda
re-opens the handle inside the spawn thread (per arc 053
thread_owned discipline), constructs a fresh cursor, drives a
tail-recursive `step!→send→repeat` loop until either side
disconnects.

Per user direction setting the architecture:
> "i want another circuit.. sql -> filter -> consumer / same
> tx,rx pairs lock stepping each other"

### Slice 1e — round-trip integration test + scope-bound TempFile/TempDir

`wat-tests/telemetry/reader.wat` writes 3 sample Event::Log
rows via the auto-spawn writer, joins the driver, opens with
ReadHandle, streams the rows back via spawn-producer + collect,
asserts count == 3. End-to-end CSP circuit lights up: 3 stages,
2 bounded(1) channels, drop-cascade. 122ms wall-clock for the
3-row round-trip including write + EDN encode / decode of all
NoTag/Tagged columns.

Substrate completion landed alongside: `:wat::io::TempFile` /
`:wat::io::TempDir` wrappers around Rust's `tempfile` crate.
Auto-delete on Drop — the file/dir unlinks when the wat
binding's Arc-count reaches zero. Per user direction:
> "we should just wrap whatever rust's temp file and temp dir
> are - they live for the duration of the scope of usage."

### Slice 2 — TimeConstraint enum + WHERE pushdown

Single shared `:wat::telemetry::TimeConstraint` enum
(`Since(Instant) | Until(Instant)`) consumed by both
stream-logs and stream-metrics. After slice-1a's "time only in
SQL" decision, log and metric constraint enums had identical
shape; one type beats two synonyms. Builders `(since instant)` /
`(until instant)` wrap the variant constructors.

Cursor signatures change to `(handle, Vec<TimeConstraint>)`.
Rust `parse_time_constraints` walks the vec, builds an
AND-joined WHERE fragment against the cursor's time column
(`time_ns` or `start_time_ns`), extracts each Instant → epoch
nanos, returns a `WhereClause { sql, params }`. Producer thread
formats SQL with the narrowing fragment, binds the i64 params
via rusqlite's `ToSql`.

Four reader tests verify narrowing actually works: empty vec =
full scan (3 rows), `Since(2000)` keeps {2000,3000} (2 rows),
`Until(1500)` keeps {1000} (1 row), `Since(1500) AND
Until(2500)` keeps only {2000} (1 row).

### Slice 3 — data-ast / data-value materialization helpers

Two wat-side defines on `Event::Log`:

- `data-ast` returns `Option<HolonAST>` — pattern-match on
  Log variant, unwrap the Tagged newtype's inner field via
  `:wat::edn::Tagged/0`, return `Some(HolonAST)`. `:None` on
  the Metric variant. Cheap — no eval, just extraction.
- `data-value<T>` returns `Option<T>` — same extract path,
  then runs the AST through `:wat::eval-ast!` to lift to a
  live Value of whatever type the log was. The lifted
  `Value::Struct` is what `:wat::form::matches?` (arc 098)
  accepts as subject — the pry/gdb UX the worked examples
  were designed around.

### Slice 4 — `examples/interrogate/` worked example

Self-contained binary running the full UX end-to-end: writes
6 sample Event::Log rows whose `data` column carries
`:demo::Trade` struct values, reopens read-only, runs two
queries (Q1 count = 6; Q2 `matches?` "buy ∧ qty > 10" = 2 hits
as designed). Output confirms the entire pipeline lights up:

```text
── Q1: warmup — count all logged trades ──
  total logs: 6

── Q2: matches? — buy AND qty > 10 ──
  hits: 2
── done ──
```

---

## Substrate completions surfaced alongside

Three substrate gaps surfaced during the arc and shipped as
separate commits / arcs:

1. **HolonAST EDN read** (slice 1c, in this arc's commits).
   Arc 091/092 shipped only the WRITE side
   (`holon_ast_to_edn`); the matching read was a TODO
   placeholder (`:wat::edn::read` errored on
   `#wat-edn.holon/*` tags). Slice 1c needed it for Tagged
   column reify, so it shipped here:
   `wat::edn_shim::read_holon_ast_tagged` /
   `read_holon_ast_natural` + `edn_to_holon_ast`. The
   `tagged_to_value` rejection turned into a routing call.

2. **`:wat::io::TempFile` / `:wat::io::TempDir`** (slice 1e).
   Auto-deleting wrappers around `tempfile::NamedTempFile` /
   `tempfile::TempDir`. Tests + ad-hoc scripts that need a
   fresh sqlite file or scratch buffer reach for these
   instead of composing string-concat over epoch-nanos.

3. **Arc 102** — `:wat::eval-ast!` polymorphic return. Surfaced
   mid-slice-3 design: the `data-value` lift to `Value::Struct`
   was blocked by arc 066's `value_to_holon` HolonAST-wrap that
   universal-carrier'd every eval result. Three-question
   discipline (simple / honest / good UX) pointed all three the
   other way, so arc 102 reverted the wrap by changing
   `eval-ast!`'s scheme to `Result<:T, :EvalError>` polymorphic
   — same trust-the-caller pattern `:wat::edn::read` already
   used. Five substrate unit tests + 3 call-site annotations
   migrated.

---

## Tests

cargo test --workspace green at every checkpoint commit:

- 9 reader.wat tests (3 round-trip variations, 4 narrowing,
  2 materialization)
- existing wat-telemetry-sqlite tests preserved (Sqlite +
  auto-spawn + edn-newtypes + hashmap-field paths)
- 737 substrate lib tests + every integration test
- `examples/interrogate` runs successfully via
  `cargo run -p interrogate-example`

---

## What's NOT in this arc

- **Arc 093 BACKLOG slice 5** — this INSCRIPTION + 058 changelog
  row + (per the BACKLOG draft) a CIRCUIT.md update. The
  CIRCUIT.md update is deferred until a real consumer needs
  the diagram redrawn — the existing one's pre-arc-093
  shape is still load-bearing for arc 091 / 095 explanations.
- **Cross-database scripts.** Per DESIGN §5: "rejected." Single
  .db per script — the user's shape.
- **Joining metric ↔ log via uuid.** Per DESIGN §8: "rejected."
  Wat-side `(filter ... matches?)` over a single stream is the
  surface; if a real consumer wants joins, that's a future arc.
- **`run-with` primitive.** Per DESIGN §7: rejected; the
  existing `:wat::std::stream::filter` IS the surface.
- **High-cardinality SQL pushdown** (uuid / metric-name
  / level / caller). Per slice 1a: the matcher does the
  filtering post-narrowing.

---

## Lessons

1. **CSP power-house works exactly as designed.** Three stages,
   two bounded(1) channels, drop-cascade. The substrate's
   existing stream-stdlib pattern fit arc 093's reader without
   any new threading primitives. The wat-side
   `spawn-producer` lambda + Rust step shim composition is the
   honest shape — Rust thread owns the SQLite trinity on its
   stack; wat thread does step → send forwarding; consumer
   thread does whatever. Same pattern every other Stream<T>
   consumer uses.

2. **Multiple revisions of the index set are healthier than
   committing to the wrong shape.** Slice 1a started with 7
   indexes (per DESIGN §6), revised to 4 mid-slice when the
   user pulled high-cardinality columns out, revised again to
   2 when "everything except time goes through the matcher"
   landed. Each revision was honest about what the previous
   version got wrong. `DESIGN.md §6` records the progression
   so future readers see the path, not just the destination.

3. **Substrate gaps surface in implementation, not in design.**
   Three substrate completions (HolonAST EDN read, TempFile/
   TempDir, eval-ast! polymorphic) all shipped because slice
   work hit them. The DESIGN didn't anticipate them; couldn't
   have, without the implementation forcing the question. *The
   shape we want is the shape we build; analysis-only design
   misses the gaps that real wiring exposes.*

4. **Three-question discipline catches half-built primitives.**
   Arc 102's reversal of arc 066 was the slice 3 design's
   forcing function. *"What is simple? What is honest? What is
   a good UX?"* — applied to arc 066's wrap, all three
   pointed away from it. The discipline doesn't tell you the
   answer; it surfaces when you're avoiding one. Reversal arcs
   are part of how the substrate is honest about its own
   history.

5. **Thread_owned discipline + path-stash for cross-thread.**
   The cursor's Rust producer thread can't share a
   `ThreadOwnedCell<ReadHandle>` with the consumer's thread.
   Solution: ReadHandle stashes the path string at construction;
   the spawn-producer lambda captures the path (a String, freely
   transferable) and re-opens a fresh handle inside its own
   thread. SQLite handles many concurrent read connections;
   the second open is cheap. *When the constraint is "this
   value can't cross thread boundaries," the workaround is
   often "transfer enough state to recreate it on the other
   side, not the value itself."*

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "the first real caller is us debugging the program. we have a
> time indexed edn store - i want to do filter evaluations on
> this time indexed data..."

> "we'll do some run and then we'll interrogate the system -
> we'll build up script templates and treat wat like a ruby of
> sorts"

> "i want another circuit.. sql -> filter -> consumer / same
> tx,rx pairs lock stepping each other"

> "we are a CSP power house - we are exceptionally good at it"

> "i think we just do time filter in sqlite and do all other
> content filtering in our stream... just draw the line in the
> sand"

> "we should just wrap whatever rust's temp file and temp dir
> are - they live for the duration of the scope of usage"

The arc closed when slice 4's worked example printed the
expected output and `cargo test --workspace` came back green for
the final time. The substrate is what the user said it should
be when he named it.

**PERSEVERARE.**
