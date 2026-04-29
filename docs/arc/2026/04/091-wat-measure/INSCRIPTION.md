# Arc 091 — `:wat::telemetry::*` — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate gained `wat-telemetry` (and sibling `wat-telemetry-sqlite`)
— first-class measurement primitives the trading lab (and any future
consumer) uses to instrument blocking work. Across eight slices, we
moved the lab from a hand-rolled `:trading::log::LogEntry` enum to
the substrate-defined `:wat::telemetry::Event` and proved the wires
end-to-end through SQLite.

**Predecessors:**
- Arc 029 — `:trading::rundb::Service` (the lab's first telemetry
  CSP wrapper; arc 091 retired it via the substrate `Service<E,G>`).
- Arc 080 — `:wat::telemetry::Service<E,G>` (queue-fronted shell
  this arc parameterizes over Event).
- Arc 085 — Sqlite/auto-spawn schema derivation (this arc's
  Tagged/NoTag/HashMap arms extend it to richer field types).
- Arc 087 — ConsoleLogger pattern (closure-over-(handle, caller,
  now-fn); WorkUnitLog mirrors it).
- Arc 095 — Service paired channels (Handle = (ReqTx, AckRx); the
  lab's pre-arc-095 ReqTxPool + ack-channel-per-call construction
  retired here).
- Arc 096 — Telemetry crate consolidation (rename `wat-measure` →
  `wat-telemetry`, `:wat::measure::*` → `:wat::telemetry::*`).

**Surfaced by:** user direction 2026-04-29:

> "we need to be able to do work on our data we generate -- we'll
> use wat to query it - not sqlite3 ... we need a utility suite for
> doing work on the metrics and logs.... these are provided with
> WorkUnit and co...."

> "all of the custom logging for trading lib can be implemented on
> :wat::telemetry::Event - we just killed it off - the
> :wat::telemetry::Event handles logs and metrics - we use those"

---

## What shipped

### Slices 1–5 (substrate primitives)

**Slice 1** — Tagged/NoTag newtypes + sqlite auto-dispatch arms.
`:wat::edn::Tagged` (round-trip-safe) and `:wat::edn::NoTag`
(lossy/natural) wrap a HolonAST; the auto-dispatch shim recognizes
each and binds via the matching write strategy.

**Slice 2** — `wat-telemetry` crate scaffold + `:wat::telemetry::uuid::v4`.
The crate registers under the substrate's deps mechanism; uuid
generation flows through `wat-edn`'s `mint` feature (arc 092).

**Slice 3** — `WorkUnit` + mutation primitives. Thread-owned cell
holding counters (`HashMap<HolonAST, i64>`) and durations
(`HashMap<HolonAST, Vec<f64>>`); mutations via `incr!` /
`append-dt!`; namespace and tags declared at construction
(immutable for the scope's lifetime). Started-epoch-nanos captured
at construction so `start-time-ns` columns work.

**Slice 4** — Event types + WorkUnit/make-scope HOF. Substrate
defines `Event::Metric` (8 fields) and `Event::Log` (7 fields) flat-
field per arc 085's auto-dispatch contract. `WorkUnit/make-scope`
is a closure factory — `(make-scope handle namespace) → fn(tags,
body) → T` — capturing the stable bits once and exposing a clean
per-call surface. Plus `WorkUnit/timed<T>` for IO/expensive-call
durations. The "ship walker" inside make-scope folds counters and
durations into Vec<Event::Metric> at scope-close (CloudWatch model:
one row per data point).

**Slice 5** — `WorkUnitLog` + Log emission primitives. Closure-
over-(handle, caller, now-fn) mirroring arc 087's ConsoleLogger.
`/log` (universal) + `/debug` `/info` `/warn` `/error` (sugar with
level baked). Sync-per-event (one Service/batch-log round-trip
per emission); `data` parameter takes :wat::WatAST so producers can
pass quoted/quasiquoted forms (or struct->form lifts — slice 8) and
the substrate structurally lowers via `Atom`'s WatAST arm.

### Slice 6 (lab refactor)

Lab consumes `:wat::telemetry::Event` directly. Retired
`:trading::log::LogEntry` entirely — `wat/io/log/{LogEntry,
telemetry,rate-gate}.wat` deleted. Migrated:

- `wat/cache/reporter.wat` — cache stats observations as Event::Log
- `wat/sim/encoding-cache.wat` — snapshot Log row carrying 5 stats
  in a struct
- `wat/services/treasury.wat` — make-scope per Tick / per
  broker-request; `timed` wraps blocking work; per-Tick state
  observation as Event::Log via WorkUnitLog
- `wat/programs/{pulse,smoke,bare-walk}.wat` — showcase shapes
- `wat-tests-integ/proof/{002,003,004}/*.wat` — outcome rows via
  WorkUnitLog

`io/telemetry/` moved out to `wat/telemetry/` per the user's
direction "drop io from the name." A shared `:trading::PaperResolved`
struct extracted to `wat/types/paper-resolved.wat`.

### Slice 7 (lab-surfaced substrate gaps)

Slice 6's lab integration uncovered three real flaws the substrate's
slice-1 through slice-5 tests didn't catch (those tests stub
the dispatcher or use primitive data; only the lab exercises the
full Sqlite/auto-spawn path with rich payloads):

