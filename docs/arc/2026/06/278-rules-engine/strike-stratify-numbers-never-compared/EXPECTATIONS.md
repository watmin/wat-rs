# EXPECTATIONS — L2-3, written BEFORE the strike

## Scorecard

| what | command | expected |
|---|---|---|
| the oracle number, re-confirmed on the executor's binary | `wat wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat` | ANCHOR `{"l23::Ok2" 1}`; MEASURE `{}` |
| the native number for the bag set | the new in-crate test, printed or asserted | **PREDICTION: `Tally` ⇒ 1.** A refutation here is a valid, reportable result |
| the native ANCHOR (negation) | same test | a raised stratum — if this is empty the native instrument is inert and no reading below counts |
| key spelling agrees between engines | the test reports both sides' raw keys | identical strings, or STOP-2 |
| `derived_exists_acc` still green | `cargo nextest run --release -E 'test(derived_exists_acc)'` | 3/3 pass, untouched |
| floor | `scripts/floor.sh` | 5472 + the new test(s), **0 fail** |
| clippy | `cargo clippy --release --all-targets` | rc=0 |

## Runtime prediction

30–50 min. Most of it is constructing `StratifyView`s the way `fire/rules.rs` does; the assertion
itself is small.

## Trap doors, named in advance

- **Key spelling.** The oracle prints `l23::Ok2` — colon-free FQDN. If the native's `produced`
  strings are spelled differently, a naive compare reports a divergence that is not one. This is
  STOP-2 and it is the most likely way this strike produces a false positive.
- **A green that proves nothing.** If the chosen rule set puts every type at stratum 0 on both
  sides, both maps are `{}` and the test passes while seeing nothing. The negation anchor exists
  to refuse exactly that, and it belongs IN the test file, not in this document.
- **The behavioural fixture is green.** `derived_exists_acc` already asserts the two engines agree
  on facts for this shape. If the strata differ while facts agree, the finding is a **false
  lockstep claim in two headers**, not a correctness defect — say so plainly rather than inflating
  it. That is reading 1 in the DESIGN and it is the likelier of the two.
- **A stale binary answers with no error.** `wat/` is `include_str!`'d. If the scratch `.wat`
  disagrees with the numbers in the DESIGN, suspect the binary before the engine:
  `cargo install --path . --force` from `wat-rs/`. This already cost the orchestrator one retracted
  measurement in this strike.
