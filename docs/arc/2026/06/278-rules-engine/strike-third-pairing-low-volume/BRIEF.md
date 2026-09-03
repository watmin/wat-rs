# BRIEF — the third pairing at low volume, and a Clara twin for the shape that beat us

Make `clara | oracle | native` a low-volume check across every perf axis, and give
`parametric-erasure` the Clara twin it lacks. The pairing costs 43 seconds, not four hours — the
hours were the oracle on the perf ladder, which is out of scope here and forbidden by the builder.

## Read in order

1. `wat-scripts/perf/grid/check-query-compat.sh:1-12` — **the model, and it already exists**:
   *"query mouth, three ways — Clara 0.24.0 | wat-oracle | wat-native"*, 24 rows, wired into CI
   today. Copy its shape; do not invent a fourth harness idiom.
2. `wat-scripts/perf/grid/check-where-shapes.sh:23,140` — the cost lesson: 38 families in **one
   JVM, 3.7 s Clara total**, against ~67 s for six of them through `run-axis.sh`. One JVM, all rows.
   Also `:140` is the `where-*.wat` glob that proves a non-`where-` name is never swept there.
3. `tests/rete/wat_scripts_grid_port_check.rs` — **today's gate and your size table.** Its axis list
   and correctness sizes are the population this strike must match. Its four guards (echoed size,
   non-vacuity, the pairing, oracle-side cardinality) are the shape to follow, including the
   ordering lesson recorded in its header.
4. `wat-scripts/perf/grid/gen-min-finding.sh:1-13` — a generator's contract: `gen-<axis>.sh ARGS`
   emits a Clara program on stdout whose `:derived` is canonically encoded to compare byte-for-byte
   with wat's.
5. `wat-scripts/perf/grid/parametric-erasure.wat` — the axis needing a twin. Its rules and its
   canonical encoding are what your `.clj` must reproduce.
6. `wat-scripts/perf/grid/run-all.sh:78-99` — why the twin is a **static `.clj`** and not a
   `gen-` script: a perf axis is `<axis>.wat` WITH `gen-<axis>.sh`, and one without a LADDER rung
   exits 2.

## Driven by the orchestrator, at HEAD `ed555d02e`

`probe-threeway.sh.txt` beside this brief, run before the brief was written: **11/11 axes agree on
all three pairings at the port gate's sizes, 43 s total.** Per-axis element counts are in the
DESIGN. So the gate is green the day it lands, except for the axis that has no twin yet — its
absence is the work.

Sizes (the port gate's own): `min-finding [100 3]` `negation [50]` `leading-exists [20]`
`neg-consumer [50]` `asym-join [100]` `strat-neg [3 50]` `user-reduce [5 20]` `node-share [10 20]`
`accum [10 20]` `deep-cascade [5 20]` `fanout [500]` `parametric-erasure [200]`.

## Two extraction facts that cost the orchestrator two wrong readings

- The EDN is `:derived #wat.core/PersistentVector [1 2 3]` — a **tagged literal** sits between the
  key and the bracket, and elements are **space**-separated. A regex for `:derived \[` finds
  nothing; a comma-counter reports 0 elements.
- Anchor on the **leading space**: a naive search for `:derived` will also match inside
  `:oracle-derived`. (⚠ Note the tree carries a doc-comment claiming `:oracle-derived` *contains*
  `:derived` — it does not, the colon is part of the needle. The hazard is latent, not live.)
- A Clara file for namespace `min-finding` must be named `min_finding.clj` — dashes become
  underscores or `clojure -M -m` cannot locate it.

## The two pieces

1. **The harness.** Every perf axis at correctness sizes; `clara`, `:derived` and `:oracle-derived`
   compared three ways; a divergence **names which pair** and prints both sets plus their symmetric
   difference. Non-vacuity is a hard guard — an empty set agrees with an empty set.
2. **`parametric-erasure.clj`, static.** One class whose instances differ in packability on the wat
   side; in Clojure they are ordinary heterogeneous records, which is the point — Clara referees the
   **derived set**, not wat's declaration. **Verify by construction that reverting D7's cure makes
   the three-way RED on this axis** (`git checkout 523152b31 -- src/rete/kernel/fire/pass/alpha.rs`),
   naming `oracle≠native`. If it does not, the twin does not carry the shape.

## Blast radius

`wat-scripts/perf/grid/` (one harness + one static `.clj`) and at most one gate under `tests/`.
**No `src/` change.** Note two gates read every `.wat` under `wat-scripts/`, and
`tests/lint/every_parity_script_is_invoked.rs` requires every `check-*.sh` in that directory to be
invoked by CI **or** by a Rust test — so a new script must be wired, or it is not a gate.

## STOP triggers

1. **If any axis's three-way is RED at HEAD**, stop and report it. `oracle≠clara` means the SPEC is
   wrong, and that outranks this strike entirely.
2. **If any set is empty**, stop. `[] == []` reports agreement and proves nothing.
3. **If the harness's runtime exceeds ~120 s**, stop and report the sizes. 43 s is the measured
   baseline with one JVM per axis; batching should lower it, never raise it.
4. **If writing `parametric-erasure.clj` requires a `gen-` script or a LADDER rung**, stop and
   report — that would drag a correctness shape into the perf artifact, which the DESIGN rejects.

## Mutation proofs — run all three, report all three

1. **Revert D7's cure** → `parametric-erasure` REDs, naming `oracle≠native`. Proves the new twin
   carries the shape.
2. **Corrupt one axis's Clara program** (perturb its derived encoding) → that axis REDs naming
   `oracle≠clara` **and** `native≠clara`, and NOT `oracle≠native`. Proves the harness reads Clara and
   attributes the pair correctly.
3. **Empty one axis's sets** → `VACUOUS`, not `match`.

Restore after each, and **verify the restore by hash** — `git checkout <sha> -- <path>` STAGES, so
`git diff --stat` shows nothing after a real mutation.

## What to report

- The harness output across all axes and its wall time.
- All three mutation results, each with the RED it produced.
- The new `.clj`, and the proof it carries D7's shape (mutation 1).
- How the script is invoked (CI, a Rust test, or both) — `every_parity_script_is_invoked` gates this.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Yesterday's rider found that the gate it was
  told to build already existed and compared `X == X`; the brief had not named the file. Assume
  something comparable is here and go looking for it.

Do not commit.
