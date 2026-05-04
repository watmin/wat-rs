# Arc 148 Slice 3 — Pre-handoff expectations

**Drafted 2026-05-03.** Foundation slice — substrate ord-comparison
coverage extension + new test file. NO renames; NO architectural
change beyond extending `eval_compare`'s reach. Predicted SMALL-
MEDIUM slice (Mode A ~80%; Mode B-recursion-edge ~10%; Mode B-
unfindable-Value-variant ~5%; Mode C ~5%).

**Brief:** `BRIEF-SLICE-3.md`
**Output:** EDITS to `src/runtime.rs` (`eval_compare` extension or
new `values_compare` helper) + NEW `tests/wat_arc148_ord_buildout.rs`.

## Setup — workspace state pre-spawn

- Arc 148 slice 2 shipped (`SCORE-SLICE-2.md`); arithmetic per-Type
  leaves at `,2`-suffixed names; bare names freed for slice 4.
- Workspace baseline (per FM 9, post-slice-2): reflection-layer
  baselines all green (45/45 across 5 test files); workspace
  failure profile is the documented `CacheService.wat` noise.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EDITS to `src/runtime.rs` (`eval_compare` body or new `values_compare` helper) + NEW `tests/wat_arc148_ord_buildout.rs`. Possibly EDITS to imports if new test file pulls in test helpers. NO `src/check.rs` edits (this is purely runtime). NO `wat/` edits. NO retirement of any handler. |
| 2 | 8 new ord arms | Extend `eval_compare` to accept: `Instant`, `Duration`, `Bytes`, `Vec` (recursive), `Tuple` (recursive), `Option` (variant-ordered), `Result` (variant-ordered), `Vector` (element-wise). Existing arms (`i64`, `u8`, `f64`, mixed-numeric, `String`, `bool`, `keyword`) preserved. |
| 3 | Recursion correctness | `Vec`/`Tuple`/`Option`/`Result`/`Vector` recursion mirrors `values_equal` shape (`src/runtime.rs:4507-4548`). Recursion either uses a self-call or a helper-call structure that handles arbitrary depth without panicking on non-comparable element types. |
| 4 | Variant-order semantics | `Option`: `None < Some(_)`; `Some(x) cmp Some(y) = x cmp y`. `Result`: `Err < Ok`; same-variant comparisons recurse on payload. |
| 5 | Rejected types still raise | `HashMap`, `HashSet`, `Enum`, `Struct`, `unit`, `HolonAST` raise `TypeMismatch` when ord is attempted. The existing fall-through arm in `eval_compare` handles this (no new explicit rejection arms needed; just verify the fall-through still triggers for these). |
| 6 | New test file shipped | `tests/wat_arc148_ord_buildout.rs` exists with: 4 test cases per new ord-comparable type (`<`/`>`/`<=`/`>=`); 1 test case per rejected type asserting `TypeMismatch`; 2 test cases each for recursive types (shallow + deep). |
| 7 | All baseline tests still green | `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17; `wat_arc143_define_alias` 3/3; `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8; `wat_polymorphic_arithmetic` (existing). |
| 8 | New tests pass | All test cases in `tests/wat_arc148_ord_buildout.rs` pass. |
| 9 | Workspace failure profile unchanged | Pre-slice + post-slice both: only documented `CacheService.wat` noise. |
| 10 | Honest report | ~250-word report covers all required sections from BRIEF. |

**Hard verdict:** all 10 must pass. Rows 2 + 3 + 4 are the
load-bearing rows (substrate correctness for the new ord arms).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | Total slice diff: 150-400 LOC (substrate ~50-150 LOC + new test file ~100-250 LOC). >500 LOC = re-evaluate scope. |
| 12 | Style consistency | New ord arms mirror `values_equal`'s structure exactly (Vec uses `iter().cmp()`-equivalent; Tuple recurses element-wise; etc.). Test cases mirror `wat_polymorphic_arithmetic.rs`'s `run(src)` pattern. |
| 13 | clippy clean | No new warnings. |
| 14 | Audit-first discipline | If sonnet finds a recursive type that needs an Ord impl Rust doesn't provide on the substrate's Value enum, surface as STOP-at-first-red rather than improvising. If a Value variant the BRIEF lists doesn't exist or is named differently, surface in the report. |

## Independent prediction

- **Most likely (~80%) — Mode A clean ship.** Substrate work
  contained to one Rust file + one new test file. The recursion
  pattern is well-trodden via `values_equal`. ~30-45 min wall-
  clock.
- **Mode B-recursion-edge (~10%):** sonnet hits an issue with
  recursive comparison for one of the parametric types (e.g.,
  `Vec`'s element comparison panics on non-comparable elements
  instead of returning `TypeMismatch`). Surfaces as honest delta;
  orchestrator may revise scope or add a sentinel arm.
- **Mode B-unfindable-Value-variant (~5%):** the brief assumes
  `Value::Bytes` / `Value::wat__time__Instant` / etc. exist with
  those exact names; if naming differs, sonnet's grep finds the
  actual names and adapts.
- **Mode C (~5%):** hits an unforeseen substrate edge (e.g.,
  `eval_compare`'s structure forbids extension without major
  refactor).

## Time-box

60 min wall-clock (≈2× the predicted upper-bound of 30-45 min). If
the wakeup fires and sonnet hasn't completed: TaskStop + Mode B
score with the overrun as data.

## What sonnet's success unlocks

Slice 5 (numeric comparison migration) can retire
`infer_polymorphic_compare`'s non-numeric branch knowing the
substrate's ord coverage actually matches DESIGN's claim. The
universal-same-type-delegation rule becomes substantively true.

## After sonnet completes

- Re-read AUDIT's OQ1 against the SCORE
- Score the 10 hard rows + 4 soft rows
- Verify load-bearing rows (2, 3, 4) by spot-checking the new
  arms + running the new tests
- Write `SCORE-SLICE-3.md`
- Commit the SCORE before drafting slice 4's BRIEF (calibration
  preserved across compactions)
