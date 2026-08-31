# EXPECTATIONS — rete agrees with core

> Written **before** the strike. Scored against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | control passes before | `cargo nextest run --release -E 'test(a_real_enum_variant)'` | **1 passed**, fixture prints `1` |
| 2 | arm 1 RED before | `cargo nextest run --release --no-capture -E 'test(a_misspelled_enum)'` | **FAIL** — `SILENT WRONG ANSWER … printed "0" with exit 0` |
| 3 | arm 2 RED before | the fixture you build | **FAIL** — bare tagged variant prints `0`, exit 0 |
| 4 | both arms GREEN after | as rows 2–3 | passed, each via a **located refusal**, not a changed count |
| 5 | control STILL `1` | as row 1 | `1`. **A control that drops to 0 means legitimate enum constraints now refuse** — worse than the defect |
| 6 | agreement with core, arm 1 | drive `(:wat::core::= :evt::G::Hii :evt::G::Hi)` and the rete rule | both refuse, both at check time, both naming the type disagreement |
| 7 | agreement with core, arm 2 | same for the tagged pair | both refuse |
| 8 | blast radius | `git diff --stat` | `validate/typing.rs` + the probe files. A second `src/` file is a STOP |
| 9 | the floor | `./scripts/floor.sh`, Summary from the captured log | **5,186 / 5,186** (5,183 + control + 2 arms), 21 skipped, exit 0 |
| 10 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — one per arm

- drop the **arity-0** requirement → **arm 2 alone** reddens; restore
- restore the **hand-rolled prefix** resolution → **arm 1 alone** reddens; restore

If a mutation reddens nothing, that is a coverage finding, not a null result.

## Runtime prediction

35–50 minutes. Two or three release builds; the fix is a dozen lines in one function, and most of
the work is arm 2's fixture and the two side-by-side core comparisons.

## Trap doors named in advance — with the step

- **Row 5 is the one that can be silently lost.** Requiring arity 0 could over-refuse and break
  legitimate unit-variant constraints. **Step:** the control must still print `1` — read the
  number, do not infer it from a green test.
- **A refusal that is red for the wrong reason.** Rows 6–7 exist because "it refuses now" is not
  the bar; agreement with core is. **Step:** drive both engines and quote both messages.
- **Arm 2 may be hard to route down the tagged path.** **Step:** if it is, say so and name it
  unproven rather than declaring the arm fixed — the helper accepts tagged variants, so a green
  arm 2 without a driven fixture proves nothing.

## What would make this a failure even if every test passes

A refusal whose reason or phase differs from core's. That replaces one divergence with another and
leaves the arc's stated contract — agreement between the two engines — still broken.
