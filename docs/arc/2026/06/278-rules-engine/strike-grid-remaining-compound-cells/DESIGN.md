# DESIGN — the remaining four compound cells, now that one is measured

**Status:** drawn 2026-09-09, following `strike-grid-first-compound-cell` (`22d96739b`). Closes the
rest of **`3P1`** (peragrare, 5×L1).

## What the first cell established

`(record, absent, derived, leading, na)` — `accum-lead-derived` — is closed. Census moved
**9→10 visited, 99→98 empty, 16→17 members**, total unchanged at 108. Floor 5490, no new `#[test]`.

**The three engines agreed byte-for-byte** — Clara 0.24.0, wat oracle and wat native all derived the
identical 10-element multiset at depth 9. **At that cell, the two cures compose.**

⭐ **And the registration surface is larger than I wrote.** My first design said *"discovered by
walk — no runner registration needed."* That is true of `check-grid-three-way.sh:123`'s glob and
**false** of two Rust gates that assert **exact set equality against hardcoded arrays**:

| site | what it needs |
|---|---|
| `wat-scripts/perf/grid/X.wat` + `X.clj` | the fixture and its Clara twin |
| `peragrare-census.sh` | `EXPECTED_FIXTURES` **and** the coordinate table |
| `tests/rete/wat_scripts_grid_axes_live.rs` | `SIZED_AXES` — **exact set equality** |
| `tests/rete/wat_scripts_grid_port_check.rs` | `CORRECTNESS_SIZES` — **exact set equality** |

**An unregistered fixture reddens the floor.** That is the correct behaviour and it is why this
cannot be done by dropping files in.

## The four remaining cells

Each is a **different axis pair** from the one measured (`accum-from × accum-position`), so **none is
derisked by the first result** — that is the executor's own judgement and I am taking it:

| # | hypothesis | axes crossed |
|---|---|---|
| 1 | a user-fn head whose LHS accumulates over a type the same ruleset derives | head-kind × accum-from |
| 2 | a duplicate-retract feeding a **leading** accumulate | retract × accum-position |
| 4 | a positive consumer downstream of a **leading** gate | consumer × accum-position |
| 5 | a duplicate-retract of the accumulate's own source | retract × accum-from |

Parents for each already exist as isolated axes: `userfn-head`, `retract-multiplicity`,
`leading-exists`, `neg-consumer`, `accum-over-derived`, `accum`.

## The one contract decision, pinned

⛔ **A DISAGREEMENT IS THE DELIVERABLE, NOT A DEFECT IN THE FIXTURE.** The hypothesis behind every
one of these cells is that **a cure proven under one pressure may not hold under two.** If any cell's
three engines disagree: **STOP on that cell, capture all three outputs verbatim, report, and keep
going with the others.** ⛔ **Do NOT tune a fixture until the engines agree** — a fixture adjusted
into agreement proves nothing and destroys the only evidence the cell exists to produce.

⭐ The first cell agreeing is a *result*, not a precedent. Four independent experiments remain.

## Out of scope = rejected

- **Widening the census's axes.** The grid is what it is.
- **`run-all.sh`'s `ORDER`** — the dialed perf population, not this one.
- **Chasing any disagreement to a root cause.** Capture it and report; the cure is its own strike.
