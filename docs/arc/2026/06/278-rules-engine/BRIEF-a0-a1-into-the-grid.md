# BRIEF — A0 (deep-cascade) and A1 (fanout) into the grid

## The work

The grid runs **seven axes, A2–A8**. A0 and A1 exist as scripts, were speed-measured in R4, and the
grid DESIGN says of both: *"✅ speed done; **ADD** the `:derived`-set accuracy differential."* Neither
got one, and neither is in `run-all.sh`. Bring them into the grid, contract-conformant, with the
accuracy differential.

**A1's top rung (40,000) is the only size in this project's history where Clara ever beat us**
(`REALIZATIONS.md:201` — `40k Clara 1.4×`). Measuring it again is the point of the stone.

This is a **measurement** stone: no `src/`, no `wat/`. Nothing in the engine moves.

## Read in order

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-a0-a1-into-the-grid.md`** — the three conformance
   gaps and the contract decision.

2. **`wat-scripts/perf/grid/run-axis.sh:1-30`** — the contract every axis must meet: stdin is an i64
   vector, stdout is one `#grid/Result {… :derived <SORTED> :native-ns N}`, and a sibling
   `gen-<axis>.sh` emits the Clara translation with the same `:derived`.

3. **`wat-scripts/perf/grid/accum.wat` + `gen-accum.sh`** — THE EXEMPLAR PAIR. Read how `:derived` is
   canonically encoded (`gen-accum.sh` header: `kind*1e15 + g*1e9 + val`, sorted ascending) so both
   sides emit a byte-identical vector. Your two axes each need their own injective encoding of the
   same kind.

4. **`wat-scripts/perf/deep-cascade.wat`** — A0's existing logic. Adapt it; do not rewrite it. Note it
   currently takes a `:perf::Params` record on stdin and emits `:derived` as a **count**.

5. **`wat-scripts/perf/matrix/fanout-join.wat` + `fanout-clara.clj`** — A1's existing pair. The `.clj`
   is static; you need a `gen-fanout.sh` that emits it parameterized by size.

6. **`wat-scripts/perf/grid/run-all.sh`** — the LADDER block. Two entries to add; the comment there
   explains why the ladder is the artifact.

## ★ THE ONE CONTRACT DECISION

**`:derived` must be the canonically-encoded SORTED FACT SET on both sides — or the axis does not land.**

A0 emits `:derived 120` today: a count. A count cannot detect a wrong answer with the right
cardinality, which is the exact class the accuracy differential exists to catch.

**A speed-only axis is worse than no axis**, because `run-all.sh` tallies it and the "N of N" line then
claims coverage the measurement does not have. If you cannot make the two sides agree byte-for-byte,
**STOP and report the gap** — do not land a speed number and call the axis covered.

## The ladder (add to `run-all.sh`)

```
[deep-cascade]="10 5|20 5|30 5"        # dial: depth (width fixed at 5)
[fanout]="10000|20000|40000"           # dial: items — 40000 is R4's Clara-win size, deliberately
```

## Blast radius

`wat-scripts/perf/grid/` only: two new axis `.wat`, two new `gen-<axis>.sh`, two ladder entries, and
`deep-cascade`/`fanout` added to `run-all.sh`'s `ORDER`. Leave the legacy
`perf/deep-cascade.wat` and `perf/matrix/*` in place — they are R4's record.

## STOP triggers (each is a rejection: ship nothing, report the gap)

1. **STOP-1** — if `:derived` cannot be made a byte-identical sorted vector on both sides, STOP. Do
   not fall back to a count, and do not land the axis speed-only.
2. **STOP-2** — if an axis comes back `:accuracy :MISMATCH`, **STOP and report it verbatim with both
   `:derived` sets.** That is a finding about the ENGINE, not a bug in your axis. Do not adjust the
   encoding until the sets agree — that would be encoding a real divergence away.
3. **STOP-3** — if A1 comes back `:winner :clara`, that is DATA, not a failure. Report it and stop. Do
   not optimize anything; whether the support-chain provenance still earns its cost is a separate
   ruling with its own design doc.
4. **STOP-4** — do not touch `src/` or `wat/`. If an axis seems to need an engine change, STOP.

## Definition of done

- `bash wat-scripts/perf/grid/run-axis.sh deep-cascade "10 5"` → one `#grid/Verdict`.
- `bash wat-scripts/perf/grid/run-axis.sh fanout "10000"` → one `#grid/Verdict`.
- `bash wat-scripts/perf/grid/run-all.sh` with no arguments runs **nine** axes.
- `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` — the new `.wat` files must
  load and type-check under the loader gate.
- `cargo nextest run --release` — the floor, unchanged.
- Report every `#grid/Verdict` you produced, verbatim, including any `:clara` or `:MISMATCH`.
- `git diff --stat`.

Leave the tree dirty and uncommitted. Do not commit, push, or stash.

## A prior result to copy for shape

`accum.wat` + `gen-accum.sh` are a complete, working, contract-conformant axis pair. You are producing
two more of the same thing from logic that already exists.
