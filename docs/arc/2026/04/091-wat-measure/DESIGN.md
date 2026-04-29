# Arc 091 — `:wat::telemetry::*` — DESIGN

**Status:** slices 1–4 shipped 2026-04-29; slices 5–6 pending.

> **Namespace + crate rename note (2026-04-29).** This arc was originally
> scoped under `:wat::measure::*` in a `wat-measure` crate. Mid-arc the
> /gaze ward folded measure into the broader telemetry concern: the crate
> is **`wat-telemetry`** at `crates/wat-telemetry/` and the namespace is
> **`:wat::telemetry::*`**. The rename landed via arc 096 ("telemetry
> crate consolidation"). Every reference below has been updated; older
> commits / inscriptions still use the `:wat::measure::*` form.

The substrate gains a sibling crate `wat-rs/crates/wat-telemetry/` claiming
the namespace `:wat::telemetry::*`. The crate ships **WorkUnit** +
**WorkUnitLog** — the in-memory measurement and structured-log primitives
the lab (and any future consumer) uses to instrument blocking work.

The companion arc 093 ships **WorkQuery** — the wat-side reader that pulls back
the data measured here and lets wat scripts interrogate it. Together they
close the loop: measure → store → query, all in wat, all in HolonAST shape.

(Arc 092, originally reserved for WorkQuery, was taken on 2026-04-29 by
`wat-edn` v4 minting — the small Rust-side prerequisite that lets
wat-telemetry mint UUIDs through wat-edn instead of taking its own
`uuid` pin.)

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

A wat function that wants measurement opens a scope. The body is a
**1-arity lambda taking the freshly-constructed `wu`** — that's how
the body does work on the WorkUnit. The shipped form is
**`WorkUnit/make-scope` — a closure factory.** Capturing the handle
and namespace ONCE so they "vanish" from the per-call surface:

```scheme
;; (make-scope handle namespace) — captured once at producer init.
;; Returned scope-fn takes (tags, body) per call; auto-ships at close.
((scope :wat::telemetry::WorkUnit::Scope<T>)
 (:wat::telemetry::WorkUnit/make-scope handle namespace))

(scope tags (lambda ((wu :wat::telemetry::WorkUnit) -> :T)
  ;; INSIDE the scope-fn body (body has wu):
  ;;   - wu has a fresh uuid + start time + the call-site's tags + namespace
  ;;   - bump counters: (incr! wu (:wat::holon::Atom :requests))
  ;;   - append durations: (append-dt! wu :sql-page secs)  ; slice 4 surface
  ;;   - emit logs (slice 5): (info wu data) — uuid auto-stamped on Event::Log
  ;;
  ;; AT scope-end (substrate owns this):
  ;;   - end-time + duration computed
  ;;   - counters + duration maps folded into Vec<:wat::telemetry::Event> rows
  ;;     (one Event::Metric per counter; one per duration sample —
  ;;     CloudWatch model)
  ;;   - rows shipped via the captured handle (Service::batch-log + ack)
  ;;   - body's return value passed through
  ...))
```

**Why a factory and not a bare `WorkUnit/scope`.** The user's direction
2026-04-29: "we want our deps to vanish as fast as possible." Tags
vary per scope-call (the dynamic context); namespace is fixed per
producer (the producing fn's identity); handle is wired once at
producer init. The factory captures the two stable pieces; per-call
surface is just `(scope tags body)`.

**No row-builder seam.** The substrate owns the row shape end-to-end:
it defines `:wat::telemetry::Event` (Metric + Log variants — flat
fields per the auto-dispatch arc 085 contract). The handle is a
`Service::Handle<Event> = (ReqTx<Event>, AckRx)` paired-channel tuple
(arc 095). The consumer's sink (e.g. lab's `Sqlite/auto-spawn` over
`:wat::telemetry::Event`) consumes the substrate type directly; no
consumer-supplied callback to map from "internal measurement state"
to "consumer's E type" because the substrate's E IS the type.

Mutation is in-place via `ThreadOwnedCell` (the `#[wat_dispatch(scope =
"thread_owned")]` macro wraps the struct). Same wat-native pattern
wat-lru's `LocalCache` uses for thread-owned mutable state — Tier 2 of
ZERO-MUTEX.md.

The shipped interior (slice 3 + slice 4 namespace addition):

```rust
pub struct WatMeasureWorkUnit {
  // Mutable — bumped via incr! / append-dt!
  counters:  HashMap<String, (Value, i64)>,        // canonical-key → (orig, count)
  durations: HashMap<String, (Value, Vec<f64>)>,   // canonical-key → (orig, samples)

  // Immutable for the scope's lifetime — declared at WorkUnit::new(namespace, tags).
  // tags:      the queryable HolonAST→HolonAST map ridden on every emitted row.
  // namespace: the producing fn's fqdn keyword, fixed per call site.
  tags:      Value,    // Value::wat__std__HashMap
  namespace: Value,    // Value::holon__HolonAST

  // Captured at construction for downstream Event::Metric start/end columns.
  started_epoch_nanos: i64,    // SystemTime::now() at new()
  uuid: String,                // canonical 8-4-4-4-12 hex (wat_edn::new_uuid_v4)
}
```

Counters and durations are keyed by HolonAST (canonicalized to a stable
String at insert time, per arc 057's hashmap_key contract). A keyword
`:requests` is a HolonAST. A list-form `(:broker eval-position)` is a
HolonAST. At ship-time each key is rendered via `:wat::edn::write-notag`
to TEXT for the metric_name column.

**Why namespace adjacent to tags at construction.** Both are immutable
for the scope's lifetime. Tags are declared upfront (the user's
direction: "tags must be cleared on creation; you cannot do
assoc/disassoc — they must be declared upfront so they are on all log
lines"). Namespace IS the wu's identity (the producing fn), not a
queryable tag. Per the user's 2026-04-29 framing, both belong on
`WorkUnit::new(namespace, tags)`.

### The substrate-owned Event types

The shape that ships through the consumer's `Service<E,_>`. The
substrate defines all variants; consumers don't model their own
measurement-event variants.

```scheme
;; Tags shape — typealiased once in wat/telemetry/types.wat so
;; every WorkUnit / Event field that mentions it reads cleanly:
(:wat::core::typealias :wat::telemetry::Tag
  :(wat::holon::HolonAST,wat::holon::HolonAST))
(:wat::core::typealias :wat::telemetry::Tags
  :HashMap<wat::holon::HolonAST,wat::holon::HolonAST>)

;; The Event enum's variants carry flat-field payloads — no nested
;; struct — because the substrate's auto-dispatch shim (arc 085)
;; supports only primitive + :wat::edn::Tagged/NoTag field types
;; per variant, NOT struct-typed fields. Each variant's fields are
;; the columns of its derived table; no second level of unwrapping.
;;
;; The CloudWatch shape: ONE Event::Metric row per data point.
;; A counter that ends at 7 emits ONE row (metric-value = leaf 7).
;; A duration sampled N times emits N rows (one per sample).
;; metric-value is uniformly a primitive HolonAST leaf — never a
;; collection — so NoTag rendering stays clean (bare numbers,
;; no `#wat-edn.holon/Bundle` prefix from operator tags surviving
;; NoTag's struct-and-enum-only tag-stripping rule, per arc 086).
;; Aggregation (SUM/AVG/PERCENTILE) lives in arc 093's WorkQuery —
;; the same shape CloudWatch + Prometheus use.
(:wat::core::enum :wat::telemetry::Event
  (Metric
    (start-time-ns :i64)                 ; wu start (wall-clock epoch ns)
    (end-time-ns   :i64)                 ; wu end
    (namespace     :wat::edn::NoTag)     ; producing fn's fqdn keyword
    (uuid          :String)              ; from the WorkUnit
    (tags          :wat::telemetry::Tags); HolonAST → HolonAST map
    (metric-name   :wat::edn::NoTag)     ; the counter/duration key
    (metric-value  :wat::edn::NoTag)     ; primitive HolonAST leaf — never a Bundle
    (metric-unit   :wat::edn::NoTag))    ; :count, :seconds, etc.
  (Log
    (time-ns   :i64)                       ; emit moment (wall-clock epoch ns)
    (namespace :wat::edn::NoTag)           ; producing fn's fqdn keyword
    (caller    :wat::edn::NoTag)           ; producer identity
    (level     :wat::edn::NoTag)           ; :info/:warn/:error/:debug
    (uuid      :String)                    ; from the WorkUnit
    (tags      :wat::telemetry::Tags)      ; same map, attached to every log line
    (data      :wat::edn::Tagged)))        ; round-trip-safe message HolonAST
```

The `tags` field replaces the original DESIGN's `dimensions` name
on Metric — same concept, with the user's framing making the third
concern explicit: tags ride alongside counters and durations as
WorkUnit state, not just as a per-row payload. Logs gain tags too: a
log emitted mid-scope inherits the wu's immutable tag map.

The `data` field on `Log` is `Tagged` because logs are queryable
structured records — we need to read them back as HolonAST and
pattern-match on them. NoTag would lose struct/enum identity. The
indexed fields (namespace, metric-name) are NoTag so SQL queries
match the natural form. Per-field choice via the type, no implicit
conventions (the `Tagged`/`NoTag` discipline slice 1 shipped).

A consumer's sink instantiates `Service<:wat::telemetry::Event, _>`.
The lab's `Sqlite/auto-spawn` over this enum derives a two-table
schema (per arc 085's auto-dispatch): the `Metric` variant lands in
a `metric` table, the `Log` variant in a `log` table. Cross-variant
joins via `uuid` work directly.

A producer that wants common tags on every emitted row builds the
tag map once at scope entry — the wu carries it for the lifetime of
the scope:

```scheme
;; Producer's common tags + per-call additions, declared at scope-call:
(scope (:wat::core::assoc common-tags
         (:wat::holon::Atom :stage) (:wat::holon::Atom :market-eval))
  (lambda ((wu :wat::telemetry::WorkUnit) -> :T)
    ...))
```

Tags are immutable for the scope's lifetime (no assoc-tag! /
disassoc-tag! — the user's invariant: every log line within one
scope MUST share the same tag set so rows correlate via a stable
queryable shape). Build the right map at scope-call; the substrate
attaches it to every Event the wu emits.

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
Slice 1 — substrate plumbing for HolonAST-as-TEXT binding   [SHIPPED 2026-04-29]
   wat-edn:    :wat::edn::Tagged + :wat::edn::NoTag newtypes
   wat-sqlite: auto-dispatch shim grows two match arms; tests round-trip
   Unblocks every consumer that wants to store HolonAST in sqlite.

Slice 2 — wat-telemetry crate scaffold + uuid::v4           [SHIPPED 2026-04-29]
   crates/wat-telemetry/ scaffolded per CONVENTIONS.md "publishable wat crate"
   (originally crates/wat-measure/; arc 096 folded measure into telemetry)
   Cargo.toml deps: wat (path), wat-macros (path),
                    wat-edn (path, features = ["mint"])  ; minting via arc 092
   :wat::telemetry::uuid::v4 -> :String   (canonical 8-4-4-4-12 hex; lives
     under the `:wat::telemetry::uuid::*` sub-namespace per `::` = free-fn
     convention; the `/` separator is reserved for type-method calls)
   wat_sources() + register() exports.
   Tests verify uniqueness across many calls.

Slice 3 — WorkUnit + data mutation primitives                [SHIPPED 2026-04-29]
   Rust shim: WatMeasureWorkUnit (ThreadOwnedCell via #[wat_dispatch])
     state: counters:  HashMap<String,(Value,i64)>,
            durations: HashMap<String,(Value,Vec<f64>)>,
            tags:                 Value,    // immutable Map (declared at new())
            namespace:            Value,    // immutable HolonAST (slice 4 add)
            started_epoch_nanos:  i64,
            uuid:                 String
   wat surface (in wat/telemetry/WorkUnit.wat):
     :wat::telemetry::WorkUnit  (typealias to :rust::telemetry::WorkUnit)
     :wat::telemetry::WorkUnit::new namespace tags        -> WorkUnit
     :wat::telemetry::WorkUnit/uuid                  wu   -> String
     :wat::telemetry::WorkUnit/namespace             wu   -> HolonAST
     :wat::telemetry::WorkUnit/tags                  wu   -> Tags
     :wat::telemetry::WorkUnit/started-epoch-nanos   wu   -> i64
     :wat::telemetry::WorkUnit/counter      wu name       -> i64
     :wat::telemetry::WorkUnit/durations    wu name       -> Vec<f64>
     :wat::telemetry::WorkUnit/counters-keys         wu   -> Vec<HolonAST>
     :wat::telemetry::WorkUnit/durations-keys        wu   -> Vec<HolonAST>
     :wat::telemetry::WorkUnit/incr!        wu name       -> ()
     :wat::telemetry::WorkUnit/append-dt!   wu name secs  -> ()

Slice 4 — Event types + make-scope closure factory + ship walker  [SHIPPED 2026-04-29]
   Substrate Event enum (in wat/telemetry/Event.wat):
     :wat::telemetry::Event
       (Metric start-time-ns end-time-ns namespace uuid tags
               metric-name metric-value metric-unit)
       (Log    time-ns namespace caller level uuid tags data)
     Flat-field per arc 085's auto-dispatch contract (no nested struct).
   Typealiases (in wat/telemetry/types.wat):
     :wat::telemetry::Tag           — :(HolonAST,HolonAST)
     :wat::telemetry::Tags          — :HashMap<HolonAST,HolonAST>
     :wat::telemetry::SinkHandles   — :Service::Handle<Event>
                                      ≡ :(ReqTx<Event>, AckRx)  per arc 095
     :wat::telemetry::WorkUnit::Body<T>   — :fn(WorkUnit) -> T
     :wat::telemetry::WorkUnit::Scope<T>  — :fn(Tags, Body<T>) -> T
   Ship-walker helpers + Event-builders:
     build-counter-metric / build-duration-metric         (one-row-per-data-point)
     collect-duration-events-for-name                     (fanout per sample)
     WorkUnit/scope::collect-metric-events                (full Vec<Event>)
   The closure factory:
     :wat::telemetry::WorkUnit/make-scope<T>
        (handle :SinkHandles) (namespace :HolonAST) -> :Scope<T>
     Returns a closure that takes (tags, body); opens a fresh wu;
     calls (body wu) — body has the wu and does its work; computes
     end-time at scope-close; walks counters-keys + durations-keys
     to build Vec<Event>; batch-log + ack via the captured handle;
     returns body's val.
   The HOF:
     :wat::telemetry::WorkUnit/timed<T>
        (wu :WorkUnit) (name :HolonAST) (body :fn()->T) -> :T
     Bumps `name`'s counter; runs body; appends `(end - start)/1e9`
     seconds to `name`'s duration list; returns body's val. Pure
     wat composition over incr! + epoch-nanos + append-dt!. Single-
     name discipline so the row count is predictable: N timed calls
     ⇒ 1 counter row + N duration rows under one metric-name.
   27 wat-side tests in wat-telemetry pass; full workspace cargo test green.

Slice 5 — Log emission primitives (the Log variant of Event)         [PENDING]
   :wat::telemetry::WorkUnit/info  wu data       ; emits Event::Log at :info
   :wat::telemetry::WorkUnit/warn  wu data
   :wat::telemetry::WorkUnit/error wu data
   :wat::telemetry::WorkUnit/debug wu data
   Each renders the substrate's Event::Log variant inline; ships
   through the same captured handle the make-scope factory holds.
   Open question: does the wu carry the handle internally so emit-sites
   don't repeat it? Likely yes — slice 4's make-scope closes over the
   handle; slice 5 should follow the same shape (the closure or the wu
   knows; emit-site signature stays just (wu, data)).
   tests verify uuid join with metrics from the same scope, level routing.

Slice 6 — lab refactor: consume substrate Event directly             [PENDING]
   The lab's :trading::log::LogEntry retires its Telemetry variant in
   favor of consuming :wat::telemetry::Event directly. The lab's
   Sqlite/auto-spawn instantiates Service<:wat::telemetry::Event,_>;
   the auto-dispatch shim (arc 085) derives the two-table schema from
   the substrate enum (one table per variant). No lab-side
   Log/Metric variants — the substrate IS the source of truth for
   measurement-event shape.
   pulse.wat / smoke.wat / bare-walk.wat: per-stage emit-sites migrate
   to make-scope (one scope per loop iteration; counters + durations
   + logs all attached to the wu).
   docs/CIRCUIT.md (lab) — update Logging section: rows go to the log
   table or metric table per Event variant; namespaces stay
   circuit.candle / circuit.market / etc.
   This slice closes when pulse runs and the run db has both populated
   tables with proper joinable uuids.
```

Slices ship sequentially. Each one tests its own piece; arc closes when slice 6's
pulse benchmark produces a queryable run db (the actual test of arc 093's reader
path comes in arc 093 itself).

## What's NOT in this arc

- **Arc 093 — `:wat::telemetry::WorkQuery`.** Reader side. Time-indexed queries;
  prolog-y unify; combinators; bidirectional join. Builds on the writer this arc
  ships.
- **Arc 094 — circuit.wat.** The N×M topology smoke test (per
  `holon-lab-trading/docs/CIRCUIT.md`). First production consumer of arc 091's
  WorkUnit.
- **SQLite EDN UDF.** The eventual upgrade path that lets `WHERE` clauses
  pattern-match EDN in SQL directly. Substantial substrate add; out of scope
  until scale demands.
- **Common-tags merge primitive.** With slice 4's tag-immutability rule
  (the wu's tag map is fixed at scope-call), the original "merge fixed
  tags into per-call data" pattern collapses to "build the right
  HashMap at the scope-call site." `:wat::core::assoc` (arc 020 over
  HashMap) suffices; no substrate `merge` needed. If a producer's
  common tags grow large enough to want a real `merge`, that lands as
  its own substrate slice — not part of this arc.

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

The /gaze ward then named the home: originally `:wat::measure::*` (sibling
crate); the HOF was renamed `WorkUnit/scope` (was `/measure` — reflexive);
`time<T>` was renamed `timed<T>` (collided with `:wat::time::*`). Arc 096
later folded measure into the broader `:wat::telemetry::*` concern. Slice 4
finally landed `WorkUnit/make-scope` (a closure factory) instead of a bare
`WorkUnit/scope` HOF — the user's "we want our deps to vanish as fast as
possible" direction; capturing handle + namespace once at producer init,
returning a (tags, body) -> T closure.

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
