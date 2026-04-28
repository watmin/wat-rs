# Arc 080 — Promote rundb Service to substrate as `:wat::std::telemetry::*`

**Status:** PROPOSED 2026-04-29. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 029 — `:trading::rundb::Service` shipped lab-side. CSP wrapper over sqlite RunDb. Per-variant dispatcher.
- Arc 078 — service contract codified (Reporter + MetricsCadence + null-helpers + typed Report enum). Two substrate cache services follow it; rundb does not yet.
- In-progress (not committed as own arc): rundb retrofit added Stats + MetricsCadence threading + tick-window for self-heartbeat. That work absorbs into this arc as the substrate-shape canon.

**Surfaced by:** The user's recognition (2026-04-29):

> "RunDbService - is a bad name... we need something generic... it holds records and logs and records as metrics.. its modeled to be something like a cloudwatch... telemetry service?"

Followed by a load-bearing correction (2026-04-29):

> "the LogEntry /must/ be user defined - we do not provide anything here.. or maybe we provide extremely basic things to educate the reader on how to implement their own bespoke entries with whatever complexity"

The substrate ships the SHELL (queue, driver, cadence, stats), not the entry shape. Each consumer defines its own entry enum with whatever variants make sense for its domain. Substrate is generic over E.

---

## What this arc is, and is not

**Is:**
- A generic Service shell at `:wat::std::telemetry::Sqlite<E,G>` — queue-fronted, batch-accepting, parameterized by entry type `E` and cadence gate type `G`.
- Caller-supplied dispatcher fn `(:wat::sqlite::Db E) -> :()` — substrate doesn't know how to write any specific E to sqlite; the consumer brings the writer.
- Caller-supplied stats-translator fn `:Stats -> :Vec<E>` — substrate's self-heartbeat builds its own metric entries by asking the consumer how to encode `Stats` as E. Substrate dispatches the result through the same dispatcher.
- Stats + MetricsCadence + null-helpers + tick-window — same shape as arc 078's cache services.
- Educational example at `wat-tests/std/telemetry/` showing one tiny entry enum + dispatcher + stats-translator. Teaches the pattern; not load-bearing.

**Is not:**
- A canonical TelemetryEntry shape. Substrate ships zero variants — the consumer defines every entry.
- A clock-injecting entry-maker factory. That's a CONSUMER pattern; substrate documents it but doesn't ship a generic factory (each consumer's entries have different constructor shapes; one-size-fits-all factory doesn't fit).
- Console-flavored output. Arc 081.
- Lab call-site sweep. Lab proposal 059-002.
- A schema engine. The consumer's dispatcher manages its own DDL.

---

## Surface

### Stats + MetricsCadence (substrate-defined)

```scheme
;; The Service's own internal counters. Substrate-fixed shape; users
;; cannot extend this. The translator below converts these to whatever
;; entry shape the consumer wants on cadence-fire.
(:wat::core::struct :wat::std::telemetry::Sqlite::Stats
  (batches :i64)
  (entries :i64)
  (max-batch-size :i64))

;; Same MetricsCadence pattern as arc 078.
(:wat::core::struct :wat::std::telemetry::Sqlite::MetricsCadence<G>
  (gate :G)
  (tick :fn(G,Sqlite::Stats)->(G,bool)))

(:wat::core::define
  (:wat::std::telemetry::Sqlite/null-metrics-cadence
    -> :Sqlite::MetricsCadence<()>) ...)
```

### Service shell (substrate-defined, generic over E)

```scheme
;; The driver-thread function. Generic over E (the consumer's entry
;; type) and G (cadence gate). Caller supplies:
;;   - dispatcher  : how to write one E to the Db
;;   - translator  : how to encode service-Stats as Vec<E>
;;
;; The shell knows nothing about E's structure. It just queues
;; batches of E, dispatches each, and on cadence-fire builds
;; (Vec<E>) via translator and dispatches those too.
(:wat::std::telemetry::Sqlite<E,G>
  (path :String)
  (count :i64)
  (dispatcher :fn(:wat::sqlite::Db E) -> :())
  (stats-translator :fn(Sqlite::Stats) -> :Vec<E>)
  (cadence :Sqlite::MetricsCadence<G>)
  -> :Sqlite::Spawn<E>)

;; null-stats-translator opt-out — returns empty vec; Service still
;; ticks but emits nothing. Useful when self-heartbeat isn't wanted.
(:wat::core::define
  (:wat::std::telemetry::Sqlite/null-stats-translator
    (_stats :Sqlite::Stats) -> :Vec<E>)
  (:wat::core::vec :E))
```

### Educational entry-maker example (in `wat-tests/`)

```scheme
;; A 30-line example showing the consumer-side pattern. NOT shipped
;; in the substrate's stdlib — lives under wat-tests as
;; demonstration. Reading this teaches the shape; consumers write
;; their own.

;; 1. Define your entry enum.
(:wat::core::enum :my::log::Entry
  (Greeting (who :String) (timestamp-ns :i64))
  (Counter (name :String) (value :i64) (timestamp-ns :i64)))

;; 2. Define your maker — closure over a clock.
(:wat::core::define
  (:my::log::maker/make
    (now-fn :fn() -> :wat::time::Instant)
    -> :EntryMaker)
  ...)

;; 3. Define your dispatcher — knows your variants + your tables.
(:wat::core::define
  (:my::log::dispatch
    (db :wat::sqlite::Db) (entry :Entry) -> :())
  ...)

;; 4. Define your stats-translator — encodes Sqlite::Stats as your
;;    entry variant of choice.
(:wat::core::define
  (:my::log::translate-stats
    (stats :Sqlite::Stats) -> :Vec<my::log::Entry>)
  ...)

;; 5. Spawn.
(:wat::std::telemetry::Sqlite path 1
  :my::log::dispatch
  :my::log::translate-stats
  cadence)
```

