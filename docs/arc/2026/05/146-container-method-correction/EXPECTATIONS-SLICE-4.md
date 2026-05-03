# Arc 146 Slice 4 — Pre-handoff expectations

**Drafted 2026-05-03.** Last migration slice. 5 alias migrations
+ per-Type impls + retirements. Predicted SMALL slice (Mode A
~75%; Mode B-arc144-test ~10%; Mode B-load-order ~10%; Mode C
~5%).

**Brief:** `BRIEF-SLICE-4.md`
**Output:** EDITS to runtime.rs + check.rs + stdlib.rs + NEW
wat/core-aliases.wat. NO new test files. ~250-450 LOC + report.

## Setup — workspace state pre-spawn

- Slice 3 closed (5/10 primitives migrated to dispatch).
- All baselines green except CacheService.wat noise.
- Slice 1/2/3 substrate completions in place.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to src/runtime.rs + src/check.rs + src/stdlib.rs + NEW wat/core-aliases.wat. NO new test files. |
| 2 | 5 per-Type impls | HashMap/assoc/dissoc/keys/values + Vector/concat. Each with inner helper + eval wrapper. |
| 3 | 5 dispatch arms + dispatch_substrate_impl entries | All 5 added. |
| 4 | 5 TypeScheme registrations | In register_builtins; reuse existing helpers (hashmap_of, vec_of, etc.). |
| 5 | NEW wat/core-aliases.wat | Header + 5 define-alias declarations. |
| 6 | wat/core-aliases.wat registered | AFTER wat/runtime.wat in STDLIB_FILES. |
| 7 | 5 sets of old machinery RETIRED | 25 retirement targets total: 5 eval_* fns + 5 eval-arms + 5 infer_* fns + 5 infer-arms + 5 arc 144 fingerprints. |
| 8 | All baseline tests pass | wat_arc146_dispatch_mechanism 7/7; wat_arc144_lookup_form 9/9; wat_arc144_special_forms 9/9; wat_arc144_hardcoded_primitives 17/17 (with Q3 updates if needed); wat_arc143_lookup 11/11; wat_arc143_manipulation 8/8; wat_arc143_define_alias 3/3. |
| 9 | Workspace failure profile UNCHANGED | Only CacheService.wat noise. |
| 10 | Honest report | All sections; Q1-Q3 decisions; deltas surfaced. |

**Hard verdict:** all 10 must pass. Row 8 is load-bearing.

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget (250-450) | Net (after retirements). |
| 12 | Style consistency | Mirrors slice 2/3 patterns; alias declarations mirror wat/list.wat shape. |
| 13 | clippy clean | 40 → 40. |
| 14 | Audit-first discipline | Q1-Q3 surfaced cleanly; deltas noted. |

## Independent prediction

- **Most likely (~75%) — Mode A clean ship.** Smallest migration
  slice. Mechanism + substrate completions all in place.
  ~10-20 min wall-clock.
- **Mode B-arc144-test (~10%):** alias-shape vs dispatch-shape
  for signature-of returns; sonnet adapts via Q3.
- **Mode B-load-order (~10%):** alias declarations need careful
  load-order (after wat/runtime.wat); should be straightforward
  given slice 1/2/3 pattern.
- **Mode C (~5%):** Rust friction.

**Time-box: 40 min cap (2× upper-bound 20 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat`.
4. Read 1-2 of the 5 new per-Type impls.
5. Read wat/core-aliases.wat.
6. Run all baselines + workspace + clippy.
7. Score; commit `SCORE-SLICE-4.md`.

## What this slice unblocks

- **Slice 5** — closure paperwork (INSCRIPTION + 058 + USER-GUIDE).
- **Arc 144 slice 4** — verification simpler post-arc-146 close.
- **Arc 130 RELAND v2** — accessible.
- **Arc 109 v1 closure** — one arc closer.

User's finish line: **every defined symbol queryable at
runtime.** Slice 4 closes the LAST 5 of 10 originally-violating
primitives. After this slice: 10/10 ✅.
