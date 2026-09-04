# EXPECTATIONS — an arm set for the branch the fire takes

> ⚠ **This strike MEASURES. It optimises nothing.** A report claiming the filter phase got faster is
> out of scope; a report claiming coverage it did not measure is worse.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,407 plus every arm you drive.**

## The scorecard — pre-values driven at HEAD `deecfac6e`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the taken branch has arms | ⛔ **none** — A/B/D/E/F all measure `exec_where`, called **0** times here | arms for `bind_view`, `candidates`, the two `HashSet` builds, the tid loop, `d_beta` pushes |
| 2 | ★ coverage is MEASURED and stated | arm C = **0.132 / 0.419 ms = 31.5%**; ~68.5% unmeasured | the new set's coverage **with its spread**, and what remains unaccounted named |
| 3 | ★ coverage is never ARRANGED | `F+C` = **656% accounted** (1 sample; C6's six: 684/693/734/723/686/698) | no tolerance chosen to make a check pass — STOP-3 |
| 4 | the pre-value is re-measured at six | **one** sample at 656%, outside C6's band | six samples before reasoning from any movement |
| 5 | the old arms keep their meaning | summed into a phase they do not run in | relabelled as the `exec_where` branch; `B-E` headroom preserved |
| 6 | each arm is wired to the number | — | mutation 1: zeroing an arm moves coverage by its share |
| 7 | the two ladders are distinguishable | — | mutation 2, or an honest STOP-1 saying why not without editing the fire |
| 8 | arms scale with the phase | — | mutation 3: halving tokens holds the fraction |
| 9 | no engine change | — | **zero diff outside `node_share_cost.rs`** |
| 10 | floor / lints / clippy | **`5407 tests run: 5407 passed (2 slow), 21 skipped`**, 0 FAIL, lints **258**, clippy rc=0 | ≥ 5407 + arms, 0 FAIL, lints ≥ 258, rc=0 |

## Runtime prediction

**70–100 minutes.** Wiring five arms into the existing harness is mechanical; deciding what the
reconstruction may honestly assert is the work. ⚠ Budget for release rebuilds — the last strike ran
110 min against a 50–80 estimate because the mutation matrix needed five of them.

## Trap doors named in advance

- **⛔ THE ARMS MUST NOT BE SCALED TO `evals_per_round`.** That constant is a *pre-where-tree* eval
  count and is exactly what made the old ladder stale (`node_share_cost.rs:382`). The taken branch
  runs **per token**, not per eval.
- **Over-accounting is the same defect as under-accounting.** STOP-2 at ~120%.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.
- **One sample is not a shift.** Row 4.

## What would make this strike a failure even if every test passes

**Asserting a reconstruction that was arranged rather than measured.** C6 measured the declared check
failing at ~7x and **refused to assert it** — that refusal is the standard this strike is held to. An
arm set that lands nearer and is asserted on that basis is the same unfalsifiable claim C8 was opened
for, wearing a better number.

**And building arms nobody can tell apart from the old ones.** If the new ladder cannot be shown to
track the taken branch while A/B/D/E/F track the other, then there are eleven arms and still no
evidence about which branch the phase is in. Row 7 is what prevents that.
