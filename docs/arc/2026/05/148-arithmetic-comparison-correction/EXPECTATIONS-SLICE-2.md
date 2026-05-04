# Arc 148 Slice 2 — Pre-handoff expectations

**Drafted 2026-05-03.** Foundation slice — pure mechanical rename
of 8 substrate primitives + call-site sweep. NO new entities; NO
architectural change. Predicted MEDIUM slice (Mode A ~75%; Mode B
-missed-call-site ~15%; Mode B-substrate-coupling-surprise ~10%).

**Brief:** `BRIEF-SLICE-2.md`
**Output:** EDITS to `src/runtime.rs` + `src/check.rs` + an
unknown number of test/wat files (slice 1 audit didn't enumerate
call sites; sonnet's grep finds them). NO new files.

## Setup — workspace state pre-spawn

- Arc 148 DESIGN updated 2026-05-03 with audit resolutions (OQ1,
  OQ2, OQ3) + revised slice plan (slices 2-6).
- Arc 148 slice 1 audit shipped (`AUDIT-SLICE-1.md`).
- 1 in-flight uncommitted file (`crates/wat-lru/wat-tests/lru/CacheService.wat`
  — arc 130 noise; ignore).
- Workspace baseline (per FM 9 baseline check 2026-05-03):
  reflection-layer baselines all green (45/45 across 5 test files);
  workspace failure profile is the documented CacheService.wat noise.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to `src/runtime.rs` (`eval_i64_arith` arms + `eval_f64_arith` arms + freeze pipeline list) + `src/check.rs` (TypeScheme registrations) + test/wat files containing call sites. NO new Rust files. NO new wat files. NO new test files. NO modifications to `eval_*` BODIES — only registration names + caller names. |
| 2 | 8 renames performed | All 8 names listed in BRIEF: `:wat::core::i64::{+,-,*,/}` and `:wat::core::f64::{+,-,*,/}` → add `,2` suffix. Each visible at the new name; old name no longer registered. |
| 3 | TypeScheme registrations updated | 8 entries in `src/check.rs:8718-8750` use the new `,2`-suffixed names. |
| 4 | Freeze pipeline pure-redex list updated | `src/runtime.rs:15605-15641` lists the new `,2` names; old names removed. |
| 5 | Call-site sweep complete | `grep -rn ":wat::core::i64::[+\-*/]"` and same for f64 returns ZERO matches outside of the registration sites + retirement-marking comments. (Or matches are explicitly accounted for in the report — e.g., comments in INSCRIPTION-style docs that intentionally cite the old name.) |
| 6 | All baseline tests still green | `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8. |
| 7 | Full workspace `cargo test` passes | All tests across `wat-rs/` and its `crates/` pass. (Fails surfaced as honest deltas if any.) |
| 8 | No new clippy warnings | `cargo clippy` count unchanged from pre-slice baseline. |
| 9 | Workspace failure profile unchanged | Pre-slice: only the documented `CacheService.wat` noise. Post-slice: same. |
| 10 | Honest report | ~250-word report covers all required sections from BRIEF. Total renames (8), total call sites updated (count), files touched (list), workspace state, honest deltas. |

**Hard verdict:** all 10 must pass. Rows 5 + 6 + 7 are the
load-bearing rows (sweep completeness + tests still passing).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 50-300 LOC (mostly in tests/wat call sites; the registration changes are 8 × 3 lines each ≈ 24 LOC in src/). >500 LOC = sweep found unexpected scope. |
| 12 | Style consistency | Renames are exact char-substitutions (`:wat::core::i64::+` → `:wat::core::i64::+,2`); no incidental refactoring. Test fixtures match the same pattern. |
| 13 | No phantom citations | Every file:line in the report is verifiable. |
| 14 | Audit-first discipline | If sonnet finds a call site that can't be mechanically renamed (e.g., dynamic name construction), surface as STOP-at-first-red rather than improvising. |

## Independent prediction

- **Most likely (~75%) — Mode A clean ship.** Mechanical rename;
  brief is detailed; scope contained to one rename pattern.
  Predicted ~30-45 min wall-clock — most time in the call-site
  grep + sweep, not in the substrate edits.
- **Mode B-missed-call-site (~15%):** sonnet's grep misses a call
  site (e.g., one buried in a doc string or a Cargo.toml doctest);
  workspace test fails post-sweep with "no such symbol". Surfaces
  as honest delta; orchestrator finds + fixes.
- **Mode B-substrate-coupling-surprise (~10%):** the substrate has
  hidden coupling at one of the renamed names (e.g., a hardcoded
  string check somewhere besides the audit's enumerated sites).
  Sonnet hits the failing test; surfaces with file:line of the
  unexpected coupling.

## Time-box

60 min wall-clock (≈2× the predicted upper-bound of 30-45 min). If
the wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with the overrun as data.

## What sonnet's success unlocks

Slice 4 (numeric arithmetic migration) can place variadic wat
function wrappers at the freed bare names. Slice 3 (values_compare
buildout) is independent — can run in parallel or sequential, your
call.

## After sonnet completes

- Re-read the audit's OQ2 against the SCORE
- Score the 10 hard rows + 4 soft rows
- Verify load-bearing rows (5, 6, 7) by re-running cargo test
- Write `SCORE-SLICE-2.md`
- Commit the SCORE before drafting slice 3's BRIEF (so calibration
  preserved across compactions)
