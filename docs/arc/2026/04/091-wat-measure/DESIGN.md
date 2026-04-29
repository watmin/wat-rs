# Arc 091 — `:wat::measure::*` — DESIGN

**Status:** in design 2026-04-29.

The substrate gains a sibling crate `wat-rs/crates/wat-measure/` claiming the
namespace `:wat::measure::*`. The crate ships **WorkUnit** + **WorkUnitLog** —
the in-memory measurement and structured-log primitives the lab (and any future
consumer) uses to instrument blocking work.

The companion arc 093 ships **WorkQuery** — the wat-side reader that pulls back
the data measured here and lets wat scripts interrogate it. Together they
close the loop: measure → store → query, all in wat, all in HolonAST shape.

(Arc 092, originally reserved for WorkQuery, was taken on 2026-04-29 by
`wat-edn` v4 minting — the small Rust-side prerequisite that lets wat-measure
mint UUIDs through wat-edn instead of taking its own `uuid` pin.)

## What we know

### The CloudWatch-shaped foundation

The lab's telemetry has been a flat `LogEntry::Telemetry` row since arc 029.
That worked while every emit-site logged "the same kind of thing." Three
recognitions across arcs 089–091 split the shape:

1. **Logs and metrics are different events.** Logs carry structured messages at
   one timestamp. Metrics carry counters and durations bracketed by a work
   unit's lifecycle. Different schemas; different indexes; different query
   shapes.
2. **Both join via uuid.** Anything happening within one work unit shares its
   uuid. SQL joins from a metric to its logs (or vice versa) by time-windowing
   then filtering on uuid.
3. **Anything structured rides as HolonAST → EDN.** Any text we write that
   represents structured data must be HolonAST converted to EDN. Strings stay
   strings only when they're truly opaque (uuid hex, file paths). The rule
   surfaced from the recurring "this name should be queryable" question; the
   answer is "name everything in HolonAST so it always is."

The two tables that fall out:

```sql
CREATE TABLE log (
  time_ns    INTEGER NOT NULL,    -- INDEXED  (single emit time)
  namespace  TEXT    NOT NULL,    -- INDEXED  (notag EDN of fqdn keyword)
  caller     TEXT    NOT NULL,    -- notag EDN keyword
  level      TEXT    NOT NULL,    -- notag EDN keyword
  uuid       TEXT    NOT NULL,    -- uuid hex; not indexed; filter only
  data       TEXT    NOT NULL     -- TAGGED EDN; round-trip-safe; the message IS data
);
CREATE INDEX idx_log_time ON log(time_ns);
CREATE INDEX idx_log_ns   ON log(namespace);

CREATE TABLE metric (
  start_time_ns  INTEGER NOT NULL,   -- INDEXED  (wu start)
  end_time_ns    INTEGER NOT NULL,   -- INDEXED  (wu end)
  namespace      TEXT    NOT NULL,   -- INDEXED  (notag EDN of fqdn keyword)
  uuid           TEXT    NOT NULL,   -- uuid hex; not indexed
  dimensions     TEXT    NOT NULL,   -- notag EDN map literal
  metric_name    TEXT    NOT NULL,   -- notag EDN
  metric_value   TEXT    NOT NULL,   -- EDN literal — numbers bare; structured values quoted
  metric_unit    TEXT    NOT NULL    -- notag EDN
);
CREATE INDEX idx_metric_start ON metric(start_time_ns);
CREATE INDEX idx_metric_end   ON metric(end_time_ns);
CREATE INDEX idx_metric_ns    ON metric(namespace);
```

Indexes go where queries naturally narrow (time, namespace). Everything else
filters in wat after the indexed query culls. This is mini-CloudWatch on
SQLite — no scan-as-DynamoDB-anti-pattern; always indexed query first.

### The HolonAST-to-TEXT binding rule

The substrate's auto-dispatch shim (arc 085) currently binds wat values to
SQLite ToSql parameters via per-type match arms (`:String → TEXT`,
`:i64 → INTEGER`, `:f64 → REAL`, `:bool → INTEGER`). Storing HolonAST in TEXT
columns needs a new pair of match arms — one tagged, one notag.

The choice between tagged and notag is per-field, not per-call. The decision
lives at the enum-decl site, in the field type. Two newtypes around HolonAST:

```scheme
(:wat::core::newtype :wat::edn::Tagged :wat::holon::HolonAST)
(:wat::core::newtype :wat::edn::NoTag  :wat::holon::HolonAST)
```

Lives in `:wat::edn::*` because EDN-write-strategy is the actual concern; the
sqlite shim integrates with EDN's existing `:wat::edn::write` and
`:wat::edn::write-notag` family.

The rule, stated:

- Anything that represents structured data is HolonAST.
- HolonAST is converted to TEXT via either `:wat::edn::write` (round-trip-safe;
  field declared as `:wat::edn::Tagged`) or `:wat::edn::write-notag` (lossy,
  human-readable; field declared as `:wat::edn::NoTag`).