This example is the docs's load-bearing teaching artifact. It compiles. It runs. It shows every piece a consumer brings.

---

## Slice plan

Three slices, each one named function-decomposition piece per the iterative-complexity discipline.

### Slice 1 — Substrate types (Stats, MetricsCadence, null-helpers)

`crates/wat-telemetry/wat/std/telemetry/Sqlite.wat`:
- `Sqlite::Stats` struct
- `Sqlite::MetricsCadence<G>` struct
- `Sqlite/null-metrics-cadence` factory

Tests at `wat-tests/std/telemetry/types.wat`:
- Construct each type; assert field round-trip.
- `null-metrics-cadence` returns a cadence whose tick never fires.

No service yet. No sqlite. Pure type primitives.

### Slice 2 — Substrate Sqlite Service shell (generic over E + dispatcher + translator)

`crates/wat-telemetry/wat/std/telemetry/Sqlite.wat`:
- The Service shell (queue + driver + select loop) — same shape as today's rundb Service.
- Dispatcher invocation (caller-provided `:fn(Db,E) -> :()`).
- Stats threading through loop (per the in-progress retrofit, formalized).
- tick-window<E,G>: on fire, calls `stats-translator stats → Vec<E>`, dispatches each through the same dispatcher.
- `Sqlite/spawn count cap dispatcher stats-translator cadence`.

Tests at `wat-tests/std/telemetry/sqlite.wat`:
- Tiny entry enum (`SmokeEntry::Greeting (who :String)`).
- Stub dispatcher pushes received entries onto a Vec via `:trading::test::collect-tx` (no actual sqlite required for the substrate test — caller is free to point dispatcher at anything).
- Spawn → send 5 batches → drop senders → join. Assert collected vec has 5 entries.
- Stats heartbeat test: counter cadence fires-every-3; spawn → send 9 batches → assert collected vec includes Stats-translated entries at the expected boundaries.

### Slice 3 — Educational example + USER-GUIDE section

`wat-tests/std/telemetry/example.wat`:
- The 5-step example above. Compiles + runs.
- Tests that exercise it end-to-end: construct entries with frozen-clock; assert deterministic timestamps.

`wat-rs/docs/USER-GUIDE.md` gains a new section: "Defining your own telemetry entries — the entry-maker pattern." Walks through the example. Explains the consumer-side trio (entry enum, maker, dispatcher, stats-translator).

The lab's call-site sweep (replace `:trading::rundb::Service` with `:wat::std::telemetry::Sqlite` + lab dispatcher) lands as **lab proposal 059-002**, not part of this arc. This arc ships substrate; consumers migrate at their own cadence.

---

## Open questions

### Q1 — Where does `wat-telemetry` live as a crate?

Three options:
- **Sibling crate**: `crates/wat-telemetry/`, mirrors `crates/wat-lru/`, etc.
- **Inside core**: `wat-rs/wat/std/telemetry/*.wat` + Rust sqlite shim in `src/`.
- **Two crates**: telemetry-core (no sqlite) + telemetry-sqlite.

Default: **sibling crate `wat-telemetry`**, with sqlite as the destination it ships. The crate has a Rust shim for sqlite write primitives (mirrors the existing rundb shim's contract). Future console destination is its own crate.

### Q2 — `:fn(Db, E) -> :()` shape vs richer dispatcher state

Today's dispatcher takes (db, entry). Future destinations might want richer state. Stay simple in slice 2; revisit if a third destination surfaces a need.

### Q3 — Should the `Sqlite::Stats` heartbeat be opt-in or always-on?

`null-stats-translator` returns empty vec — heartbeat ticks but emits nothing. That's the off-switch. Default: cadence is required; translator is required; opting out of heartbeat means passing the null-translator. Symmetry with arc 078's "both injection points required" philosophy.

### Q4 — Substrate-ship a "default" SmokeEntry to make trivial setups easier?

Tempting but rejected. The whole point of the user's correction is that substrate doesn't ship entry types. A "default" SmokeEntry would tempt consumers to use it instead of defining their own — exactly the wrong gravity. The educational example in `wat-tests/` is enough.

---

## Test strategy

- Slice 1: type primitive tests (no I/O).
- Slice 2: Service shape tests with stub dispatchers (no sqlite). Stats heartbeat verified via the test's collected vec.
- Slice 3: end-to-end example with frozen clock + assertion on entry order + timestamps.

Lab integration (sweep) lands in 059-002 with proof_005 as its capstone.

---

## Dependencies

**Upstream (must ship before this arc starts):** none.

**Downstream (this arc unblocks):**
- Arc 081 (telemetry::Console) — same Service shape, console-backed dispatcher.
- Lab proposal 059-002 (telemetry sweep) — sweeps producers onto the new substrate.

**Parallel-safe with:** Arc 079 (wat-edn shims) — independent. Arc 082 (SERVICE-PROGRAMS docs) — independent.

PERSEVERARE.
