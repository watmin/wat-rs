# Arc 081 — `:wat::std::telemetry::Console` (EDN/JSON-per-line dev sink)

**Status:** PROPOSED 2026-04-29. Pre-implementation reasoning artifact.

**Predecessors:**
- Arc 079 — `:wat::edn::*` shims (renderer for any wat value).
- Arc 080 — substrate Sqlite Service shell + Stats + MetricsCadence. Same Service contract; this arc is the parallel destination.
- `wat/std/service/Console.wat` — the existing tagged-stdout primitive. Each line of console output goes through this gateway.

**Surfaced by:** The user's directive (2026-04-29):

> "no free form log lines... no rando (println! ...) bullshit.... the users must operate on data at all times"
>
> "we created a [crate] on the side... wat-edn... for console reporting.. we emit edn per line or json per line - the user chooses - we just print the data forms as the log line...."

The substrate's console output today (`Console/out`) takes a `:String`. That's the lowest gateway. ABOVE it, every log line must come from a structured value rendered through the EDN/JSON renderer at the boundary. Console::Telemetry is that wrapper.

---

## What this arc is, and is not

**Is:**
- A queue-fronted destination service at `:wat::std::telemetry::Console<E,G>`.
- Same Service contract as `Sqlite<E,G>` (arc 080) — accepts batches of E, runs a dispatcher per entry, threads Stats + MetricsCadence, emits self-heartbeat through dispatcher.
- The DISPATCHER for Console is internal: each entry gets rendered via `:wat::edn::write` (or `:wat::edn::write-json` per format-knob) and the resulting string is sent via `Console/out`.
- A spawn-time format knob: `Console::Format::Edn` | `Console::Format::Json`. Caller picks once at construction.
- One line per entry. No batching at the print layer. No prose. Every line is a parseable EDN or JSON value.

**Is not:**
- A logger replacement for stderr checkpoints. Existing `Console/err` use cases (T1/T2/T3 stderr markers) keep working — they're at a lower level. Telemetry::Console is for STRUCTURED entries; raw stderr stays for low-level diagnostics.
- A dispatcher abstraction. Unlike Sqlite, the Console destination's dispatcher is built-in (rendering + Console/out is the only sensible behavior). Caller doesn't supply one.
- A pretty-print layer. `:wat::edn::write` is compact-by-default; one line per entry. `write-pretty` is a separate primitive consumers reach for explicitly.
- A stats-translator OPT-OUT wrapper. The substrate Service contract from arc 080 still applies — caller provides a `stats-translator :fn(Stats) -> :Vec<E>`. Console's heartbeat goes through render-then-print like any other entry.

---

## Surface

### Format knob (substrate)

```scheme
(:wat::core::enum :wat::std::telemetry::Console::Format
  Edn   ;; render via :wat::edn::write (compact)
  Json) ;; render via :wat::edn::write-json
```

### Service spawn

```scheme
(:wat::std::telemetry::Console<E,G>
  ;; Console handle the destination uses to write each line.
  ;; Caller pops this from a Console/spawn HandlePool.
  (console-tx :wat::std::service::Console::Tx)
  (count :i64)
  ;; Substrate Service contract — same as Sqlite.
  (stats-translator :fn(Sqlite::Stats) -> :Vec<E>)
  (cadence :Sqlite::MetricsCadence<G>)
  ;; Format knob — picked once at construction.
  (format :Console::Format)
  -> :Console::Spawn<E>)
```

The substrate contract reuses arc 080's `Sqlite::Stats` and `Sqlite::MetricsCadence<G>` types — they're not actually sqlite-specific despite the namespace. (Open question Q1.)

### Internal dispatch (NOT consumer-facing)

```scheme
;; Substrate-defined; not exposed for override. The Console
;; destination always dispatches by rendering through wat-edn.
(:wat::core::define
  (:wat::std::telemetry::Console/dispatch<E>
    (console-tx :Console::Tx)
    (format :Console::Format)
    (entry :E)
    -> :())
  (:wat::core::let*
    (((line :String)
      (:wat::core::match format -> :String
        (:Console::Format::Edn  (:wat::edn::write entry))
        (:Console::Format::Json (:wat::edn::write-json entry)))))
    (:wat::std::service::Console/out console-tx
      (:wat::core::string::concat line "\n"))))
```