- A field's type tells the substrate which strategy. No magic, no implicit
  column-name convention.

### The WorkUnit shape

A wat function that wants measurement opens a scope:

```scheme
(:wat::measure::WorkUnit/scope handles (lambda ((wu :wat::measure::WorkUnit) -> :T)
  ;; INSIDE scope:
  ;;   - wu has a fresh uuid + start time
  ;;   - bump counters: (incr! wu :requests)
  ;;   - time blocking calls: (timed wu :sql-page (lambda () (some-io-work)))
  ;;     — bumps :sql-page counter, appends duration, returns the io-work's val
  ;;   - emit logs through wu: (info wu {:event :started})
  ;;     — uuid auto-stamped on the WorkUnitLog
  ;;
  ;; AT scope-end:
  ;;   - end-time + duration computed
  ;;   - counters and counter-durations folded into LogEntry::Metric rows
  ;;   - rows shipped via the lab's telemetry sink (Service/batch-log + ack)
  ;;   - body's return value passed through
  ...))
```

Mutation is in-place via ThreadOwnedCell wrapping the WorkUnit's interior maps.
Same wat-native pattern wat-lru's LocalCache uses for thread-owned mutable
state — Tier 2 of ZERO-MUTEX.md.

The interior:

```rust
struct WorkUnitState {
  uuid:       String,                       // hex
  started:    Instant,                       // wall-clock at scope open
  counters:   HashMap<HolonAST, i64>,       // bumps via incr! and timed
  durations:  HashMap<HolonAST, Vec<f64>>,  // appends via timed and append-dt!
}

pub struct WatMeasureWorkUnit {
  cell: ThreadOwnedCell<WorkUnitState>,
}
```

Counters and durations are keyed by HolonAST. A keyword `:requests` is a
HolonAST (per arc 057). A list-form `(:broker eval-position)` is a HolonAST. A
deeply structured form is a HolonAST. At ship-time each key is rendered via
`:wat::edn::write-notag` to TEXT for the metric_name column.

### Logs are first-class structured data

`:wat::measure::WorkUnitLog` is the second event shape:

```scheme
(:wat::core::struct :wat::measure::WorkUnitLog
  (time-ns   :i64)
  (namespace :wat::edn::NoTag)        ; producing fn's fqdn keyword
  (caller    :wat::edn::NoTag)        ; producer identity
  (level     :wat::edn::NoTag)        ; :info/:warn/:error/:debug
  (uuid      :String)                  ; from the WorkUnit
  (data      :wat::edn::Tagged))       ; round-trip-safe message HolonAST
```

The `data` field is tagged because logs are queryable structured records — we
need to read them back as HolonAST and pattern-match on them. Notag would lose
struct/enum identity.

A producer that wants common tags on every log emits via:

```scheme
;; The lab's common-tags pattern — merge fixed lab fields with the per-call data.
(:my::log/info logger wu (:wat::core::merge common-tags {:event :buy :price 100.5}))
```

`merge` is a HolonAST-map merge primitive — the lab adds fixed tags
(file/line/function name, etc.) to every log without ceremony.

## What we don't know

- **The query shape after arc 093.** WorkQuery's prolog-y pattern matching is
  designed but not built; it'll surface decisions arc 091 doesn't anticipate.
