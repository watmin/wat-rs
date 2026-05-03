# Arc 146 Slice 3 — Pre-handoff expectations

**Drafted 2026-05-03.** BUNDLED migration: 4 dispatches +
10 per-Type impls + 4 retirements. Predicted MEDIUM slice
(Mode A ~60%; Mode B-multi-arg-pattern ~15%; Mode B-return-type-
varies ~10%; Mode B-arc144-test ~10%; Mode C ~5%).

**Brief:** `BRIEF-SLICE-3.md`
**Output:** EDITS to src/runtime.rs + src/check.rs + wat/core.wat +
tests/wat_arc144_hardcoded_primitives.rs (Q2 updates). NO new
test files. ~400-700 LOC.

## Setup — workspace state pre-spawn

- Slice 1 closed; slice 1b rename closed; slice 2 closed (length
  migrated; canary GREEN; substrate completions for dispatch
  mechanism shipped).
- Workspace baseline (FM 9): all green except CacheService.wat
  noise.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to runtime.rs + check.rs + wat/core.wat + tests/wat_arc144_hardcoded_primitives.rs. No new test files; no other Rust changes. |
| 2 | 10 per-Type impls | 3 empty? + 3 contains? + 2 get + 2 conj. Each follows slice 2's inner-helper-plus-eval-wrapper shape. |
| 3 | 10 dispatch arms | All 10 added to dispatch_keyword_head's switch (BELOW the dispatch_registry guard). |
| 4 | 10 TypeScheme registrations | All 10 in register_builtins, adjacent to slice 2's length registrations. Use existing helpers (vec_of, hashmap_of, option_of, etc.). |
| 5 | 4 dispatch declarations in wat/core.wat | empty?, contains? (mixed verbs), get (2 arms), conj (2 arms). |
| 6 | 4 sets of old machinery RETIRED | Each: eval_* fn + dispatch arm + infer_* fn + dispatch arm + arc 144 fingerprint. 16 retirement targets total (4 fns × 4 + 4 fingerprints). |
| 7 | arc 144 hardcoded_primitives tests | Q2 Option A pattern applied to any test asserting the retired primitives' fingerprint shapes. Result: 17/17 still pass. |
| 8 | All baseline tests pass | wat_arc146_dispatch_mechanism 7/7; wat_arc144_lookup_form 9/9; wat_arc144_special_forms 9/9; wat_arc143_lookup 11/11; wat_arc143_manipulation 8/8; wat_arc143_define_alias 3/3 (length canary stays green). |
| 9 | Workspace failure profile | UNCHANGED from post-slice-2 (only CacheService.wat noise). |
| 10 | Honest report | ~400-word report covers all sections; Q1-Q3 decisions named. |

**Hard verdict:** all 10 must pass. Row 8 is load-bearing — these
4 migrations must not break the proven substrate.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget (400-700) | Total slice diff. >900 = re-evaluate scope. |
| 12 | Style consistency | Each migration mirrors slice 2's pattern; helpers reused; comment style consistent with arc 146 commit messages. |
| 13 | clippy clean | 40 → 40 warnings. |
| 14 | Audit-first discipline | If Q1 (multi-arg pattern grammar) or Q2 (return-type unification) reveal slice 1 gaps, surface clean diagnostic. Don't ship workarounds. |

## Independent prediction

- **Most likely (~60%) — Mode A clean ship.** Mechanism proven;
  pattern mechanical; substrate-completion deltas already fixed
  in slice 2. ~25-40 min wall-clock (4× slice 2's per-migration
  work, but parallel scope so overhead is amortized).
- **Surprise on multi-arg pattern grammar (~15%) — Mode B-Q1.**
  Slice 1 may have only tested 1-arg patterns. If sonnet's Q1
  audit finds the parser rejects multi-arg arms, surface as
  slice 1 substrate-completion gap; orchestrator opens a
  slice 3-pre to fill it.
- **Surprise on get return type (~10%) — Mode B-Q2.** infer_dispatch_call
  may need adaptation for per-arm return-type variance.
- **Surprise on arc 144 hardcoded_primitives test (~10%) —
  Mode B-arc144-test.** Q2 updates may be more invasive than
  expected; sonnet adapts.
- **Borrow / Rust friction (~5%) — Mode C.** Adapts.

**Time-box: 80 min cap (2× upper-bound 40 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 4 file changes expected (3 modified, 1 wat append + tests update).
4. Read 2-3 of the 10 new per-Type impls.
5. Read all 4 dispatch declarations in wat/core.wat.
6. Verify retirements via `git diff src/runtime.rs src/check.rs`
   — confirm 4 eval_* + 4 infer_* + dispatch arms + 4 fingerprints
   are GONE.
7. Run all baseline tests + workspace + clippy.
8. Score; commit `SCORE-SLICE-3.md`.

## What this slice unblocks

- **Slice 4** — pure rename family (5 renames; no dispatch needed;
  smaller scope).
- **Slice 5** — closure paperwork.
- **Arc 144 slice 4** — verification simpler post-arc-146-slice-3
  (the polymorphic primitives all behave consistently via dispatch).
- **Arc 130 RELAND v2** — accessible after arc 146 closes.

User's finish line: **every defined symbol can be queried at
runtime.** Slice 3 closes 4/5 of the remaining gap. Slice 4
closes the last 5 (renames). Then arc 146 closes; foundation
strengthens by 9 properly-defined primitives + 4 substrate-
completion deltas + 1 entity kind (Dispatch).

The mechanism is the proof; this slice is the rhythm.
