# BRIEF — measure the nine, then budget the ones that need it

## The work

`.config/nextest.toml`'s rete-cohort override names `accum_fire_phase_census` explicitly because it
lives in `binary_id(wat)` and the `binary_id(wat::rete)` filter *"silently misses it"*. **Nine other
`#[test]`s in that same binary run the same (200,200) census primitive and are named nowhere.**

**This strike is a MEASUREMENT first.** Only names whose measurement earns the budget go into the
filter. If a test measures safe, it stays out and the number says so.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/strike-rete-cohort-budget/DESIGN.md` — the population, the
   hazard, and ⚠ **the counter-evidence**: two green floors this session, neither tripping any of
   the nine. Read that before you assume the budget is needed.
2. `.config/nextest.toml`, the rete-cohort block — the existing override, its three measured rows
   (`alone / under floor / contention`), the recorded **3.5x–4.4x** band, and the sentence
   *"A future test added here should be budgeted at alone × 4.4, not × 3.5."* **This is the format
   your rows must match.**
3. The same file's `retries = 0` note — what a red here would be mislabelled as, and why that
   matters more than the seconds.
4. `src/rete/kernel/tests/accum_cost.rs:169-247` (`accum_fire_phase_census`) — **the calibration
   point.** It runs a 4-point ladder `[(25,50),(50,100),(100,200),(200,200)]` *plus one extra*
   (200,200) call, and that shape costs 8.13s alone. Every one of the nine must be compared to
   *this* shape, not to each other.
5. The nine, listed in DESIGN.md with file and line.

## Method — the file's own, not a new one

- **Alone:** `cargo test --release <name> -- --exact --nocapture` per test, **nextest bypassed** so
  no deadline can truncate the number (the existing rows were taken exactly this way, and the file
  says so). Six samples, take the **minimum** — this repo's own rule is minimum-across-runs, and
  `[[six-samples-or-no-number]]`.
- **Loaded:** one `scripts/floor.sh`, then read each of the nine's duration out of
  `.floor/latest/clean.log`. That is one floor for all nine, not nine floors.
- **Contention:** loaded ÷ alone, per test.
- **Verdict per test:** `alone × 4.4` — the band's top, per the file's own instruction — against the
  **default 30s kill**. Over ⇒ it needs the budget. Under ⇒ it does not, and the row records that.

## Implementation sketch

Add only the earning names to the existing `filter = '…'` alternation — **the same rule, extended**,
not a new `[[profile.default.overrides]]` block. ⚠ The file warns three times that **ORDER IS
LOAD-BEARING** and first-match wins; a new block above `binary_id(wat::lint)` would strip budgets
from tests that already have them. Then mirror into `[profile.ci]` and `[profile.slow]`, which the
file does for every other cohort with the reason *"a deadline only the dev profile can meet is not
a deadline."*

Above the filter, add the measured rows in the existing table format:

```
#   test                          alone    under floor   contention   verdict
```

## STOP triggers

1. **If a test measures UNDER 30s at `alone × 4.4`** — it does not go in the filter. Record the
   number and move on. Adding it anyway would be padding the filter with unmeasured names, which is
   the defect this strike exists to close, committed by its own cure.
2. **If any of the nine measures ALONE over 30s** — STOP and report. That is a live floor hazard
   today, not a projection, and it outranks the config edit.
3. **If the nine are not in `binary_id(wat)`** — STOP. The whole premise is that they share
   `accum_fire_phase_census`'s binary and therefore its blind spot. Verify with
   `cargo nextest list` before measuring.
4. No change to any existing budget, to `RUNS`, or to any test body.

## Blast radius

`.config/nextest.toml` only. **No `.rs` file is touched by this strike.**
