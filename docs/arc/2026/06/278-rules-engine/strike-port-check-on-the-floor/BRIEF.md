# BRIEF — put the port check on the floor, and give its corpus the shape that beat it

Every axis `.wat` already computes both the native and the oracle answer in one process. Nothing has
ever compared them. Make that a floor gate, and add the axis shape whose absence would have let D7
through even if it had been running.

## Read in order

1. `wat-scripts/perf/grid/run-axis.sh:289-296` — the three pairings and what each diagnoses. Note
   `oracle vs native ⇒ a PORT bug`, and that it needs **no Clara**.
2. `wat-scripts/perf/grid/min-finding.wat:35-52` — an axis's stdin/stdout contract: stdin is an i64
   vector of sizes, stdout is one `#grid/Result` EDN line carrying `:derived` and `:oracle-derived`.
3. `wat-scripts/perf/grid/run-axis.sh:296-315` — how the runner extracts `:oracle-derived` and
   compares. **Reuse the extraction shape; do not invent a second one.**
4. `docs/arc/2026/06/278-rules-engine/strike-two-writers-one-alpha/SCORE.md` — D7's trigger, which is
   the shape your new axis must carry.
5. `tests/rete/probe_arc278_d7_parametric_erasure_differential.wat` — a working parametric-record
   world with mixed packability, already in tree. **Copy its shape for the new axis.**

## Driven by the orchestrator, at HEAD

All eleven axes at correctness sizes (`min-finding [100 3]`, `negation [50]`, `asym-join [100]`,
`strat-neg [3 50]`, `accum [10 20]`, `fanout [500]`, …): **11/11 `:derived` == `:oracle-derived`,
total 5 seconds, no JVM.**

**⭐ RE-DRIVEN 2026-09-03 at HEAD `daa92c3b0`** — the original drive was at `3144f9123` and **three
`src/` changes have landed since** (D10, D11, C19: `check.rs`, `freeze.rs`,
`validate/{mod,typing,error}.rs`). Still **11/11, 5.7s** — and this time with per-axis element counts,
so the green is not vacuous:

```
min-finding 49 · negation 25 · leading-exists 20 · neg-consumer 25 · asym-join 200 · strat-neg 75
user-reduce 5 · node-share 20 · accum 50 · deep-cascade 200 · fanout 400
```

So the gate is green the day it lands — its value is the next port bug, not this one.

## The two pieces

1. **The gate.** Every axis, correctness sizes, `:derived` vs `:oracle-derived`, failing with **both
   sets named** — a count is not enough, a port bug can be right-sized and wrong-valued (D7 was).
2. **The corpus hole.** Add a parametric axis: one class whose instances differ in packability. **0
   of 185 `defrecord` forms in the corpus are parametric today**, so the gate cannot currently
   express D7's defect. Verify by construction that reverting D7's cure REDs your new axis — if it
   does not, the axis does not carry the shape.

## Blast radius

`wat-scripts/perf/grid/` (a new axis + its `gen-` twin if the ladder discovery requires one) and one
gate under `tests/`. **No `src/` change.** Note two gates read every `.wat` under `wat-scripts/`, so a
new axis must load and resolve.

## STOP triggers

1. **If any axis's port check is RED at HEAD**, stop and report it as a finding — that is a live port
   bug and it outranks this strike.
2. **If any axis's `:derived` set is EMPTY, stop.** An empty set compares equal to an empty set and
   prints `match`, proving nothing. Driven: `fanout` at the wrong size arity yields `[] == []` on
   three different sizes. Non-vacuity is a gate requirement, not a nicety. (And the old wording of
   this STOP was wrong: `fanout` emits a `#fan/QuerySplit` line **and then** a `#grid/Result` — read
   the second line; there is no record-type problem. It takes a **single-number** size.)
3. **⛔ THIS STOP'S PREMISE WAS BACKWARDS AND IS CORRECTED — do not stop on it.** It read
   *"`run-all.sh` discovers axes by that pairing and errors on a `.wat` without one"*. **The opposite
   is true** (`run-all.sh:85`): a `.wat` with **no** `gen-` twin is `continue`d — *"not a perf axis
   (where-* has no gen-, by design)"*. What exits 2 (`:88-99`) is a `.wat` **WITH** a `gen-` twin and
   **no LADDER rung**. So: give the parametric axis **no `gen-` twin**, and do not name it `where-*`
   (`check-where-shapes.sh:140` globs `where-*.wat` and hard-fails a missing `.clj` twin). It then
   lands clean — run-all skips it, check-where-shapes never sees it. The two `wat-scripts/` lint
   gates DO read it: it must load and every `:wat::rete::` name must resolve.
4. **If your gate's runtime exceeds ~60s**, stop and report the sizes; correctness sizes are not the
   perf ladder and must not drift toward it.

## Mutation proofs — run both, report both

1. **Revert D7's cure** (`git checkout 523152b31 -- src/rete/kernel/fire/pass/alpha.rs`) → **your new
   parametric axis must go RED**, naming both sets. This is the row that proves the corpus hole is
   closed; the other ten axes will stay green, which is the point.
2. **Corrupt one existing axis's oracle answer** (perturb its `oracle-derived` computation) → that
   axis REDs. Proves the gate reads the oracle column and is not merely re-comparing native to
   itself — the C16 failure, one directory over.

Restore after each.

## What to report

- The gate's output across all axes, and its runtime.
- The new axis, and the proof it carries D7's shape (mutation 1).
- Both mutation results.
- Whether `fanout` is covered or excluded, and why.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong.** Ten riders have run on this arc; every one found a real
  defect in the brief, twice an instrument I named that was structurally blind. Be blunt.

Do not commit.
