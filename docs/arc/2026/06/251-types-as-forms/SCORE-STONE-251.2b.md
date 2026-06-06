# SCORE — Stone 251.2b — signal + observe lift

**Verdict: PASS (green lift).** Orchestrator-verified, not sonnet's say-so. One tracked
re-export-surface item routed to the 251.2e ward (purgare).

## Scorecard (orchestrator re-run)

| # | What | Result |
|---|---|---|
| 1 | new files | ✓ `src/value/{observe.rs, signal.rs}` (home now: encoding_ctx, frame, mod, observe, signal) |
| 2 | signal block gone from runtime.rs | ✓ (the 3 grep hits are COMMENTS — "con·struct TrackedValue" substring in doc lines 4575/8065/14586; no real defs) |
| 3 | observe block gone from runtime.rs | ✓ |
| 4 | render_value moved | ✓ runtime.rs=0 defs; observe.rs:183 `pub(crate) fn render_value` (line-4 hit is the module doc) — NO eval back-references (confirmed clean move) |
| 5 | 251.2a dead pubs dropped | ✓ EncodingCtx + FrameInfo/snapshot_call_stack now plain `use` |
| 6 | **lib tests IDENTICAL** | ✓ **923 / 0 / 1** |
| 7 | clippy in-home | ✓ nothing citing src/value/ |
| 8 | external API intact | ✓ `pub use value::{RuntimeError, RuntimeErrorKind}` in lib.rs |

Files: observe.rs + signal.rs created; runtime.rs (3 blocks removed, render_value call repointed to
`crate::value::observe::render_value`, imports added); lib.rs (re-export moves); ~8 consumer files
repointed (argspec/error, assertion, edn_shim, harness, io, macros/eval, macros/expand, runtime_error_edn).

## Re-export audit (the finding — same class as 251.2a's dead pubs)

runtime.rs carries 2 re-export lines (1866, 1871) covering 7 moved types. Audited each for a
`crate::runtime::<type>` consumer:

| type | old-path consumers | re-export status |
|---|---|---|
| ValueSnapshot | **74** | LOAD-BEARING (correct re-export) |
| RuntimeError | 1 | marginal (1 consumer) |
| Provenance, TrackedValue, EvalBreak, EvalSignal, RuntimeErrorKind | **0** | **dead `pub`** |

The dead ones: runtime.rs needs the `use` (its eval loop uses EvalBreak/EvalSignal/RuntimeErrorKind/
TrackedValue/Provenance internally) — but the `pub` re-exports them on `crate::runtime::*` with zero
consumers. Only the `pub` is dead; the import stays (plain `use`). Sonnet over-applied the zero-churn
re-export ("backstop") even after repointing consumers — the brief's principle was applied
inconsistently.

## Disposition — ward-time, not a per-stone treadmill

The `value/` home is NOT warded until 251.2e. **purgare** (vigilia spell, owns dead code) is the
enforcement point. Cleaning dead `pub` at every sub-stone is churn (2c/2d each add their own re-export
surface). **Plan: a single purgare sweep at the 251.2e WARD converts every non-externally-consumed
`pub use crate::value::X;` → plain `use` (keep the load-bearing ValueSnapshot re-export; repoint or
keep RuntimeError's 1).** Tracked here + carried in the 251.2e ward scope; named-mechanism, not a
deferral-violation. (251.2a's 2 dead pubs WERE cleaned in-stone; going forward the re-export surface
is transitional until the ward.)

## Next: 251.2c

`environment.rs` ← Environment/EnvBuilder/EnvCell/BoundEntry + co-locate `Function` (carries
`closed_env: Option<Environment>`; and signal.rs's EvalSignal::TailCall + the value home all reference
Function — lifting it here resolves the transitional `use crate::runtime::Function`). Baseline 923/0/1.
Apply the re-export principle (likely RE-EXPORT Environment/Function — heavily used); do NOT add new
dead pubs (or accept them as ward-swept). Then 251.2d symbol_table, 251.2e value.rs + the vigilia ward
(with the purgare re-export sweep).