Per-line newline appended at the boundary — the renderer doesn't include trailing newline (one line is one value).

---

## Slice plan

### Slice 1 — Format enum + dispatch helper

`crates/wat-telemetry/wat/std/telemetry/Console.wat`:
- `Console::Format` enum
- `Console/dispatch` helper

Tests at `wat-tests/std/telemetry/console-dispatch.wat`:
- Stub Console::Tx that captures sent strings into a Vec.
- Build a tiny entry, dispatch via Edn, assert the captured string is the EDN render.
- Same for Json.

### Slice 2 — Service shell (parallel to Sqlite Service)

`crates/wat-telemetry/wat/std/telemetry/Console.wat`:
- The Service spawn function.
- Same Stats + MetricsCadence + tick-window pattern as Sqlite — the only difference is the dispatcher is the built-in render+print one.

Tests:
- Smoke test: spawn → send 3 entries (one Edn, one Json, one Edn) → drop → join. Capture stdout via the existing `run-hermetic-ast` harness; assert each line is the expected render.
- Stats heartbeat: cadence fires after 3 entries; assert heartbeat lines render correctly.

### Slice 3 — USER-GUIDE section + cross-doc with arc 080

USER-GUIDE.md gains "Logging to console — telemetry::Console" — explains the format knob, the one-line discipline, and how the same entry-maker pattern from arc 080 works here too. The consumer's Reporter is the only thing that swaps; Console vs Sqlite is a deployment-layer choice.

---

## Open questions

### Q1 — Stats type: per-destination or shared?

Currently I propose Console reuses `:wat::std::telemetry::Sqlite::Stats`. That's awkward — Stats lives under Sqlite's namespace. Two options:

- **Promote Stats to `:wat::std::telemetry::Stats`**, used by both Sqlite and Console (and any future destination). Slight scope creep on arc 080.
- **Console has its own ConsoleStats** with the same shape. Names align but types are distinct.

Default: **promote to `:wat::std::telemetry::Stats`** as part of arc 080's scope (move Stats out of Sqlite-specific namespace). Same for MetricsCadence. Telemetry destinations share the contract.

### Q2 — Single-Console handle or per-line scoped?

The Service holds ONE Console::Tx for its lifetime. All entries go through it. If the trader needs telemetry to stdout AND its own stderr-checkpoints, it spawns Console with multiple handles and hands one to the telemetry::Console service.

That's already how Console/spawn works. No change needed. Document the wiring in USER-GUIDE.

### Q3 — Pretty mode?

Initial slice ships compact (`write` not `write-pretty`). Pretty is a follow-up arc if a consumer surfaces a debug-mode need.

### Q4 — Multiple format-knobs per spawn (mixed Edn+Json)?

Not in scope. One format per Console destination. Two destinations if you want both. Keeps the contract simple.

---

## Test strategy

- Slice 1: render → string → assert. Pure transform tests.
- Slice 2: full service shape — spawn, drive, drop, join. Capture stdout; assert per-line equals expected EDN/JSON render.
- Slice 3: docs only.

---

## Dependencies

**Upstream (must ship before this arc starts):**
- **Arc 079** (wat-edn shims) — slice 1 needs `:wat::edn::write` and `:wat::edn::write-json`.
- **Arc 080** (Sqlite Service substrate) — slice 2 needs the substrate `Stats` + `MetricsCadence` types and their conventions. Q1's promote-to-shared-namespace lands as part of arc 080.

**Downstream (this arc unblocks):**
- Lab proposal 059-002 — telemetry sweep can target both Sqlite + Console.
- Future cross-domain consumers (MTG, truth-engine) — choose Console for dev, Sqlite for prod, both with one Reporter swap.

**Parallel-safe with:** Arc 082 (SERVICE-PROGRAMS docs) — independent.

PERSEVERARE.
