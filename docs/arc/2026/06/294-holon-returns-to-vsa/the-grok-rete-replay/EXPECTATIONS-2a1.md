# EXPECTATIONS 2a1 — written BEFORE the strike

| # | what | check | expected |
|---|---|---|---|
| E1 | D1: registration survives a stale body | the D1 unit test on sift_rules and W2f | every enum each file declares registers, fields in declaration order |
| E2 | D1: the cost of one call | the D1 test's timing, on sift_rules' forms | milliseconds, reported (an `eval-with-defs!` turn is ~460 ms) |
| E3 | D2: the snapshot is built once and cannot deadlock | the D2 test (startup capture with its measured delta, OR the no-re-entry test for a lazy build) | pass; the delta or the proof reported |
| E4 | D3: the verb exists, documented like its siblings | read the verb | pure, no `!`; `@arg`/`@ret`/`@example`; reuses `eval_type_of`'s construction |
| E5 | D4: cases 1–7 | `cargo nextest run --release -E 'test(<the D4 tests>)'` | all pass |
| E6 | D4: the oracle, case 8 | same | the verb's answer = `type-of` after `startup_from_source`, for each case that loads |
| E7 | isolation | D4 case 6 | two same-named types in two calls: each call sees only its own |
| E8 | the tests can fail | break the copy (reuse ONE copy across two calls, in a scratch edit), run case 6, restore | case 6 goes RED |
| E9 | walls green | `scripts/floor.sh` · clippy | 0 failed · 0 lines |

**Runtime prediction:** 2–4 h. Most of it is D1/D2: finding the exact user half of `build_env`, and
getting the snapshot without a re-entrant lock.

**Trap doors, named in advance:**
- **Calling `build_env(vec![])` lazily from inside the verb without proving no re-entry.** That is
  `src/runtime.rs:10294`'s deadlock.
- **A second `TypeInfo` construction.** Two implementations of one slot drift.
- **Swallowing a failed declaration as "no types".** That is the silent drop the refusal exists to
  prevent.
