# EXPECTATIONS — `.wat.bad` becomes an enforced claim

> ⛔ **This scorecard REPLACES one measured with the wrong driver.** The superseded version asserted
> *"17 fail with MainSignatureError, 200 tests unfalsifiable"* — an artifact of driving
> `./target/release/wat`, which requires a `:user::main` because it EVALS one. The tests use
> `startup_from_file`, which does not. Every pre-value below was re-driven through that.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,376 plus every arm you drive.**

## The scorecard — every pre-value driven at HEAD `beb0c9554` via `startup_from_file`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ every `.wat.bad` actually fails at startup | ⛔ **16 of 281 return `Ok`** | **0**, enforced by a gate |
| 2 | ★ the 16 renamed to `.wat` | 16 mis-named | renamed, every `.rs` referrer updated, each classified by reading its test |
| 3 | ★ the gate is discovered, not listed | no gate reads this corpus for failure | walks `tests/`+`wat-scripts/`+`docs/`; population **281** or STOP-3 |
| 4 | the gate REDs on a regression | — | mutation 1: renaming a passing fixture back REDs, naming it |
| 5 | a renamed fixture stays load-bearing | — | mutation 2: breaking it REDs its own test |
| 6 | ★ the gate cannot pass vacuously | — | mutation 3: an empty population FAILS, not passes |
| 7 | the rowed mechanism, honestly scoped | **`MainSignature` = 2**, both `wat_arc170_slice_1e_user_main_nil_*`, whose subject IS the main signature | unchanged — **left alone, and said so** |
| 8 | no fixture needs a `:user::main` added | — | **zero mains added.** A fixture needs one only if the binary will eval it |
| 9 | floor / lints / clippy | **`5376 tests run: 5376 passed, 21 skipped`** (425.7 s, 0 FAIL), lints **228**, clippy rc=0 | ≥ 5376 + arms, 0 FAIL, lints ≥ 228, rc=0 |
| 10 | `src/` | — | **zero diff, index AND worktree** |

## Runtime prediction

**60–90 minutes.** The gate is short; classifying and renaming the 16 with their referrers is the work.

## Trap doors named in advance

- **⛔ THE DRIVER IS THE WHOLE QUESTION.** `./target/release/wat <file>` and `startup_from_file` give
  opposite verdicts on these same 16 files. Measure through the one the tests use, or you will
  rewrite this strike as the orchestrator did.
- **A rename moves a file between gate populations.** `.wat` under `wat-scripts/` is read by two
  gates (parse+type-check, rete-name resolution). Confirm where the 16 live before renaming.
- **Do not truncate output.** An earlier cut of this probe sliced to 400 chars and lost a fixture
  whose error sits at char 4441.
- **`git checkout <sha> -- <path>` STAGES.** Verify restores by hash, never `git diff`.

## What would make this strike a failure even if every test passes

**Adding a `:user::main` to any fixture.** That was the superseded design's prescription and it is
now a defect: the test runners construct a world and invoke at will, so a main is required only when
the binary evals. Row 8 exists to catch a rider that inherits the wrong draft.

**And a gate that passes on an empty population.** Row 6. A discovered gate that finds nothing and
reports success is precisely the defect this arc keeps re-finding — including in the strike that
landed yesterday.
