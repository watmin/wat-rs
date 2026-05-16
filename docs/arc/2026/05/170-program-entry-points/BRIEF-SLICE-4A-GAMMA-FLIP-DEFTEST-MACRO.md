# Arc 170 Slice 4a-γ-flip BRIEF — flip deftest macro body to run-thread

**Task:** #314
**Phase:** Slice 4a-γ third sub-stone (audit ✓ → decorate ✓ → flip).
**Predecessors:**
- Audit shipped at `f2e78ea` — 5 sites flagged for hermetic decoration
- Decorate shipped at `7e1f417` — all 5 flagged deftests now use `:wat::test::deftest-hermetic`; site 154 rearchitected; sites 200/225 carry duplicate-marker comments
- Every plain `:wat::test::deftest` in the codebase is now safe for run-thread default per the three-rule classification

## Goal

One-line change to the deftest macro body at `wat/test.wat:303`:

```
;; Before
(:wat::test::run-hermetic ~body)

;; After
(:wat::test::run-thread ~body)
```

Plus a doc-comment refresh on the surrounding header (lines 260-273) to reflect the new thread-default behavior. After this slice, every `:wat::test::deftest` invocation expands to a `run-thread` call; every `:wat::test::deftest-hermetic` invocation continues to expand to `run-hermetic`. The doctrine ("thread by default; hermetic by explicit choice") lands at the user-facing macro layer.

## Edits in scope

Three small edits to `wat/test.wat`:

1. **Line 303 (macro body):** `(:wat::test::run-hermetic ~body)` → `(:wat::test::run-thread ~body)`
2. **Lines 263-273 (header doc-comment):** rewrite to describe thread-default semantics + cite FM 7-ter for hermetic-required cases.
3. **Line 293 (expansion sketch comment):** update `(:wat::test::run-hermetic <body>)` to `(:wat::test::run-thread <body>)`.

## Substrate edits — NONE

No `src/` Rust changes. No edits to deftest-hermetic, run-thread, run-hermetic, run-thread-driver, or any other macro/function. Pure consumer-of-substrate edit at the deftest user surface.

## Constraints (HARD)

- DO NOT modify `:wat::test::deftest-hermetic` macro at line 326 — unchanged.
- DO NOT modify run-thread / run-hermetic / run-thread-driver / failure-from-thread-died / run-hermetic-driver families.
- DO NOT touch any test file (audit + decorate already settled the per-site choices).
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / the recovery doc / INTERSTITIAL.

## Scorecard (4 rows, YES/NO with grep/build/test evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | Deftest macro body expands to `(:wat::test::run-thread ~body)` | `awk '/defmacro/,/run-thread/' wat/test.wat | grep "run-thread ~body"` returns the line |
| B | Header doc-comment cites thread-default + FM 7-ter | grep for "thread" + "FM 7-ter" in the comment block above the macro |
| C | `cargo build --release --workspace --tests` clean | build output Finished, zero errors |
| D | Workspace failure count ≤ baseline (post-decorate: 12; rotation band 8-11) | `cargo test --release --workspace --no-fail-fast`: total failed ≤ 12 |

## STOP triggers

- Build fails after the edit → STOP; surface the error.
- Workspace failure count REGRESSES significantly (>15 failed) → STOP; the flip surfaced a real regression in thread mode. Investigate.
- The 5 decorated `deftest-hermetic` tests no longer pass → STOP; the decoration sweep was incomplete; surface which.

## Time-box

This is a one-line code change + 3 small doc comment edits + workspace test verification. Predicted ~10 min orchestrator-direct (no sonnet spawn — overhead exceeds the work scope). EXPECTATIONS sets the band.