1. **HashMap auto-dispatch arm.** `Event.tags :wat::telemetry::Tags`
   (HashMap<HolonAST,HolonAST>) couldn't bind. Added `type_to_affinity`
   + `value_to_tosql` arms recognizing HashMap fields; renders via
   `:wat::edn::write-notag` and binds as TEXT. `derive_schema` now
   `expand_alias()`s field types first so `:wat::telemetry::Tags`
   resolves to its parametric form.
2. **NoTag double-colon bug.** `holon_ast_to_edn_notag` was
   `format!(":{}", s)`-prefixing keywords already carrying their
   colon — `:asset` rendered as `::asset`. Fixed by passing the
   symbol through directly.
3. **EDN map separator waste.** Switched `, ` to a single space —
   commas are whitespace per EDN spec; cleaner canonical form.
4. **WorkUnitLog/log<E>: data param HolonAST → WatAST.** Atom's
   polymorphism (arc 057) covers primitives + HolonAST + WatAST but
   NOT Struct. Producers couldn't lift a struct value through Atom;
   now they pass a quoted/quasiquoted form and the substrate lowers
   via `watast_to_holon`.

### Slice 8 (substrate ergonomics)

The user asked: "is there a func that does the quoting for us
without us having to do `,some-bare-symbol`?" Two new substrate
primitives:

1. `:wat::core::quasiquote` — runtime version of the same form
   macros use as their template. Walks a template; at each unquote
   site evaluates the inner expression and converts the result to a
   literal WatAST node; returns `Value::wat__WatAST(walked)`. Same
   depth-tracking discipline as macros.rs's expand-time walker
   (arc 029).

2. `:wat::core::struct->form value` — lift a struct VALUE to its
   constructor-call FORM. Reads `Value::Struct.type_name` and
   field values; builds `WatAST::List(:type-name/new field0 field1
   ...)`. Inverse of struct construction; round-trips through
   `eval-ast!`. Lab use sites read:

   ```scheme
   (/info wlog wu (:wat::core::struct->form pr))
   ```

   Per-shape ceremony drops to one line. The user composes the
   struct value as they would for any in-memory use; substrate
   lifts it to the constructor form for round-trip-safe Tagged
   storage.

### Acceptance criterion (slice 6 design)

> "This slice closes when pulse runs and the run db has both
>  populated tables with proper joinable uuids."

Verified end-to-end:

```bash
$ cargo run --release
$ DB=$(ls -t runs/pulse-*.db | head -1)
$ sqlite3 "$DB" '.tables'
log     metric

$ sqlite3 "$DB" 'SELECT m.metric_name, m.metric_value, l.data
                   FROM metric m JOIN log l ON m.uuid = l.uuid;'
:candle|1000|#wat-edn.holon/Bundle [#wat-edn.holon/Symbol ":trading::pulse::RunSummary/new" ...]
```

Both tables populated; `metric.uuid = log.uuid` for rows from the
same scope.

---

## Tests

- `crates/wat-edn/src/{lexer,parser,writer}.rs` — float-fixture
  literals updated; `2.5` instead of `3.14` (clippy
  `approx_constant`).
