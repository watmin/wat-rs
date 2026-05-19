# Arc 212 stone δ-process-scope — SCORE

## Header

Stone: δ-process-scope
File edited: `src/check.rs`
Function edited: `collect_process_calls` (~line 3749)
Date: 2026-05-18

---

## Summary

The sharpening applied two changes to `collect_process_calls`, bounded to the
function body:

1. **`:wat::core::let` added to the scope-boundary match arm** alongside the
   pre-existing `:wat::core::fn` | `:wat::core::lambda` arm. The walker now
   stops descent at let-form boundaries, matching the caller
   (`find_process_join_before_drain` → `infer_let`) which is already invoked
   per-let-scope. This prevents conflation of inner-let Process accessors with
   the outer scope's tracking.

2. **Recursion migrated from `for child in items` (List-only) to
   `for child in node.children()` (generic).** The `if let WatAST::List` guard
   now wraps only the List-head classification logic; the recursion loop
   executes unconditionally via `children()`. This extends traversal coverage
   to Vector and StructPattern nodes uniformly. Scope-boundary arms return
   before reaching the loop, preserving the stop-at-boundary invariant.

3. **Comment block updated** from the TEMPORARY audit-evidence framing to the
   permanent explanation of the let scope boundary and the per-let-scope caller
   framing.

LOC changed: ~20 (comment replacement + if-let restructure + let arm addition +
recursion collapse).

---

## Verification

| Test | Result |
|---|---|
| `cargo test --release --test wat_arc170_stone_a_drain_and_join` | PASS (4/4) |
| `cargo test --release --test wat_arc202_process_join_holds_stdin` | PASS (3/3) |
| `cargo test --release --test probe_run_hermetic_no_deadlock` | PASS (2/2) |

---

## Build

`cargo build --release` — CLEAN (16.73s, no warnings from edited file)

---

## Honest-delta note

No Mode B surface. All three tests passed post-migration with no new
`ProcessJoinBeforeOutputDrain` false positives. The let scope-boundary
addition mirrored the existing fn/lambda boundary cleanly; the architecture
(caller per-let-scope) held exactly as predicted.

---

## Scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `collect_process_calls` uses `node.children()` for recursion | YES |
| 2 | `:wat::core::let` added to scope-boundary match arm alongside fn/lambda | YES |
| 3 | Existing Process classification logic preserved verbatim | YES |
| 4 | `cargo test --release --test wat_arc170_stone_a_drain_and_join` green | YES |
| 5 | `cargo test --release --test wat_arc202_process_join_holds_stdin` green | YES |
| 6 | `cargo test --release --test probe_run_hermetic_no_deadlock` green | YES |
| 7 | `cargo build --release` clean | YES |
| 8 | SCORE file written; sharpening described | YES |
| 9 | Zero other code edits anywhere | YES |

---

## Mode classification

**Mode A** — all nine criteria satisfied; no STOP triggers fired.
