# Arc 109 Slice K.telemetry — `Service` grouping noun → namespace flatten

**Status: shipped 2026-05-01.** Substrate (commit `0760a61`) +
consumer sweep (`5f7430a`). 17 files swept (1 stdlib + 16
consumer); 196 insertions / 196 deletions (pure rename, equal
counts); zero substrate-gap fixes. cargo test --release
--workspace 1476/0.

First application of § K's "/ requires a real Type" doctrine on
a real codebase. The `:wat::telemetry::Service` grouping noun
retired; verbs and typealiases live at the namespace level.
Real types Stats and MetricsCadence kept their `/methods` (just
one less namespace segment deep). Substrate-as-teacher mechanism
worked cleanly — sweep agent followed the diagnostic stream;
orchestrator verified `git diff --stat` independently.

Telemetry now serves as **the Pattern A reference** per § K's
channel-naming-patterns subsection — `ReqTx<P>` / `ReqRx<P>` +
`ReqChannel<P>` for data-forward; `AckTx` / `AckRx` /
`AckChannel` for unit-back release signals. K.console will mirror
this shape when it renames Console's `Tx`/`Rx` → `ReqTx`/`ReqRx`
+ adds the missing channel typealiases.

**Originally drafted as a compaction-amnesia anchor mid-slice;
preserved here as the durable record.** Slice K.telemetry is the
fourth Pattern 3 application after slices 1c/1d/1e/9d (the first
three target parsed-TypeExpr shapes; 9d + K.telemetry target
keyword prefixes — same mechanism, simpler detection level). The
walker (`validate_legacy_telemetry_service_path`) catches both
the typealias prefix (`:wat::telemetry::Service::`) AND the verb
prefix (`:wat::telemetry::Service/`) with a single function;
canonical replacement strips the `Service` segment.

## What this slice does

First application of § K's "/ requires a real Type" doctrine on a
real codebase. The `:wat::telemetry::Service` grouping noun is
fake-Type cosplay (no struct, no value, no kind — just a label
hung on top of a namespace). Under § K it retires; verbs and
typealiases flatten to bare `:wat::telemetry::*`. Real types
(Stats, MetricsCadence) keep their `/methods` because they ARE
structs.

Channel-naming pattern: telemetry is **Pattern A** (Request +
Ack) and is **already canonical** per gaze finding 2026-05-01.
This slice does NOT change channel typealiases — only the
Service-grouping flatten.

## Substrate work scope

### Service-grouping retirement

**17 typealiases flatten** (one less namespace segment):
```
:wat::telemetry::Service::AckChannel       → :wat::telemetry::AckChannel
:wat::telemetry::Service::AckRx            → :wat::telemetry::AckRx
:wat::telemetry::Service::AckTx            → :wat::telemetry::AckTx
:wat::telemetry::Service::Connection<E>    → :wat::telemetry::Connection<E>
:wat::telemetry::Service::DriverPair<E>    → :wat::telemetry::DriverPair<E>
:wat::telemetry::Service::Handle<E>        → :wat::telemetry::Handle<E>
:wat::telemetry::Service::HandlePool<E>    → :wat::telemetry::HandlePool<E>
:wat::telemetry::Service::IndexedDriverPair<E> → :wat::telemetry::IndexedDriverPair<E>
:wat::telemetry::Service::MetricsCadence<G> → :wat::telemetry::MetricsCadence<G>   ;; real struct; keeps /methods
:wat::telemetry::Service::Pending<E>       → :wat::telemetry::Pending<E>
:wat::telemetry::Service::ReqChannel<E>    → :wat::telemetry::ReqChannel<E>
:wat::telemetry::Service::ReqRx<E>         → :wat::telemetry::ReqRx<E>
:wat::telemetry::Service::ReqTx<E>         → :wat::telemetry::ReqTx<E>
:wat::telemetry::Service::Request<E>       → :wat::telemetry::Request<E>
:wat::telemetry::Service::Spawn<E>         → :wat::telemetry::Spawn<E>
:wat::telemetry::Service::Stats            → :wat::telemetry::Stats              ;; real struct; keeps /methods
:wat::telemetry::Service::Step<G>          → :wat::telemetry::Step<G>
```

**13 verbs flatten** (lose the `Service/` prefix; become bare
top-level verbs in the namespace):
```
:wat::telemetry::Service/ack-all                → :wat::telemetry::ack-all
:wat::telemetry::Service/batch-log              → :wat::telemetry::batch-log
:wat::telemetry::Service/bump-stats             → :wat::telemetry::bump-stats
:wat::telemetry::Service/drain-pairs            → :wat::telemetry::drain-pairs
:wat::telemetry::Service/extend                 → :wat::telemetry::extend
:wat::telemetry::Service/loop                   → :wat::telemetry::loop
:wat::telemetry::Service/loop-step              → :wat::telemetry::loop-step
:wat::telemetry::Service/maybe-merge            → :wat::telemetry::maybe-merge
:wat::telemetry::Service/null-metrics-cadence   → :wat::telemetry::null-metrics-cadence
:wat::telemetry::Service/pair-rxs               → :wat::telemetry::pair-rxs
:wat::telemetry::Service/run                    → :wat::telemetry::run
:wat::telemetry::Service/spawn                  → :wat::telemetry::spawn
:wat::telemetry::Service/tick-window            → :wat::telemetry::tick-window
```

