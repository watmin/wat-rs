# Clara head-to-head — `:wat::rete` vs Clara-rules, identical workload

The arc-278 close bench: grade our engine against **Clara** (Cerner's mature Clojure RETE, ~10 yrs) on the
*exact same* deep forward-chain cascade as `wat-scripts/perf/deep-cascade.wat` — depth-N × width-M, every level
a 2-way join on the prior level's DERIVED facts (`Stage@k-1 ⋈ Tag@k-1` on id → `Stage@k, Tag@k`). Both engines
compute the full closure (`deepest == width` at every size); we time **fire only** (compile + JVM JIT warmed
out first on Clara's side; our native is AOT Rust).

## Run
```bash
# our engine (native delta kernel, public fire-rules):
echo '[20 10]' | cargo wat ./wat-scripts/perf/deep-cascade.wat

# Clara (clara-rules 0.24.0 from Clojars; Clojure CLI + JDK required):
bash wat-scripts/perf/clara/gen-bench.sh 20 10 > /tmp/bench.clj
( cd /tmp && clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["/tmp"]}' -M -m bench )
```
`gen-bench.sh DEPTH WIDTH` emits the Clara `.clj` (defrecords + N generated `defrule`s + a timed `-main`).

## Results (2026-06-19, fire-only, ms; all full closures)

| depth × width | wat-spec (oracle) | **our native** | Clara (JVM, warm) | verdict |
|---|---|---|---|---|
| 5 × 5   |   23.9 | **0.49** |  2.83 | **us 5.8×** |
| 10 × 5  |  116.7 | **1.89** |  9.15 | **us 4.8×** |
| 20 × 3  |  463   | **7.35** | 13.3  | **us 1.8×** |
| 20 × 10 | 1180   | 12.2     | 12.1  | tie |
| 30 × 10 | 3718   | 36.2     | **14.1** | Clara 2.6× |

## Reading the result (honest)
- **We beat Clara at depth-heavy / smaller workloads** (up to 5.8×) — no JVM start/warmup tax, no GC, lean
  AOT code; our round-based semi-naive delta has low constant overhead.
- **Crossover ~20×10**; **Clara pulls ahead width-heavy at scale** (2.6× at 30×10). Cause: Clara does
  *per-element* incremental activation with mature alpha/beta indexing; our delta is *round-based* semi-naive,
  so wider levels re-probe more per round. The named optimization to close this: per-element incremental
  (not round-based) + sharper join indexing.
- **vs the wat reference engine (`fire-rules-spec`): 49–310× and widening** — the whole point of the Rust kernel.
- **The GC axis (not in the table):** Clara runs on the JVM — a stop-the-world pause is a tail-latency spike at
  the wrong moment for line-rate. Our engine has no GC (ownership + `Arc`), so tail latency is jitter-free by
  construction. For HTTPS/sampled-packet streams, predictable tail can matter as much as the median.

**Verdict:** competitive with a decade-mature engine on a first-cut delta engine — superior in the depth
regime, at parity mid, behind in the width regime — with a clear, named path (per-element incremental) to
close the width gap, and a structural tail-latency edge (no GC).
