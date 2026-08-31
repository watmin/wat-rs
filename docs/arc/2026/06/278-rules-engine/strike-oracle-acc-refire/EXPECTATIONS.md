# EXPECTATIONS — the oracle accretes superseded accumulate results

| # | what | expected |
|---|---|---|
| 1 | probes RED before | oracle **3** on two-changes where Clara says **1**; oracle **2** on one-change |
| 2 | Clara re-run by the rider | `Tally count = 1  values = [2]` — **not taken from DESIGN on trust** |
| 3 | always-empty still emits | all engines **1** (`n=0`) before AND after — the regression guard |
| 4 | oracle GREEN after, one change | **1**, value `n=1` |
| 5 | oracle GREEN after, two changes | **1**, value `n=2` |
| 6 | native untouched | `git diff` shows **no `src/` file** |
| 7 | termination test | if `length ==` was replaced, the report says so and proves the fixpoint still terminates |
| 8 | blast radius | ≤ 2 oracle `.wat` files + the probe. A third is a STOP |
| 9 | the floor | **5,188 / 5,188** (5,186 + 2 probes), 21 skipped, exit 0 |
| 10 | clippy | silent, exit 0 |

## The mutation proof

Restore the monotone `merge-facts` → **the two changing shapes redden, the always-empty one stays
green**. That asymmetry is the proof the fix is aimed at supersession and not at emission.

If a mutation reddens nothing, that is a coverage finding, not a null result.

## Trap doors named in advance — with the step

- **The termination test is a length comparison.** Retraction can hold the length equal while the
  set changes → false fixpoint, silent wrong answer. **Step:** replace it with a set comparison and
  prove termination on all three probes plus the existing oracle differentials.
- **Over-correcting into "never emit".** Would pass rows 4–5 and break row 3. **Step:** run the
  always-empty probe explicitly and read the number.
- **`derived_exists_and_acc_spec_matches_native` passes today because its fixture has a `where`
  fence** that hides this. **Step:** it must still pass, and the report should say whether removing
  its fence now also passes — that is the real regression signal.
- **The oracle is 2,164 lines nobody has changed this way before.** Scope is a genuine unknown.

## What would make this a failure even if every test passes

The oracle agreeing with Clara because it was made to mirror native. That converts an independent
reference into a copy, and every differential taken against it afterwards proves nothing.