**Stats and MetricsCadence keep their `/methods`** (real Types per § K):
```
:wat::telemetry::Service::Stats/batches         → :wat::telemetry::Stats/batches
:wat::telemetry::Service::Stats/entries         → :wat::telemetry::Stats/entries
:wat::telemetry::Service::Stats/max-batch-size  → :wat::telemetry::Stats/max-batch-size
:wat::telemetry::Service::Stats/new             → :wat::telemetry::Stats/new
:wat::telemetry::Service::Stats/zero            → :wat::telemetry::Stats/zero
:wat::telemetry::Service::MetricsCadence/gate   → :wat::telemetry::MetricsCadence/gate
:wat::telemetry::Service::MetricsCadence/new    → :wat::telemetry::MetricsCadence/new
:wat::telemetry::Service::MetricsCadence/tick   → :wat::telemetry::MetricsCadence/tick
```

The transformation rule is **uniform**: strip the `Service::`
segment from every `:wat::telemetry::Service::*` path, and strip
the `Service/` segment from every `:wat::telemetry::Service/*`
verb call. A single walker checks both prefixes.

## Pattern 3 walker

**`CheckError::BareLegacyTelemetryServicePath`** — fires on any
`WatAST::Keyword(s, span)` where `s.starts_with(":wat::telemetry::Service::")`
OR `s.starts_with(":wat::telemetry::Service/")`. The canonical
replacement strips that prefix; the diagnostic spells it out.

Same shape as slice 9d's `BareLegacyStreamPath` walker — pure
keyword-prefix detection, no parsed-TypeExpr inspection needed.

## What to ship

### Substrate (Rust + wat-stdlib)

1. **Rename inside `crates/wat-telemetry/wat/telemetry/Service.wat`** —
   every `:wat::telemetry::Service::X` and
   `:wat::telemetry::Service/X` becomes `:wat::telemetry::X`. ~282
   in-file refs (the wat-tests's count was for outside-the-crate
   consumers; this file's internal renames are separate).

2. **Mint `CheckError::BareLegacyTelemetryServicePath`** in
   `src/check.rs`:
   - `old`: the offending keyword
   - `new`: the canonical replacement (strip `Service::` or
     `Service/` segment)
   - `span`: source location
   - `Display` IS the migration brief; cites § K + the channel-
     pattern doctrine.

3. **Add walker `validate_legacy_telemetry_service_path`** that
   walks every WatAST keyword node and fires per occurrence.
   Wired into `check_program` alongside slice 9d's walker.

### Verification

Probe coverage:
- `(:wat::telemetry::Service/spawn ...)` → fires
- `(:wat::telemetry::spawn ...)` → silent
- `:wat::telemetry::Service::Stats/zero` → fires
- `:wat::telemetry::Stats/zero` → silent
- `:my::pkg::telemetry::Service::*` (user paths) → silent

## Sweep order

Same four-tier discipline as slices 1c-9d.

1. **Substrate stdlib** — `crates/wat-telemetry/wat/telemetry/Service.wat`
   (the file we just renamed internally) + any other crates/wat-telemetry/wat/
   files that reference it.
2. **Lib + early integration tests** — `src/check.rs` walker
   doc strings, any embedded wat strings in src/runtime.rs lib
   tests, src/stdlib.rs comments if present.
3. **`wat-tests/`** + **`crates/*/wat-tests/`** —
   `crates/wat-telemetry/wat-tests/telemetry/*.wat`,
   `crates/wat-telemetry-sqlite/wat-tests/telemetry/*.wat`.
4. **`tests/`**, **`examples/`**, **`crates/*/wat/`** (other
   crates that consume telemetry) — wat-scripts/, examples/,
   tests/, crates/wat-telemetry-sqlite/wat/telemetry/Sqlite.wat,
   etc.

Final gate: `cargo test --release --workspace` 1476/0;
`grep -rln ':wat::telemetry::Service[/:]' tests/ wat-tests/ wat/ examples/ crates/`
returns empty (or only the substrate's own legitimate recognizer
strings in src/check.rs).

## Estimated scope

- Internal renames in Service.wat: ~282 sites (the substrate file
  itself is dense)
- Consumer files (post survey): 25 files outside the crate
- Total occurrences across consumers: 282 (per pre-slice grep)
- Combined: ~564 rename sites across ~26 files

Comparable to slice 9d (286 sites). Sonnet-tractable single agent
sweep.

## What does NOT change

- **Stats and MetricsCadence as real types** — they keep their
  PascalCase names, their `/methods`, their struct definitions.
  They just live one less namespace segment deep.
- **The 4 channel typealiases** (`ReqTx`, `ReqRx`, `AckTx`,
  `AckRx`) — telemetry is the Pattern A reference; no channel
  renames in this slice.
- **Internal helper structure** — every define keeps its body;
  every typealias keeps its body. Pure naming.
- **Consumer call shapes** — `(:wat::telemetry::Service/spawn ...)`
  becomes `(:wat::telemetry::spawn ...)` mechanically; argument
  order and types unchanged.

## Closure (slice K.telemetry step N)

When sweep is structurally complete:

1. Update `INVENTORY.md` § K — strike telemetry's row in the
   "Grouping nouns that DON'T earn /" table; mark ✓ shipped.
2. Update `J-PIPELINE.md` — slice K.telemetry done; remove from
   independent-sweeps backlog.
3. Update `SLICE-K-TELEMETRY.md` — flip from anchor to durable
   shipped-record.
4. Add 058 changelog row noting the first § K application;
   names the Pattern A doctrine validation on a real codebase.

## Cross-references

- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § K — the doctrine
  this slice applies; "/ requires a real Type" + the channel-
  naming-patterns subsection.
- `docs/arc/2026/04/109-kill-std/SLICE-9D.md` — Pattern 3 walker
  precedent (keyword-prefix detection level).
- `docs/SUBSTRATE-AS-TEACHER.md` — the migration mechanism.
- `crates/wat-telemetry/wat/telemetry/Service.wat` — the file
  whose internal symbols flatten.
