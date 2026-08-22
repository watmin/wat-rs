# EXPECTATIONS — the `:peers` bijection keeps its negative controls

Written BEFORE the strike. Brief:
`BRIEF-STONE-the-peers-bijection-keeps-its-negative-controls.md`.

## The scorecard

| # | what | the command I will run | expected |
|---|---|---|---|
| 1 | five tests exist and are registered | `cargo nextest run --release -E 'binary_id(wat::services)'` lists them | five new test names appear |
| 2 | the suite is green | same | `binary_id(wat::services)` **133/133** (128 today + 5) |
| 3 | ★ case 3 accepts the `:-` form | its test | PASS |
| 4 | ★ cases 1·2·4·5 reject | their tests | PASS (each asserts an `Err`) |
| 5 | ★★ case 5 names the surface | read its golden `.edn` | the message contains `probe::Echo` |
| 6 | ★★ perturb-to-prove | rider reports both directions | tests RED with a bijection check stubbed to succeed; GREEN after revert |
| 7 | `wat/service.wat` untouched | `git diff wat/service.wat` | **empty** |
| 8 | no must-fail file under the gate | `git status --short` | nothing new under `wat-scripts/` |
| 9 | goldens are only the new ones | `git status --short` | no pre-existing `.edn` modified |
| 10 | the floor | `scripts/floor.sh` (mine, centrally) | **4859/4859**, 0 FAIL, 19 skipped |
| 11 | clippy | `--workspace --all-targets --release -- -D warnings` | 0 |

**Row 10's arithmetic is a prediction, not an observation.** 4854 + 5 = 4859. If the floor reports
anything else, the difference must be EXPLAINED before it is accepted — a count that moves for an
unexamined reason is the thing the seam's own floor note exists to catch.

## The rows that can lie

**Row 2 is not row 10.** A rider's scoped `binary_id(wat::services)` run was 128/128 green on a recent
stone while the floor was red by six, all in `binary_id(wat::kernel)`. Row 2 is the rider's cheap
check; row 10 is the measurement.

**Rows 3+4 together are still weak.** Four rejections and one acceptance are all satisfied by a
bijection that rejects on the `:peers` side and never reads the ephemeral side at all. **Row 5 is the
discriminator** — case 5 is the only case that fails on the *ephemeral* side, so only its message can
prove the structural reader extracted a name from the `:-` form. If row 5 passes but its golden does
not literally contain `probe::Echo`, rows 3 and 4 prove nothing.

**Row 6 is the one that decides whether any of this is a test.** Rows 1-5 confirm five files exist and
five assertions hold today. Only row 6 shows the suite can go RED for the defect it names. A negative
control that cannot fail is decoration. `[[feedback_a_green_test_can_prove_nothing]]`

**Row 9 exists because `UPDATE_EDN=1` rewrites every golden the selected tests touch**, including
already-passing ones, re-pretty-printing them. That has produced spurious diffs twice; both were
reverted. An unexpected `.edn` in `git status` is that, not a finding.

## Independent prediction

**Runtime: 25-40 minutes.** Five fixtures are `sed` off one base file, the driver copies a committed
exemplar, and every destination in the brief was run before it was written — so the discovery cost is
near zero and the work is nearly all mechanical. The two things that can stretch it: matching the
exemplar's golden-path resolution, and step 4's perturb-revert cycle, which needs two rebuilds of a
baked stdlib file.

**Trap-doors named in advance:**
- The rider stubs a bijection check for row 6 and **fails to revert it**. Row 7 catches this, and I
  re-run it myself; `git diff wat/service.wat` must be empty.
- The rider puts a must-fail fixture under `wat-scripts/` out of habit, taking the floor red on
  `every_wat_scripts_file_loads`. Row 8.
- The rider "fixes" a mismatch by relaxing an assertion to whatever it observed. STOP-3 forbids it;
  row 5's literal-substring check is the guard that a relaxed assertion cannot pass.
- Case 3 fails, meaning the `:-` form is not accepted where I measured that it is. That is STOP-1 and
  a substrate finding — not a test to adjust.

## Mode

- **Mode A** — all rows pass, `wat/service.wat` byte-identical, row 6 reported both directions.
- **Mode B** — ships, but row 6 unreported or one-directional, or a golden adjusted to match output.
- **Mode C** — a STOP fires. Ship nothing; the report is the deliverable.
