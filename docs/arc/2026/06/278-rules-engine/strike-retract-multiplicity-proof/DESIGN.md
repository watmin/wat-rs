# DESIGN — prove the retract violation against Clara, on the grid, as recorded data

> Drawn 2026-09-06 at HEAD `32deef4d6`. Builder's ruling: **Clara is the reference for correctness;
> the wat oracle must be in Clara parity; wat-native must adhere to the oracle's public behaviour
> and differ only in performance.** Sequence: **prove the violation, then drive the oracle, then
> native.**

## What is already established

Driven this session, both sides:

```
Clara 0.24.0 : 2x insert F + 1x G, fire -> 2 rows ; retract F ONCE -> 1 row ; again -> 0
wat          : facts_after_two_identical_inserts=3 ; facts_after_one_retract=1
```

`retract` (`wat/rete/oracle/insert.wat:100`) drops **every** equal fact; Clara drops one.

## ★ retract has NO native implementation — B and C collapse into one

Verified: **no `RETE_OPS` row, no `fn retract` anywhere in `src/rete/`, one definition** at
`insert.wat:100`. It is a shared wat verb that rebuilds `Session.facts`, and both `fire-rules` and
`fire-rules$oracle` consume that vector.

So the three-way signature this strike must produce is:

| pairing | expected | meaning (from `check-grid-three-way.sh`'s own table) |
|---|---|---|
| `native != clara` | **`:accuracy :MISMATCH`** | the fast path disagrees with the reference |
| `oracle != clara` | **`:oracle-accuracy :MISMATCH`** | **the SPEC is wrong** — *"the pairing NOTHING has ever run"* |
| `oracle vs native` | **`:port-accuracy :match`** | not a port bug; one verb serves both |

That signature is the proof, and it is precisely the diagnosis the script was built to render.

## Why no grid axis has caught this

Three axes call `retract`, and **not one stages a duplicate**:

| axis / row | staged | why blind |
|---|---|---|
| `where-not-fact` row 4 `retract-cold` | one `Temp 10`, retract it | no duplicate |
| `where-not-fact` row 5 `partial-retract` | `Temp 10` + `Temp 15` — **distinct** | "remove all equal" removes exactly one |
| `where-exists` `n-at-retract` | `Wind 50` + `Wind 60` — **distinct** | same |
| `where-not-or` `n-hit-retract` | one `Wind 40`, retract it | no duplicate |

Row 5 is *named* `partial-retract` — the row closest to the question — and uses two different values.

**The instrument is NOT the blind part.** `run-axis.sh:283` is a literal string compare of the
`:derived` vector after stripping wat's `#wat.core/PersistentVector` tag, and its header says it
*"fails loudly on any real difference … (missing/extra/**reordered**)"*. Multiplicity and order both
survive into the verdict. **The fixtures are the gap, not the comparison.**

⚠ The Clara sides of those axes DO collapse — `(count (set (map :?k …)))`. The new axis must not.

## The one contract decision, pinned

**The proof is a RECORDED GRID VERDICT, not a failing test.** The grid reports `:MISMATCH` as data
and gates nothing; the floor stays green. That is what lets a proof-of-violation land without a red
floor — the thing that went wrong earlier today on `--check`.

## Scope

**IN:** one new correctness axis — `retract-multiplicity.wat` (emitting `:derived`, `:native-ns`,
`:oracle-derived`, `:oracle-ns`), `gen-retract-multiplicity.sh` (the Clara translation), the
`SIZES` row, the `CORRECTNESS_SIZES` row — plus the recorded three-way verdict. **Floor GREEN.**

**OUT, affirmatively cut:** the cure (that is the next strike, and this verdict is its acceptance
test); the perf ladder in `run-all.sh` — this is a CORRECTNESS axis and the header forbids the two
size tables drifting toward each other; the derived-multiplicity divergence, closed as justified.

## ⛔ This lands in a gated tree

A `.wat` under `wat-scripts/` is walked by `every_wat_scripts_file_loads` and
`every_rete_name_in_wat_scripts_code_resolves`; a sized grid axis with no `SIZES` row makes
`check-grid-three-way.sh` error, and `tests/rete/wat_scripts_grid_port_check.rs` mirrors that table
**on the floor**. Add both rows, and run those gates.
