# EXPECTATIONS — measuring and budgeting the rete cohort's nine

| what | command | expected |
|---|---|---|
| the nine share the measured test's binary | `cargo nextest list -E 'test(accum_leftover_split)'` etc. | all report `wat` (the lib binary), **not** `wat::rete` |
| nine alone-times, six samples each, minimum taken | `cargo test --release <name> -- --exact` ×6 | a number per test, spread reported |
| loaded times, one floor | `scripts/floor.sh` then read `.floor/latest/clean.log` | a duration per test |
| contention per test | loaded ÷ alone | inside or outside the recorded **3.5x–4.4x** band — say which |
| the verdict is arithmetic, not judgment | `alone × 4.4` vs **30s** | a per-test over/under, tabulated |
| only earning names added | `git diff .config/nextest.toml` | every added name has a row above it with its three numbers |
| order preserved | `git diff .config/nextest.toml` | the rete-cohort rule still sits **below** `binary_id(wat::lint)`; no new block above it |
| all three profiles | `grep -c 'accum_leftover_split' .config/nextest.toml` (per added name) | 3 — default, ci, slow |
| config still parses | `cargo nextest list > /dev/null` | rc=0 |
| **MUTATION — the budget is reachable** | drop the rete-cohort `period` to `1s`, run one added test | prints `SLOW [> 1.000s]`; restore, silent. ⚠ Isolation proves nothing here — the file records that the naive check was tried and failed, because alone-times sit under even the old warn. **The loaded cost is the only number the kill sees.** |
| floor | `scripts/floor.sh` | 5480, **0 fail** |

## Runtime prediction

**60–90 min**, dominated by 54 alone-runs plus two floors. The measurement is the strike; the edit
is ten minutes.

## Trap doors

- **Adding all nine because nine is tidy.** The contract decision. A name in that filter without a
  number beside it makes the whole cohort's derivation unfalsifiable — and this file has twice
  written *"⛔ RAISING A NUMBER IS NOT A FIX"* in its own margins.
- **Trusting the projection over the measurement.** Two green floors this session did not trip any
  of the nine. The 30–53s figure is `alone × band`, and the band was derived on a ~220s floor while
  this box now floors at ~452s. **Measure.**
- **Three samples.** The existing rows carry six with the spread. Three cannot resolve a 4x
  contention claim.
- **A new override block.** First-match wins and the file warns three times. Extend the existing
  alternation.
- **Forgetting `[profile.ci]` and `[profile.slow]`.** Every other cohort is mirrored into all three,
  and the file gives the reason each time.
