# BRIEF — stone ①: the `:wat::telemetry'` records in core (the data foundation)

> **Executor: one sonnet LEAF.** Orchestrator drew this; weighs the kill against its own re-run. Work ONLY in
> `/home/watmin/work/holon/wat-rs/`, NEVER worktrees. `pwd` first. `cargo nextest run` (NEVER `cargo test`), and
> `./target/release/cargo-wat <file>` to dogfood a wat file. Commit NOTHING — leave the tree for the orchestrator.

## The work (one paragraph)

Add the telemetry facility's **data vocabulary** as a new **core** wat source: the `:wat::telemetry'` records/enums that
`Metric`/`Log` are built from. This is the first real consumer of **surface-splice** (shipped `4c98b2ef`) — `Metric` and
`Log` are `defrecord`s that **splice the `Scope` surface** `[~@:wat::telemetry'::Scope own…]`. The namespace is **primed**
(`:wat::telemetry'`, staged to replace the loaded `wat-telemetry` battery bridge — no collision). It's a core baked
source, so `:wat::` is legal here (stdlib bypasses the reserved-prefix gate).

## The exact forms (from `DESIGN-telemetry-service-and-query-surface.md § contractual surface`)

Define in `:wat::telemetry'`:
- `Tags` — `(:wat::core::typealias :wat::telemetry'::Tags (:wat::core::HashMap :wat::core::Keyword :wat::core::String))`
- `Numeric` — `(:wat::core::defenum :wat::telemetry'::Numeric :wat::enum::Pure  i64 [val <- :wat::core::i64]  f64 [val <- :wat::core::f64])`
- `Unit` — `(:wat::core::defenum :wat::telemetry'::Unit :wat::enum::Pure  Count Nanos Millis Bytes Percent)`
- `Level` — `(:wat::core::defenum :wat::telemetry'::Level :wat::enum::Pure  Debug Info Warn Error)`
- `Scope` — an **exact surface**:
  `(:wat::core::defsurface :wat::telemetry'::Scope :holder :wat::core::Record :features [namespace <- :wat::core::String  uuid <- :wat::core::Uuid  tags <- :wat::telemetry'::Tags  time-ns <- :wat::core::i64])`
- `LogMessage` — an **open surface**: `(:wat::core::defsurface :wat::telemetry'::LogMessage :holder :wat::core::Record :features [])`
- `Metric` — a **`defrecord` that SPLICES `Scope`**:
  `(:wat::core::defrecord :wat::telemetry'::Metric [~@:wat::telemetry'::Scope  start-time-ns <- :wat::core::i64  name <- :wat::core::Keyword  value <- :wat::telemetry'::Numeric  unit <- :wat::telemetry'::Unit])`
- `Log` — a **`defrecord` that SPLICES `Scope`**:
  `(:wat::core::defrecord :wat::telemetry'::Log [~@:wat::telemetry'::Scope  caller <- :wat::core::Keyword  level <- :wat::telemetry'::Level  message <- :wat::telemetry'::LogMessage])`

Constructor field order (splice-first, arc-293): a `Metric` constructs positionally as
`(:wat::telemetry'::Metric namespace uuid tags time-ns  start-time-ns name value unit)` — the 4 spliced `Scope` fields,
then the 4 own. Same for `Log` (`namespace uuid tags time-ns  caller level message`). Confirm `:wat::core::Uuid` exists
(grep) — if the type is spelled differently, use the real one and note it.

## Read in order (the rooms)

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-telemetry-service-and-query-surface.md`** — § contractual surface (the
   authoritative forms) + § surface architecture (records satisfy the `Scope` surface; splice is the built mechanism).
2. **`wat/core.wat`** — a real `defsurface` (`:wat::core::Error`, ~line 1465), a real `defrecord`, a real
   `defenum … :wat::enum::Pure` (grep `runtime-meta.wat` for enum exemplars). Copy the exact declaration shapes.
3. **`tests/types/probe_arc293_surface_splice.wat`** — the WORKING splice exemplar (`~@:Surface` in a `defrecord` field
   vector, constructed + field-read). `Metric`/`Log` mirror it exactly.
4. **`src/stdlib.rs`** — the baked-source list (`include_str!("../wat/…")`, order = dependency order). Add the new file
   AFTER `wat/core.wat` (which defines defsurface/defrecord/defenum + splice) and after anything defining `:wat::core::Uuid`.

## Where it lands

- New core source **`wat/telemetry.wat`** (the `:wat::telemetry'` records above), registered in `src/stdlib.rs` at the
  right dependency position.
- The RED gate: **`tests/types/probe_arc278_telemetry_records.{rs,wat}`** (committed, `#[ignore]`'d). Un-ignore as the
  final step; it must go green. It constructs a `Metric` splicing `Scope`, reads a **spliced** field (`namespace`) and an
  **own** field (`name`), and round-trips the record through EDN (`#wat.telemetry'/Metric {…}` → read back → equal).

## STOP triggers (rejection criteria — surface, don't hack)

- **STOP-PRIMED-NS:** if `:wat::telemetry'::` fails to parse/register in a core source (the primed namespace segment),
  STOP and report the reader/registration error — do NOT fall back to an unprimed `:wat::telemetry::` (that collides with
  the loaded battery) or a different namespace.
- **STOP-SPLICE:** if `~@:wat::telemetry'::Scope` in the `defrecord` field vector doesn't expand (splice should be live
  per `4c98b2ef`), STOP and report — do NOT re-list Scope's fields by hand (that's the duplication splice exists to kill).
- **STOP-UUID:** if `:wat::core::Uuid` doesn't exist, STOP and report the real spelling; don't invent a type.

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| the records load (core boots) | `./target/release/cargo-wat <a tiny wat that constructs a Metric>` | prints the Metric's EDN, no error |
| the acceptance probe green | `cargo nextest run --release --run-ignored all -E 'test(telemetry_records)'` | 1+ passed |
| whole gate, floor 0 | `cargo nextest run --release` | `0 failed` (modulo the known arc-290-300 `no_inlined_wat_in_tests` reminder) |

Runtime ~30–45 min (includes a release rebuild — the stdlib change forces it). Trap-door: the constructor field order
(splice-first) — if the probe constructs in the wrong order it'll type-error; match splice-then-own.

## Blast radius (bounded)

`wat/telemetry.wat` (new) + `src/stdlib.rs` (one list entry) + the RED probe (un-ignored). No Rust type/runtime change.
No touching the `wat-telemetry` battery (it stays as the bridge). Use `deftest'` for any wat-side test.
