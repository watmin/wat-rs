# DESIGN — Stone P7: the Clara-comparison bench MATRIX (find or rule-out every superiority regime)

> **STATUS: QUEUED — build after P6 lands** (builder, 2026-06-19: *"how do we extend this stress test — how can
> we find other places (or prove they don't exist) where clara has superiority… yeah build the matrix harness
> after P6"*). The deep-cascade (P5b) is ONE point in the workload space; P7 makes the search systematic.

## The thesis
"We beat Clara" is unprovable from one workload. P7 turns the question into a **bounded grid search**: a single
**shape-spec** emits BOTH the wat program AND the Clara `.clj` (provably the same workload), sweeps each RETE
cost dimension, and charts the **crossover surface**. A dimension we win across its whole swept range is "ours
within [range]" (log the range NOT swept — exigere: no silent "we cover everything"). Each Clara-win cell is
categorized: **perf** (iterate, like P6), **capability** (build the stone), or **architectural** (a known named
boundary).

## The two axes (do NOT conflate)
**Axis 1 — capability (Clara wins by a feature we have not built).** Confirmed unbuilt (`wat/rete.wat:58`,
"Negation / Test / Accumulate / ExpressionJoin nodes arrive at stones 6–8"): `:not`, accumulators
(count/sum/min/max/avg/distinct/grouping + custom `accum` with `retract-fn`), expression-joins (`where`/Test),
`:exists`. These are NOT benched (we'd error) — they're **feature-deferred (stones 6–8)**, not "slower." P7
does not chase these; it records them as the capability gap and points at the stones.

**Axis 2 — perf regime (same workload, who's faster).** The matrix dimensions:

| dim | param | covered by deep-cascade? | why Clara might win |
|---|---|---|---|
| depth | chain length | ✅ (we win) | — |
| width | independent chains | ✅ (P6) | per-element indexing |
| **join arity** | conds/rule (2/5/8) | ❌ | deep beta network; join ordering |
| **selectivity / fan-out** | 1:1 unique-key vs M:N shared key | ❌ | **token explosion — THE classic RETE stress; hit FIRST** |
| **alpha selectivity** | match-ratio (most facts filtered) | ❌ | alpha throughput |
| **rule count + sharing** | 100s of overlapping LHS | ❌ | Clara's mature node-sharing |
| **raw fact scale** | millions, trivial rules | ❌ | pure throughput |
| **retraction churn** | insert/retract cycles | ❌ | **architectural: our replay rebuilds per fire; Clara incremental → Clara territory until the persistent/streaming engine (P4c-deferred)** |
| **streaming** | many small fires vs one | ❌ | cross-fire persistence (deferred) |

## The build (one shape-spec → two engines)
A generator parameterized by a shape map:
`{:depth :width :conds :fanout :alpha-match-ratio :rule-count :fact-scale :churn}`.
- **wat side:** generalize `wat-scripts/perf/deep-cascade.wat`'s `build-rule`/`seed` (quasiquote codegen) to honor
  the shape; emit a `:perf::Result` record (println → EDN), already the pattern.
- **Clara side:** generalize `wat-scripts/perf/clara/gen-bench.sh` to emit the matching `.clj` from the SAME shape.
- **driver:** an orchestrator script (`wat-scripts/perf/matrix/run.sh`) that sweeps the grid, runs both engines
  per cell, and tabulates `{shape → wat-ns, native-ns, clara-ns, deepest, verdict}` as EDN rows → a crossover map.
- **fairness invariants (carry from P5b):** both compute the full closure (assert `deepest`/count match); fire-only
  timing; Clara JIT-warmed; ours AOT. The generator must prove same-workload by construction (one spec, both emits).

## Verify / output
- Per cell: our `deepest`/derived-count == Clara's (correctness, same workload) — else the cell is invalid, fix
  the generator (not the engine).
- The crossover surface (which dims/ranges are ours, which are Clara's), each Clara-win cell tagged
  perf/capability/architectural. The honest close artifact: the map + the categorization.

## Out of scope
- Building negation/accumulators (stones 6–8) — P7 only MEASURES the gap, names the stones.
- The persistent/streaming engine (for churn) — named boundary, separate arc.
- Picking up new perf fights — P7 FINDS them; each found perf-cell is its own iterate-stone (P6-shaped).
