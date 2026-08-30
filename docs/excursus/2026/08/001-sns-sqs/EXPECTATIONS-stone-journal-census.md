# EXPECTATIONS — excursus 001 stone JOURNAL-CENSUS

**Written BEFORE the strike, 2026-08-30.** Blast radius derived from the BRIEF's own section.

## ⚠ The outcome is UNKNOWN and a RED table is the success case

This is stone 2b's shape again: a measurement of code that has never run. **Two results are
both valid deliveries** — "all 15 agree" and "N of 15 lose data". The failure mode is neither.
It is a table produced by an instrument that was not actually swapping the backend, which is
what STOP-1's control exists to catch.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | ★ the control disagrees | `span_macros`: mem 3, sqlite 1. **If it agrees, the run is void** |
| 2 | all 15 attempted | each has a row, or a named structural reason it could not run (STOP-2) |
| 3 | the table is per-fixture | mem value, sqlite value, agree?, what differs |
| 4 | a verdict line | "N of 15 lose data on the real backend" |
| 5 | zero production files | `git status --porcelain` shows no `wat/`, no `src/`, no `crates/` |
| 6 | zero committed test changes | no fixture edited in place; measurement copies deleted |
| 7 | floor unchanged | 5119, ONE known failure — before and after |
| 8 | nothing fixed | `journal`, `metric->row`, `log->row` untouched (STOP-3) |

## Runtime prediction

**60–120 minutes.** Fifteen fixtures, each needing its backend swapped and both sides run. The
mechanical part is fast; the slow part is fixtures that resist the swap for a real reason,
which are the interesting rows.

## Trap-doors

1. **A fixture may pass on both backends and still be lying.** Agreement only means the two
   stores did the same thing on *that* sequence. `journal_backend_differential` agreed for
   months while `span_macros` was losing 2 of 3 metrics — because the differential never wrote
   two metrics at one nanosecond. **If a fixture agrees, note what it actually exercised**, not
   just that it agreed.
2. **Same-nanosecond emission is the trigger.** The collision needs two rows sharing
   `(namespace+kind, time-ns)`. A fixture that writes one metric, or writes them at distinct
   times, will agree and prove nothing about the bug. That is worth a column.
3. **`:index-names` must match what `journal`'s `:init` declares** (`by-uuid`), or sqlite will
   DDL an index the clear path never names. A mismatch would look like a data-loss finding but
   be an instrument bug — the same trap `EXPECTATIONS-stone-2b` named as trap-door 2.
4. **The `_on_process` fixtures fork.** `sqlite-store(":memory:")` in a forked child is its own
   database. If that changes what the fixture means, that is STOP-2, not a workaround.

## Not in this stone

- Any change to `journal`, `metric->row`, `log->row`, or the `SortKey`.
- Any fixture's assertion.
- A permanent both-backends fixture — worth proposing, belongs to the next stone.