- `crates/wat-edn/src/writer.rs` — map writer test asserts space-
  only separator (`{:a 1 :b 2}` not `{:a 1, :b 2}`).
- `crates/wat-telemetry/wat-tests/telemetry/{WorkUnit,WorkUnitLog,
  Service,Console,uuid}.wat` — 30+ wat-tests across the surface.
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/{Sqlite,
  edn-newtypes,auto-spawn,hashmap-field}.wat` — auto-spawn round-
  trips per field-type arm.
- `wat-tests/std/struct-to-form.wat` — slice 8 primitives.

`cargo test --workspace` green; `cargo clippy --workspace
--all-targets` clean.

---

## What's NOT in this arc

- **Arc 093 — `:wat::telemetry::WorkQuery`.** Reader side. Time-
  indexed queries; prolog-y unify; combinators; bidirectional
  join. Builds on the writer this arc shipped.
- **Arc 094 — circuit.wat.** The N×M topology smoke test. First
  production consumer of arc 091's WorkUnit beyond pulse/smoke.
- **SQLite EDN UDF.** The eventual upgrade path that lets `WHERE`
  clauses pattern-match EDN in SQL directly. Substantial substrate
  add; out of scope until scale demands.
- **Auto-magic caller detection.** Today producers pass `:caller`
  explicitly to WorkUnitLog/new; a future arc could surface the
  current `module_path!()` equivalent so producers don't have to.
  Verbose-is-honest stays the default.

---

## Lessons

1. **Test-stub blind spots.** Slice 4 + 5 tests used a stub
   dispatcher (forwarding events to a queue). The full Sqlite
   write path stayed unverified until slice 6 — at which point
   THREE substrate gaps surfaced (HashMap binder, NoTag double-
   colon, struct lift). Fixing them inline as slice 7 + 8 was the
   right move; preventing them earlier means writing a real-sqlite
   integration test before a consumer arrives.

2. **Lab aliases over substrate generic types didn't expand
   transitively for foreign callers.** Briefly attempted lab-side
   `:trading::telemetry::Handle` etc.; they didn't unify with the
   underlying tuple/HandlePool shapes when used outside the
   substrate crate. Retired in favor of substrate-direct types
   (`:wat::telemetry::Service::Handle<wat::telemetry::Event>`).
   The substrate's own typealiases work; a foreign nullary alias-
   over-generic doesn't transitively expand. Real but didn't block
   slice closure.

3. **Scope vs scope-discipline.** `WorkUnit/make-scope` returns a
   reusable closure (handle + namespace captured); the lab's tests
   runner deps list (`tests/test.rs`, `tests/proof/proof_*.rs`)
   needed `wat_telemetry` + `wat_telemetry_sqlite` for the
   substrate's typealiases to be visible. Caught this only when
   the user said "make sure it's not a scoping in your test forms"
   — a direct hit.

4. **The Metric vs Log discipline.** Counter (per-event bump) and
   duration (per-blocking-call timing) are emergent from
   instrumentation — the wu's accumulated state at scope-close.
   Anything else (snapshot value, structured observation) is a
   Log line. The archive's `emit_metric ns id dims ts "deposits"
   10000.0 "Count"` pattern was conflating shapes; the substrate's
   discipline forces the distinction. The user's framing —
   "deposits 10000.0 looks like a count value in cloudwatch terms"
   — sharpened the rule: CloudWatch Count is a unit, not a metric
   type; the right home for snapshot values is the Log table's
   `data` column, where SQL filter-and-parse fits the access pattern.

---

## Surfaced by

User direction 2026-04-29 (full arc):

> "ok - 091 needs what exactly?.... update the code?..."
> ... (multi-day arc spanning slices 1–8) ...
> "we just killed it off - the :wat::telemetry::Event handles logs
> and metrics - we use those"
> "we found a flaw - fix it" (slice 7)
> "is there a func we can write who does the quoting for us
> without us having to do ,some-bare-symbol?" (slice 8)
> "let's get the docs updates - yes - please"

The arc closed when pulse ran end-to-end with metric + log tables
populated and joinable via uuid.