- **Whether `metric_value` as TEXT-EDN imposes meaningful query cost.** SQL
  numeric ranges need `CAST(metric_value AS REAL)`. SQLite handles this fine
  but it forecloses index use on metric_value (which we don't index anyway).
  If the cast cost surfaces as a hot path, we add a numeric column sidecar in
  a follow-up. Not blocking.
- **Eventual UDF for prolog-y queries.** SQLite extension that knows how to
  unify EDN against patterns directly would eliminate the wat-side fine-filter
  for huge result sets. Future arc; not blocking.

## Slices

```
Slice 1 — substrate plumbing for HolonAST-as-TEXT binding
   wat-edn:    :wat::edn::Tagged + :wat::edn::NoTag newtypes
   wat-sqlite: auto-dispatch shim grows two match arms; tests round-trip
   This unblocks every consumer that wants to store HolonAST in sqlite.

Slice 2 — wat-measure crate scaffold + uuid::v4
   crates/wat-measure/ scaffolded per CONVENTIONS.md "publishable wat crate"
   Cargo.toml deps: wat (path), wat-macros (path),
                    wat-edn (path, features = ["mint"])  ; minting via arc 092
   :wat::measure::uuid::v4 -> :String  (canonical hex like "550e8400-...";
     under the `:wat::measure::uuid::*` sub-namespace per `::` = free-fn
     convention; the `/` separator is reserved for type-method calls)
   wat_sources() + register() exports
   tests verify uniqueness across many calls

Slice 3 — WorkUnit + mutation primitives
   Rust shim: WatMeasureWorkUnit (ThreadOwnedCell-backed)
     state: counters: HashMap<HolonAST, i64>, durations: HashMap<HolonAST, Vec<f64>>,
            started: Instant, uuid: String
   #[wat_dispatch] generates the type registration + method shims
   wat surface (in wat/measure.wat):
     :wat::measure::WorkUnit (typealias to :rust::measure::WorkUnit)
     :wat::measure::WorkUnit/incr!     wu name
     :wat::measure::WorkUnit/append-dt! wu name secs
     :wat::measure::WorkUnit/timed<T>  wu name (lambda () body) -> T
   tests verify in-place mutation visible across calls in same scope

Slice 4 — WorkUnit/scope HOF + finalize-and-ship
   :wat::measure::WorkUnit/scope<T>
     (handles :wat::measure::SinkHandles)
     (body :fn(wat::measure::WorkUnit)->T)
     -> T
     [opens fresh wu; runs body; computes duration;
      walks counter+duration maps to build Vec<LogEntry::Metric> rows;
      batch-log + ack via Service<E,G> handles in `handles`;
      returns body's val]
   :wat::measure::SinkHandles — the bundled (req-tx, ack-tx, ack-rx) tuple typealias
                                so the body's type signature stays flat
   tests verify ship+ack lockstep + uuid present on every emitted row

Slice 5 — WorkUnitLog + emission primitives
   :wat::measure::WorkUnit/info  wu data       ; emits LogEntry::Log at :info level
   :wat::measure::WorkUnit/warn  wu data
   :wat::measure::WorkUnit/error wu data
   :wat::measure::WorkUnit/debug wu data
   Each renders the WorkUnitLog row inline; ships through the same Service handles
     captured in the wu (wu carries them so logs don't need extra params)
   tests verify uuid join with metrics from the same scope, level routing

Slice 6 — lab refactor
   :trading::log::LogEntry — retire Telemetry variant; introduce Log + Metric variants
     per the schema above.  Field types use :wat::edn::Tagged / :wat::edn::NoTag.
   pulse.wat / smoke.wat / bare-walk.wat: per-stage emit-sites migrate to
     WorkUnit/scope (one scope per loop iteration; counters + durations + logs
     all attached to the wu).
   docs/CIRCUIT.md (lab) — update Logging section: rows go to log table or metric
     table per variant; namespaces are still circuit.candle / circuit.market / etc.
   This slice closes when pulse runs and the run db has both populated tables
     with proper joinable uuids.
```

Slices ship sequentially. Each one tests its own piece; arc closes when slice 6's
pulse benchmark produces a queryable run db (the actual test of arc 093's reader
path comes in arc 093 itself).

## What's NOT in this arc

- **Arc 093 — `:wat::measure::WorkQuery`.** Reader side. Time-indexed queries;
  prolog-y unify; combinators; bidirectional join. Builds on the writer this arc
  ships.
- **Arc 094 — circuit.wat.** The N×M topology smoke test (per
  `holon-lab-trading/docs/CIRCUIT.md`). First production consumer of arc 091's
  WorkUnit.
- **SQLite EDN UDF.** The eventual upgrade path that lets `WHERE` clauses
  pattern-match EDN in SQL directly. Substantial substrate add; out of scope
  until scale demands.
- **Common-tags merge primitive.** The lab will need a small `:wat::core::merge`
  for HolonAST maps — handles the `(merge common-tags {:event ...})` pattern.
  If the substrate doesn't already have it (it doesn't — only HashMap has
  assoc), it ships in arc 091 slice 5 OR a tiny substrate slice. Likely the
  former; small enough not to fork an arc for.

## How sub-arcs / slices ship

Per the established pattern (arc 089 etc):
- Each slice gets implemented + tested + committed
- INSCRIPTION.md captures what shipped at arc close
- Tasks track slice progress

## Surfaced by

User direction 2026-04-29:

> "we need to be able to do work on our data we generate -- we'll use wat to
> query it - not sqlite3 ... we need a utility suite for doing work on the
> metrics and logs.... these are provided with WorkUnit and co.... these are
> WorkQuery..."

The /gaze ward then named the home: `:wat::measure::*` — sibling crate; the
HOF was renamed `WorkUnit/scope` (was `/measure` — reflexive); `time<T>` was
renamed `timed<T>` (collided with `:wat::time::*`).

User direction on the EDN rule:

> "i think the rule is... we strive to communicate any text as HolonAST...
> always... any exception must be justified strongly... some datastore can't
> handle EDN is basically the only justification..."

User direction on the indexed-query model:

> "its never a scan in the terms of dynamodb or whatever.. /its always/ an
> indexed query on time indexes.. uuid and tags are high cardinality - never
> indexed.. just filter over the indexed results..."

User direction on the explicit type marking for serialization:

> "if we are making a choice - we do it explicitly"
