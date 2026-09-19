# DESIGN — close ONE compound cell end-to-end, then judge the other four

**Status:** drawn 2026-09-09 from **`3P1`** (peragrare, 5×L1). ⛔ **Deliberately scoped to ONE of the
five cells.** The reason is in the pin.

## Why

`3P1`: *"THE CORPUS PROVES EVERY MECHANISM ALONE AND NO TWO TOGETHER."* All three defects that
birthed the ward — `userfn-head`, `retract-multiplicity`, `accum-over-derived` — have a fixture and a
mutation proof **as isolated axes**. **No fixture combines any two.** Five empty cells carry a live
compound hypothesis:

1. a user-fn head whose LHS accumulates over a type the same ruleset derives
2. a duplicate-retract feeding a **leading** accumulate
3. a leading accumulate whose `:from` is itself derived
4. a positive consumer downstream of a **leading** gate
5. a duplicate-retract of the accumulate's own source

**Each cure was proven only where the other pressure is absent.**

## Verified by driving the instrument, not reading the row

`bash peragrare-census.sh` → **108 cells, 9 visited, 99 empty, 16 members** — the row's figures hold
exactly. Axes are `(head-kind × retract × accum-from × accum-position × consumer)` =
`2 × 2 × 3 × 3 × 3 = 108`. ✓

## What a fixture costs — measured, because it decides the scope

- **`X.wat` + `X.clj`** in `wat-scripts/perf/grid/`.
- **Discovered by WALK**, not by name: `check-grid-three-way.sh:123` iterates `"$GRID_DIR"/*.wat`.
  So no runner registration is needed for the differential.
- **Census registration**: `EXPECTED_FIXTURES` and the coordinate table in `peragrare-census.sh`.
- ⚠ **`run-all.sh`'s `ORDER` is a DIFFERENT population** — 11 *dialed perf* axes, against the census's
  16 differential fixtures. A compound fixture belongs to the differential, **not** the perf grid;
  do not add it to `ORDER`.
- ⛔ **`tests/rete/wat_scripts_grid_axes_live.rs` is a LIVENESS gate**: every grid `.wat` must
  actually RUN and derive something non-empty. **An inert fixture reddens it** — which is the right
  outcome and is why this cannot be closed with a stub.

## The one contract decision, pinned

⛔ **CLOSE EXACTLY ONE CELL, COMPLETELY, AND STOP.** Not five.

The unknown is not *which* cells — the row names them — it is **what a compound fixture costs when it
must agree three ways** (wat native, wat oracle, Clara) *and* derive non-empty *and* not perturb the
census's own pins. **Nobody has built one.** Five fixtures drafted against an unmeasured cost is how
a strike becomes a swamp; one fixture driven end-to-end turns the other four into a known quantity.

**Take cell 3 — a leading accumulate whose `:from` is itself derived** — `(record, absent, derived,
leading, na)`. It is the tightest of the five: both mechanisms already exist as isolated axes
(`accum-over-derived` is `(…, derived, nonleading, …)`, `leading-exists` is `(…, none, leading, …)`),
so the fixture is their intersection and both parents can be read side by side.

**Then report the cost and stop.** I will judge the remaining four against what you learn.

## Out of scope = rejected

- **The other four cells** — see the pin. Not deferred; **cut from this strike pending its result.**
- **Adding anything to `run-all.sh`'s `ORDER`** — wrong population.
- **Widening the census's axes.** The grid is what it is; this closes a cell in it.
