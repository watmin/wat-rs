# DESIGN — Stone 251.2 — LIFT + WARD `src/value/` (THE KEYSTONE)

**Parent:** arc 251 (the great migration; `SCOUT-LIFT-MAP.md`). **Home name:** `src/value/`
(intueri-blessed, 2026-06-06). **This is the keystone** — the registration pipeline, the
concurrency eval, the collection fanout, `derive_scheme_from_function`, and every `match value`
in the codebase import these types. Nothing downstream in the migration unblocks until `value/`
stands. It lifts FIRST.

## What this stone is — a LIFT, not a transform

`value/` is a **pure structural lift** of the runtime's core data model out of the flat
`runtime.rs` (31,328 lines) into a warded home. **No behavior changes.** The clojure-ination
TRANSFORMS that touch these types (Value's keyword-encoded `type_name`, SymbolTable's keyword-keyed
maps) are NOT done here — they are marked `// TRANSFORMS — clojure-ination` in-source and reshaped
at their later symbol-surface stone. 251.2's contract: the lib + corpus baseline is **identical
pre/post**. Any behavior delta is a bug, not a feature. (Same discipline as 251.1a; no FM-2-bis
probe — there is no new behavior to disconfirm; the verification is baseline-identical.)

## The carve (Scout 1, intueri-named)

`src/value/` submodules:

| Submodule | Holds | Scout 1 lines (runtime.rs) |
|---|---|---|
| `value.rs` | `Value` enum + impls (PartialEq/Eq/Hash/`impl Value`) + payloads (`StructValue`, `EnumValue`, `SpawnOutcome`, `ProgramHandleInner`, `Clause`/`ClauseSet`/`ClauseAttempt`/`ClauseFailureReason`) + `sequence_eq`/`hash_sequence` | 383–1408 + the Clause cluster 710–767 |
| `environment.rs` | `Environment`, `EnvBuilder`, `EnvCell`, `BoundEntry` + **co-locate `Function`** (carries `closed_env: Option<Environment>`) | 1413–1613 |
| `symbol_table.rs` | `SymbolTable` + impl | 1662–1961 |
| `observe.rs` | `TrackedValue`, `ValueSnapshot`, `Provenance` + **`render_value`** (moved here — see entanglement) | 1931–2118 + render_value 18628 |
| `signal.rs` | `EvalSignal`, `EvalBreak`, `RuntimeError`, `RuntimeErrorKind` + impls | 2094–2660 |
| `frame.rs` | `FrameInfo`, `FrameGuard`, `CALL_STACK`, `snapshot_call_stack`, `replace_top_frame` | 20105–20200 |
| `encoding_ctx.rs` | `EncodingCtx` | 1595–1638 |
| `mod.rs` | home doc + vigilatum stamp + re-exports (the `lib.rs` public surface) | — |

(`observe.rs` carries the weigh-flag: revisit vs `provenance.rs` while coding — confirm the name
unifies TrackedValue + ValueSnapshot + Provenance. `encoding_ctx.rs` keeps the struct-name abbrev.)

## The one entanglement — `render_value` (the key design decision)

`ValueSnapshot::of()` calls `render_value` (runtime.rs:18628) — ~80 lines of `match Value` living
deep in the eval-interior. **DECISION: `render_value` moves into `value/observe.rs`.** It IS the
value→display engine that serves `ValueSnapshot`; it matches only on `Value` variants (all in the
value home post-lift), so it belongs with the snapshot it feeds. This keeps `ValueSnapshot::of` a
real constructor (not a shim left behind in runtime.rs). Verify: `render_value` has no eval-loop
back-references (Scout 1 read it as a pure display match — confirm at the stone; if it calls back
into eval, fall back to keeping `ValueSnapshot::of` a thin shim in runtime and moving only the type).

## Sub-stone sequence (Scout 1's difficulty order — each green→green, single-axis)

- **251.2a — `frame.rs` + `encoding_ctx.rs`** (EASY, fully independent, the entry frontier).
  `frame` (call-stack RAII, private except FrameInfo/snapshot_call_stack) + `encoding_ctx`
  (Config→encoder wiring). Zero coupling to eval logic. Proves the home + the lift mechanics on
  the lowest-risk segment. **START HERE.**
- **251.2b — `signal.rs` + `observe.rs`** (MEDIUM — resolve `render_value`). signal carries
  `Box<ValueSnapshot>` → observe must land with/before it.
- **251.2c — `environment.rs`** (MEDIUM — co-locate `Function`; `Environment::lookup` → `TrackedValue`
  so observe precedes it).
- **251.2d — `symbol_table.rs`** (MEDIUM — god-struct, wide one-way imports: load/sigma/macros/
  types/thread_io; all one-way, no cycle).
- **251.2e — `value.rs`** (HARD — foundational; LAST so all dependents can already import from the
  new home; mark the `impl Value` `type_name`/`declared_type_name` block `// TRANSFORMS`).

After 251.2e: vigilia ward the whole `value/` home → L1+L2=0 → vigilatum stamp; clippy-clean in-home.

## Verification (every sub-stone)

- `cargo test --release --lib -p wat` PASS-count IDENTICAL pre/post (pure lift).
- `./scripts/integration-run.sh` corpus baseline IDENTICAL.
- `lib.rs` re-exports updated (EncodingCtx/EnvBuilder/Environment/Function/RuntimeError/
  RuntimeErrorKind/StructValue/SymbolTable/Value → new `crate::value::*` paths) — external API
  surface unchanged (re-exports preserve `wat::Value` etc.).
- clippy clean in the new home.
- NO non-lift edits (no behavior change; no clojure-ination transform — only `// TRANSFORMS` markers).

## Out of scope = rejected (affirmative cuts)

- The clojure-ination TRANSFORMS (Value keyword type-names → symbol; SymbolTable keyed maps) —
  marked in-source, reshaped at the symbol-surface stone, NOT here.
- The registration pipeline (`register_*`) — depends on `value/` being stable; a LATER stone.
- The runtime leaf ops (scalar/algebra) + the eval spine — separate homes, separate stones.
- Any behavior change — 251.2 is structural only.

## Next (251.2a)

Crawl the `frame` + `encoding_ctx` regions to confirm exact boundaries + the `lib.rs`/consumer
import sites; write BRIEF + EXPECTATIONS for the pure lift (positive-only; rooms as read-in-order
`file:line`; STOP = rejection); spawn sonnet (`model:"sonnet"`, background); score against an
independent baseline re-run (lib + corpus identical); then proceed down the sub-stone sequence to
251.2e, and vigilia-ward the home.
