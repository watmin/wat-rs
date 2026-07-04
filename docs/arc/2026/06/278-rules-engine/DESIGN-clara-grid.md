# DESIGN — the complex Clara grid: prove `:wat::rete` meets-or-exceeds Clara (perfect accuracy + faster)

> **The close condition for rete, made measurable.** The builder: *"we need to know we meet or exceed their
> tooling — perfect accuracy and faster results."* Every rete capability is built and green **in-house**
> (native == wat-oracle: 68/68 + 26/26). This grid proves the missing half: **native == Clara (accuracy)** AND
> **native < Clara (speed)**, across every axis, at scale, as a **permanent re-runnable ward** that turns
> `278 R18 RENASCOR NON RETRACTO` PROBATVM.

## The one contract — a per-axis strike is THREE artifacts + a verdict

For each axis, mirroring the proven `deep-cascade` pair (`wat-scripts/perf/{deep-cascade.wat, clara/gen-bench.sh}`):

1. **`<axis>.wat`** — a scale-parameterized wat workload: reads size params on stdin (`echo '[…]' | cargo wat`),
   builds the rules+facts, fires the **native** `:wat::rete::fire-rules` (the user-facing fast path), and emits ONE
   EDN line: `#grid/Result {:axis "<axis>" :size […] :derived <SORTED-DERIVED-FACT-VECTOR> :native-ns <i64>}`.
   The `:derived` field is the **full sorted derived-fact set** (the accuracy witness), not just a count.
2. **`<axis>-clara.clj`** (or a `gen-<axis>.sh` emitter) — the SAME workload in Clara (`clara-rules 0.24.0`):
   same rules, same seed facts, fire, and emit `#grid/Result {:axis … :size … :derived <SORTED …> :clara-ns <i64>}`
   with `:derived` in the **same canonical shape** so the two sets compare directly.
3. **the assertion** (in the runner): `native.:derived == clara.:derived` (ACCURACY — perfect or it's a bug) AND
   `native.:native-ns < clara.:clara-ns` (SPEED — or a recorded ratio if behind, honestly). Clara timing warms the
   JVM/JIT first (3 dry fires), times fire-only; native is AOT.

## The runner — automate the compare into a verdict (the permanent ward)

`wat-scripts/perf/grid/run-axis.sh <axis> <size…>`:
- run the wat side (`cargo wat <axis>.wat`), capture its `#grid/Result`;
- run the Clara side (`clojure -Sdeps … -M -m <axis>`), capture its `#grid/Result`;
- compare `:derived` (accuracy: PASS/FAIL) + `:native-ns` vs `:clara-ns` (speed: ratio);
- emit a verdict row: `#grid/Verdict {:axis … :size … :accuracy :match|:MISMATCH :ratio <f64> :winner :us|:clara|:tie}`.
`wat-scripts/perf/grid/run-all.sh` sweeps every axis × a size ladder → the verdict grid. Re-runnable = the ward.

## The axes (the grid rows) — all built + native==oracle; this proves them vs Clara

| # | axis | Clara translation | note |
|---|---|---|---|
| A0 | **deep-cascade joins** | exists (`gen-bench.sh`) | ✅ speed done (R4); ADD the `:derived`-set accuracy differential |
| A1 | **fanout / low-selectivity** | exists (`fanout-clara.clj`) | ✅ speed done; ADD accuracy differential |
| A2 | **asymmetric-arrival joins** | Clara `defrule` join | the axis that HID R18; derived⋈input, right-before-left, at scale |
| A3 | **negation** (`:not`) | Clara `:not` accumulator-free | @ scale + Clara speed |
| A4 | **stratified negation** | Clara `:not` over derived (N strata × M rules) | **FOUNDATION axis** — the R18 capability; hardest Clara translation |
| A5 | **accumulate / exists** | Clara `acc/` + `:exists` | built-in folds (count/sum/min/max/distinct/group-by) + exists |
| A6 | **user reducers (custom accum)** | Clara `accum`/IAccumulator (custom fold) | the 118 interlock — percentiles/stddev/top-k; `acc/accumulator` vs Clara custom |
| A7 | **minimum-finding-set** (DDoS) | Clara acc + `:test` threshold | the DDoS primitive — "≥N findings to activate" |
| A8 | **node-sharing / rule-count** | high rule count, shared join-prefix | "the last unmeasured Clara cell" (NEXT-ANGLES ②) — beta/join-prefix sharing |
| — | *salience · `insert!`-family · arbitrary fact-types* | — | **DELIBERATELY CUT** rows: on the grid as "we don't, by design" (honest scope) |

## Decomposition — foundation-first, then parallel fan-out

- **STRIKE F (foundation, SOLO, weighed by orchestrator):** build the **runner** (`grid/run-axis.sh` +
  `#grid/Result`/`#grid/Verdict` shapes) AND the **A4 stratified-negation** axis end to end (wat workload +
  Clara `.clj` + accuracy differential + speed). Proves the whole pattern on the hardest translation. Nothing
  fans out until F is green + orchestrator-weighed.
- **STRIKES A0–A8 (fan-out, PARALLEL, one shadowdancer each):** each mirrors F's proven pattern for its axis.
  `secare`-clean — disjoint files under `wat-scripts/perf/grid/<axis>.{wat,clj}`, one shared runner (read-only).
- **SYNTHESIS (orchestrator):** run `run-all.sh`, weigh the verdict grid myself (re-run, never trust the report),
  produce the meet-or-exceed verdict. Turns R18 PROBATVM.

## STOP triggers (reject, surface — do not fake a comparison)
1. If an axis's Clara translation isn't faithful (different rule semantics → the derived sets can't be compared
   apples-to-apples) — STOP, surface the semantic gap; a dishonest differential is worse than none.
2. If `:derived` can't be canonicalized to the same shape both sides (fact ordering, tag form) — STOP; the
   accuracy differential is the point.
3. If a "cut" capability (salience/insert!/fact-types) is needed to express an axis in Clara — STOP; it means the
   axis isn't in our scope, record it as a cut row.

## Gate
`wat-scripts/perf/grid/run-all.sh` → every axis a verdict row; accuracy `:match` on all non-cut axes (a
`:MISMATCH` is a rete bug to fix, not to hide); speed ratios recorded honestly (win/tie/behind + the named cause).
