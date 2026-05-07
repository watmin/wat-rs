# Arc 136 — `:wat::core::do` Form Substrate EXPECTATIONS (slice 1a)

**Drafted 2026-05-06.** Pre-handoff scorecard for slice 1a.

**Brief:** `BRIEF-SUBSTRATE.md`
**Output:** EDITS to `src/check.rs` + `src/runtime.rs` +
`src/special_forms.rs` + NEW `tests/wat_arc136_do_form.rs`. COMMIT
+ PUSH when tests pass.

## Setup — workspace state pre-spawn

- HEAD: `6b6b75a` (arc 145 closed; arc 136 DESIGN forward-amended)
- Working tree clean
- Pre-baseline: `cargo test --release --workspace` = 0 failed
- DESIGN locked: substrate special form, variadic args, no `-> :T`,
  final form's inferred type IS the do's type

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EXACTLY 4 files: `src/check.rs`, `src/runtime.rs`, `src/special_forms.rs`, NEW `tests/wat_arc136_do_form.rs`. No consumer wat edits. No other test files. No other crate. |
| 2 | `infer_do` non-final type discarded | Each non-final form is `infer`d (must internally check) but its resulting type is NOT unified with anything. Verified by test #6 (non-final type unconstrained). |
| 3 | `infer_do` final type returned | Final form's inferred type IS the do form's inferred type. Verified by tests #2, #3, #4. |
| 4 | Empty form rejected | `(:wat::core::do)` produces MalformedForm. Verified by test #1. |
| 5 | Recipient unification works | Do form's inferred type unifies cleanly with caller's slot. Test #4 passes; test #5 fires TypeMismatch at recipient. |
| 6 | Eval semantics correct | Each non-final evaluated for side effect; result discarded. Final's value returned. Verified by tests #2, #3, #9. |
| 7 | Special-form registry updated | `:wat::core::do` registered with variadic sketch. Reflection round-trip (test #7) passes. |
| 8 | Tail-call sanity | Test #8 verifies tail-call optimization preserved when do is in tail position. |
| 9 | Workspace clean | `cargo test --release --workspace` returns 0 failed. New symbol; no consumer breakage. |
| 10 | Honest report + commit | Per BRIEF reporting requirements; commit + push when Mode A. |

**Hard verdict:** all 10 must hold. Rows 2 + 3 + 5 + 6 are
load-bearing (substrate semantics + recipient interaction).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | 80-150 LOC across the 4 files. >250 = re-evaluate (probably scope creep). |
| 12 | Pattern fidelity to existing special forms | infer_do shape matches infer_if's structure (minus the `-> :T` parsing). Eval matches eval_let_star's iteration shape (minus the bindings logic). |
| 13 | clippy clean | No new clippy warnings. |
| 14 | No-grinding discipline | No backwards-compat shims. No defensive programming for hypothetical scenarios. |

## Independent prediction

- **Most likely (~70%) — Mode A clean.** Pattern is well-established
  (variadic special form is similar to existing forms minus the
  `-> :T` slot); brief is precise; pre-flight references give
  sonnet the canonical pattern. ~25-40 min wall-clock.
- **Mode B-substrate-internal-bug (~15%):** edge case in tail-call
  or step paths (incremental evaluator integration). Honest STOP
  + report.
- **Mode C-unexpected-interaction (~10%):** arc 144's reflection
  trio (lookup-form etc.) interacts with the new sketch in an
  unexpected way. Surface gap.
- **Mode B-time-violation (~5%):** sweep doesn't complete in 60
  min. Surface; re-brief if needed.

## Time-box

60 minutes wall-clock. ScheduleWakeup at T+60 min.

## What sonnet's success unlocks

**Mode A clean:** substrate gains `:wat::core::do`; commit + push
ships the new symbol. Slice 1b (consumer migration sweep
let*-with-unit-bindings → do) spawns next. Slice 2 closure ships
after.

**Mode B/C:** surface gap; orchestrator adjusts brief or substrate;
reland.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Verify load-bearing rows by re-running `cargo test --release --test
  wat_arc136_do_form` locally.
- Sample 1-2 new test bodies to verify the new symbol works as
  designed (recipient unification + non-final type discarded).
- Confirm sonnet committed + pushed.
- Open follow-up tasks for slice 1b (sweep) + slice 2 (closure).

## Why this matters

User direction 2026-05-06: "remove the work for typed let and start
on do forms." This is the start. Mode A clean = the substrate gains
a clean Clojure-faithful sequencing form; the let*-with-unit-bindings
crutch retires per slice 1b's sweep; arc 109 wind-down advances.

The mutual-agreement chain:
- User → Orchestrator: "start on do forms" (post-arc-145 back-out)
- Orchestrator → Sonnet (this brief): substrate-only edits;
  variadic special form; no `-> :T`; commit when tests pass
- Sonnet → Reality: substrate ships; recipient unification verifies;
  workspace stays 0-failed
