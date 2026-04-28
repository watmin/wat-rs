# Telemetry roadmap (arcs 079–082 + lab 059-002)

**Status:** PROPOSED 2026-04-29. Five-arc dependency map for the data-not-text observability rebuild.

## What we're building

The substrate's logging story today is shaped by accident: `:trading::rundb::Service` (lab-side) wraps sqlite, accepts a hardcoded `LogEntry` enum, and is the only destination available. Console output (`:wat::std::service::Console`) takes free-form `:String`. There is no shared rendering primitive; consumers stringify-at-the-callsite when they want logs to be readable.

We're rebuilding it under one principle: **users operate on structured data; renderers run at the boundary**. Substrate ships generic shells; consumers define their own entry types; one Reporter swap chooses sqlite vs console as the destination.

## The five arcs

```
                                 ┌──────────────────────┐
                                 │ 079 wat-edn shims     │
                                 │ :wat::edn::write*     │
                                 │  any wat value -> EDN │
                                 │  / JSON / pretty      │
                                 └──────────┬───────────┘
                                            │ (renderer)
                                            ↓
   ┌──────────────────────┐      ┌──────────────────────┐
   │ 080 telemetry::Sqlite │      │ 081 telemetry::Console│
   │  generic Service shell │ ←─── │  EDN/JSON-per-line    │
   │  Stats + MetricsCadence│      │  uses 079 + 080's      │
   │  caller-provided       │      │  Stats/Cadence types  │
   │  dispatcher + translator│     │                       │
   └──────────┬─────────────┘      └──────────┬─────────────┘
              │                                │
              └────────────────┬───────────────┘
                               ↓
                    ┌──────────────────────┐
                    │ lab 059-002:         │
                    │ telemetry sweep      │
                    │ - lab Entry enum     │
                    │ - lab dispatcher     │
                    │ - entry-maker factory │
                    │ - reporter migration │
                    │ - proof_005 ships    │
                    └──────────────────────┘

   ┌──────────────────────┐
   │ 082 SERVICE-PROGRAMS │  (docs only — independent, parallel-safe with all)
   │  Step 9: nested      │
   │  service shutdown via │
   │  function decomposition│
   └──────────────────────┘
```

## Arc-by-arc

| Arc | Surface | Where | Depends on |
|---|---|---|---|
| **079** wat-edn shims | `:wat::edn::write` / `write-pretty` / `write-json` — render any wat value to text | wat-rs core | nothing |
| **080** telemetry::Sqlite | `:wat::std::telemetry::Sqlite<E,G>` — generic Service shell, caller-provided dispatcher + stats-translator. Stats + MetricsCadence shipped here (shared by 081). | new crate `wat-telemetry` | nothing |
| **081** telemetry::Console | `:wat::std::telemetry::Console<E,G>` — same Service shape, built-in render-via-wat-edn dispatcher, format knob (Edn / Json) | `wat-telemetry` | 079, 080 |
| **082** SERVICE-PROGRAMS docs | Step 9 added: function-decomposition pattern for multi-driver scope nesting (the proof_004 lesson) | wat-rs/docs | nothing |
| **lab 059-002** | Lab Entry enum, dispatcher, stats-translator, entry-maker factory; reporter migration; proof_005 | holon-lab-trading | 080 (REQUIRED), 079 + 081 (only if Console swap wanted) |

## Substrate ships zero entry variants

The load-bearing decision (per the user's 2026-04-29 correction):

> "the LogEntry /must/ be user defined - we do not provide anything here.. or maybe we provide extremely basic things to educate the reader on how to implement their own bespoke entries with whatever complexity"

Substrate's `Sqlite<E,G>` is generic over E. Substrate's `Console<E,G>` is generic over E. Substrate ships:
- Service shell (queue, driver, select loop)
- `Stats` (the Service's own internal counters)
- `MetricsCadence<G>` (the gate)
- Null helpers
- ONE small educational example in `wat-tests/std/telemetry/` showing how to roll your own entry type

Substrate ships ZERO entry variants. Every consumer defines its own enum. Every consumer brings its own dispatcher (`:fn(Db, E) -> :()`) and stats-translator (`:fn(Stats) -> :Vec<E>`).

## Suggested execution order

Per the iterative-complexity discipline (memory: feedback_iterative_complexity), each arc has independent green checkpoints.

1. **082 (docs)** — purely additive; can ship anytime; informs reading of every other arc.
2. **079 (wat-edn shims)** — independent foundation; small slice; smallest stepping stone.
3. **080 (substrate Sqlite Service)** — biggest of the substrate arcs; absorbs the in-progress rundb retrofit.
4. **081 (substrate Console)** — built on 079 + 080.
5. **lab 059-002** — sweeps the lab onto the new substrate. proof_005 caps it.

This order keeps each arc's downstream small. 082 first means readers (and we ourselves) have the right mental model when reviewing 079–081. 079 then 080 then 081 means each substrate piece lands on its own without the next blocking it. Lab last — substrate is settled when the trader migrates.

## Where the in-progress rundb retrofit goes

The Stats + MetricsCadence threading currently sitting in `holon-lab-trading/wat/io/RunDbService.wat` (uncommitted-as-its-own-arc) absorbs into arc 080. The substrate-Sqlite-Service slice is where that pattern formalizes. The lab work in `wat/io/RunDbService.wat` revert-or-retain depending on whether arc 080 ships before lab 059-002:

- If arc 080 ships first: the lab's RunDbService is fully replaced — retrofit absorbed into substrate.
- If lab 059-002 ships before arc 080 (unlikely given the dependency): retrofit stays as lab-side until arc 080 lands.

PERSEVERARE.
