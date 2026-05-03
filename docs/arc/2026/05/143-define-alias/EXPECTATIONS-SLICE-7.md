# Arc 143 Slice 7 — Pre-handoff expectations

**Drafted 2026-05-02 (late evening)** in parallel with slice 6.
Trivial slice; depends on slice 6 Mode A.

**Brief:** `BRIEF-SLICE-7.md`
**Output:** 1 NEW wat file + 1 modified `src/stdlib.rs` + 2 substrate
call-site keyword changes + ~150-word report.

## Setup — workspace state pre-spawn

- Slice 6 shipped (Mode A): `wat/runtime.wat` exists,
  `:wat::runtime::define-alias` registered as a defmacro.
- The arc 130 RELAND v1 stepping stone test
  (`deftest_wat_lru_test_lru_raw_send_no_recv`) currently FAILS
  with "unknown function: :wat::core::reduce" at
  `crates/wat-lru/wat/lru/CacheService.wat:213`.
- This is the LAST substantive slice before closure. After it, slice
  8 (paperwork) wraps the arc.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | 1 NEW (`wat/list.wat`) + 1 modified (`src/stdlib.rs`) + 2 substrate keyword changes (`crates/wat-lru/wat/lru/CacheService.wat:213` + `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:251`). NO new test files. NO Rust changes outside stdlib.rs. NO files in `wat/std/`. |
| 2 | `wat/list.wat` exists | NEW file at `wat/list.wat` containing header comment + the line `(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)`. ~15 LOC total. |
| 3 | `src/stdlib.rs` registers list.wat | Entry mirrors slice 6's wat/runtime.wat registration. Load order: `runtime.wat` BEFORE `list.wat` (the alias depends on the macro). |
| 4 | Substrate call sites updated | Both `crates/wat-lru/wat/lru/CacheService.wat:213` and `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat:251` use `:wat::list::reduce` instead of `:wat::core::reduce`. Verifiable via grep. |
| 5 | **Arc 130 stepping stone TRANSITIONS** | `deftest_wat_lru_test_lru_raw_send_no_recv` either PASSES (Mode A — substrate Get path runs to completion) OR fails with a DIFFERENT error message (Mode B — surfaces the next gap in arc 130's chain). Either way, the "unknown function: :wat::core::reduce" error is GONE. |
| 6 | **`cargo test --release --workspace`** | Either: exit=0 (the previously-failing test now passes) OR exit non-zero with the SAME test failing for a DIFFERENT reason. ZERO new regressions in any other test. |
| 7 | No `wat/std/` additions | `git diff --stat -- wat/std/` shows no changes. |
| 8 | Honest report | 150-word report includes: wat/list.wat content, stdlib.rs registration + load-order confirmation, both call-site update confirmations, the EXACT new error message (or "ok" if passing), test totals, honest deltas. |

**Hard verdict:** all 8 must pass. Row 5 is the load-bearing
end-to-end test of arc 143's chain.

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC budget | wat/list.wat ~15 LOC; src/stdlib.rs +1 entry; 2 keyword changes. Total slice diff: 20-30 LOC. |
| 10 | Commit-ready cleanliness | The diff reads as a clean "alias ships + applies" change with no unrelated edits. |
| 11 | Forward-looking namespace | The alias name is `:wat::list::reduce` (forward-looking), not `:wat::core::reduce` (matches foldl's current namespace). The brief and DESIGN both specify this; verify sonnet didn't drift. |

## Independent prediction

- **Most likely (~70%) — Mode A:** the alias resolves; the
  CacheService Get path runs through; the test transitions to PASS.
  The full arc 143 chain held end-to-end. ~5-10 min runtime.

- **Mode B-different-failure (~20%):** the unknown-function failure
  is gone; the test fails for a different reason (probably
  reply-tx-disconnected — another arc 130 substrate issue NOT in
  arc 143's scope). Successful run; surfaces the next chain link
  for arc 130 to address.

- **Mode C-load-order issue (~5%):** wat/list.wat loads BEFORE
  wat/runtime.wat; the macro isn't registered when the alias
  application runs. Sonnet surfaces; orchestrator re-orders the
  stdlib registration.

- **Mode D-macro-expansion-issue (~5%):** the macro expands but the
  emitted define has a bug (FQDN gap, type-checker rejection, etc.).
  Surface; reland slice 6 with sharper brief.

## Methodology

After sonnet returns:

1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat`.
4. Read `wat/list.wat` directly.
5. Run `cargo test --release --workspace` locally; verify the
   stepping-stone transition.
6. If Mode A: arc 143 substantive work is COMPLETE. Slice 8 runs the
   end-of-work ritual + closure paperwork.
7. If Mode B-different: arc 143 still successful; arc 130 has its
   own next link.
8. Score; commit `SCORE-SLICE-7.md`.

## Why this slice matters

After slice 7 ships, arc 143's reason-for-being is fulfilled: the
substrate-as-teacher cascade end-to-end demonstrated, the bias to
reach for `reduce` honored, the reflection foundation in active use.

Slice 8 closes paperwork (INSCRIPTION + 058 row + USER-GUIDE entry +
CONVENTIONS additions if needed). The end-of-work ritual fires:
"did we learn anything in this arc that future-me shouldn't forget?"
