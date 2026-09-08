# DESIGN — nine tests run the measured cost class with none of its budget

**Status:** drawn 2026-09-08 from vigilia row `4W1` (circumspicere, target 4, L1) — the finding that
closed the vigilia.

## Why

`.config/nextest.toml` measures `accum_fire_phase_census` at **8.13s alone / 35.39s under floor —
4.35x contention** and grants it `slow-timeout = 90s/180s, priority = 98`, by name, in the rete
cohort override. The file **records why that name is there**: the test lives in `binary_id(wat)`,
not `binary_id(wat::rete)`, so *"a `binary_id(wat::rete)` filter alone silently misses it."*

Someone found that gap, understood it, and closed it **for the one test in front of them.**

Deriving the population from the primitive rather than the name — `grep` for
`accum_phase_census(200,200)` / `accum_count_census(200,200)` / `(G, W)` over
`src/rete/kernel/tests/`, then walking back to each enclosing `fn` — gives **ten** call sites in
**nine** other `#[test]`s:

| test | file |
|---|---|
| `accum_matcher_op_census` | `accum_cost.rs:33` |
| `accum_leftover_split` | `accum_cost.rs:606` |
| `accum_seen_fire_context_split` | `accum_cost.rs:1659` |
| `accum_alpha_leftover_split` | `accum_alpha_cost.rs:75` |
| `accum_alpha_seed_after_fold_split` | `accum_alpha_cost.rs:383` |
| `cell_rank_after_fanout` | `rank_and_instrument.rs:1149` |
| `cell_rank_after_grid` | `rank_and_instrument.rs:1233` |
| `honest_cell_rank_after_arm` | `rank_and_instrument.rs:1319` |
| `gather_index_is_built_once_per_alpha_and_keyset` | `gather_probe_cost.rs:31` |

**`accum_fire_phase_census` appears 7 times in `.config/nextest.toml`. All nine appear 0 times.**
All nine are plain `#[test]`; `#[ignore]` across the four files is **0**. Four of them run **three**
(200,200) calls in a bare `RUNS = 3` loop with no cheaper ladder points — **a heavier shape than the
test that was measured and given headroom.**

## The hazard, in the file's own words

At the cohort's recorded contention band (**3.5x–4.4x**, which the file explicitly says applies to
*"a future test added here"*) these project to **30–53s against the default 15s warn / 30s kill.**

And `.config/nextest.toml`'s own struck `retries = 1` note says what that failure would look like:

> *"the second run passing DESTROYS the only evidence the first run produced, and the report says
> 'flaky' where the truth is 'failed, once, with an arm nobody kept.'"*

A sibling timing out presents as **a flaky rete test, in a repo whose `CLAUDE.md` says there is no
such thing.**

## ⚠ The counter-evidence, which must be carried

**Two full release floors this session finished green with only ONE slow line**, and it was
`deftest_wat_tests_rete_tms` — a test the cohort filter already covers, tripping its own 90s warn.
**None of the nine tripped a SLOW at 15s on either run.** Those floors totalled **452s**, roughly
double the ~220s the contention band was derived at, so the nine were not under the load the band
describes — but that is an argument, not a measurement, and it cuts against urgency.

**So this strike MEASURES before it edits.** The projection is not evidence.

## What this delivers

The nine either get the budget or are shown not to need it — **with a number either way**, in a file
whose whole discipline is *"THE NUMBER, DERIVED NOT PICKED."*

## The one contract decision, pinned

⛔ **NO NAME GOES INTO THE FILTER WITHOUT ITS OWN MEASUREMENT.** The existing entries each carry an
alone-time, a loaded-time and a contention factor. Adding nine bare names would be the exact thing
this repo's config forbids twice in its own margins — *"⛔ RAISING A NUMBER IS NOT A FIX"* — and
would make the cohort's derivation unfalsifiable for the next hand.

## Out of scope = rejected

- **Raising any existing budget.** Not this strike's numbers.
- **Splitting the cost tests.** The file names splitting as the durable fix for the *scripts gate*;
  it is a different strike and not this one.
- **Adding `#[ignore]` to anything.**
