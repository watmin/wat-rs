# Arc 143 Slice 5b — Pre-handoff expectations

**Drafted 2026-05-02 (late evening).** Tiny gating fix.

**Brief:** `BRIEF-SLICE-5b.md`
**Output:** 1 Rust file modified (`src/runtime.rs`) + ~150-word
report.

## Setup — workspace state pre-spawn

- Slice 6 shipped Mode B with two gaps; this slice fixes Gap 1.
- Slice 6's tests at `tests/wat_arc143_define_alias.rs`:
  - test 1 (`define_alias_foldl_to_user_fold_delegates_correctly`):
    FAILS with Gap 1 error → should PASS after this slice
  - test 2 (`define_alias_length_to_user_size_delegates_correctly`):
    FAILS with Gap 2 error → still fails after this slice (slice 5c)
  - test 3 (error case): PASSES already
- Workspace state: 1 pre-existing arc 130 LRU failure +
  slice 6's test 1 (Gap 1) + slice 6's test 2 (Gap 2) = 3 failing.

## Hard scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | 1 Rust file modified (`src/runtime.rs`). NO other files. |
| 2 | New arm in `value_to_watast` | The match in `value_to_watast` (line 5882+) gains an arm: `Value::holon__HolonAST(h) => Ok(holon_to_watast(&h))` (or equivalent). |
| 3 | Unit test for new path | A test in `src/runtime.rs::tests` (or wherever value_to_watast is currently tested) that constructs a HolonAST Value, calls value_to_watast, asserts the converted WatAST. |
| 4 | Slice 6 foldl test transitions | `cargo test --release --test wat_arc143_define_alias define_alias_foldl_to_user_fold_delegates_correctly` PASSES (was failing with Gap 1; now resolves). |
| 5 | **`cargo test --release --workspace`** | Exit non-zero with EXACTLY: 1 pre-existing arc 130 LRU failure + 1 slice 6 length test (Gap 2) failure = 2 failures total. ZERO other regressions. |
| 6 | Honest report | 150-word report covers: the new arm verbatim, the unit test verbatim, slice 6 foldl test transition confirmation, test totals, honest deltas. |

**Hard verdict:** all 6 must pass. Row 4 is the load-bearing
end-to-end verification.

## Soft scorecard (2 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 7 | LOC budget | Total slice diff: 5-15 LOC. >30 LOC = re-evaluate. |
| 8 | holon_to_watast signature verified | Sonnet's report confirms `holon_to_watast` is in scope and takes `&HolonAST`. If not, surfaces alternative (e.g., a different conversion path). |

## Independent prediction

- **Most likely (~85%) — Mode A clean ship:** the fix is one line;
  the test is straightforward; the slice 6 foldl test transitions.
  Sonnet ships in ~3-7 min.
- **Surprise (~10%):** `holon_to_watast` has a slightly different
  signature than the brief assumes (e.g., returns `Result<WatAST,
  _>` instead of `WatAST`). Sonnet adapts; minor delta.
- **Slice 6 test still fails after fix (~3%):** the conversion works
  but the test has another issue. Surface; investigate.
- **New regression (~2%):** unlikely but possible — value_to_watast
  is widely-used; the new arm shouldn't affect existing paths.

**Time-box: 14 min cap (2× upper-bound 7 min).**

## Methodology

After sonnet returns:
1. Read this file FIRST.
2. Score each row.
3. Diff via `git diff --stat` — 1 Rust file expected.
4. Run `cargo test --release --test wat_arc143_define_alias` —
   confirm test 1 (foldl) transitions to PASS, test 2 (length)
   still fails with Gap 2.
5. Run `cargo test --release --workspace` — confirm 2 failures
   total (LRU + length); zero other regressions.
6. Score; commit `SCORE-SLICE-5b.md`.

## Why this slice matters

Slice 5b is the SMALLEST possible fix unblocking the largest
downstream chain:
- Slice 6's foldl alias works end-to-end after this
- Slice 7 (apply :reduce/:foldl + arc 130 substrate updates) becomes
  trivially executable
- Slice 5c (length scheme registration) is independent; runs in
  parallel
- After 5b + 5c + 7 ship, arc 143 is functionally complete; slice 8
  closes paperwork
