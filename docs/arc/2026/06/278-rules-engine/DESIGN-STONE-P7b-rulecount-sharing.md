# DESIGN — Stone ② / P7b: the rule-count × shared-prefix matrix cell (the last unmeasured Clara regime)

## Status / the finding that reshaped this
The structural question — *do we share beta/join-prefixes across rules?* — is **answered YES by code
inspection**: the hash-join dedup key is `"hashjoin:<parent-id>:<cond-text>"` (`rete.wat:481,489`), so two
rules with the same parent + same (textual) condition reuse the same node (the `C1⋈C2` work is done once,
feeds both suffixes). NEXT-ANGLES called it "unconfirmed"; the code is definite. So this cell is no longer
"discover/build sharing" — it is **measure that the sharing holds vs Clara at high rule count**, and surface
the one real nuance below.

## The thesis (P7 philosophy: find or rule-out a Clara edge)
At high rule count with shared LHS prefixes, Clara's mature beta-network sharing could hide an edge. We share
too — so the bench's job is to **confirm we hold (or find the cell where we don't)**, and to expose the
**syntactic-vs-semantic** sharing question:

- **Our sharing is SYNTACTIC** — `cond-text = write-forms cond`, so a prefix is shared only if it is
  *textually identical* (same var names, same field order). Rules with the same prefix but **renamed vars**
  (`?loc` vs `?x`) do **not** share.
- Clara may **canonicalize** var names and share *semantically*. If so, the renamed-var variant is the cell
  where Clara edges us — and it points at a concrete follow-on (canonicalize the prefix before the dedup key).

## The two variants (one shape-spec → both emits, per P7)
A generator parameterized by `N` (rule count) and `K` (shared-prefix depth):
- **V1 — identical-prefix:** N rules share a K-condition prefix written *textually identically*; each rule
  adds one distinct suffix condition → derives a distinct result fact. **We share** (expect: sublinear network
  growth in N for the prefix; we hold/win on fire time).
- **V2 — renamed-prefix:** same N rules, same semantic prefix, but each rule **renames the prefix vars**. **We
  do NOT share** (every rule mints its own prefix nodes). This is the probe for the syntactic-vs-semantic gap
  vs Clara.

Sweep `N ∈ {10, 50, 100, 500}` (K=2 to start). Feed a fact set that exercises the shared prefix (so the
prefix join actually does work that sharing would amortize).

## Metrics
- **Network size** (node count after compile) for V1 vs V2 — the *direct* proof of sharing (V1 sublinear in N
  on the prefix; V2 linear). This needs no Clara — it's our own structural confirmation, and it doubles as a
  **regression guard** for the sharing optimization.
- **Fire time** (native `fire-rules'`) V1 vs V2 vs **Clara** on the identical workload — the perf head-to-head.
- Per cell: our derived-count == Clara's (same workload, else the generator is wrong, not the engine).

## Build (mirror the existing harness)
- `wat-scripts/perf/matrix/rulecount-sharing.wat` — quasiquote codegen of N shared-prefix rules (mirror
  `deep-cascade.wat`'s `build-rule` + `fanout-join.wat`'s shape); reads `[N K variant]` from stdin; emits a
  `:perf::Result` record (println → EDN). NATIVE-only timing (the wat-spec re-run is O(closure²) at scale —
  never bench the spec).
- `wat-scripts/perf/matrix/rulecount-clara.clj` — the matching Clara program from the SAME shape.
- Driver: orchestrator runs the grid (binary + clojure are ORCHESTRATOR-ONLY), tabulates `{N, variant →
  nodes, native-ns, clara-ns, derived, verdict}`.

## Fairness invariants (carry from P5b)
Both compute the full closure (assert derived counts match); fire-only timing; Clara JIT-warmed; ours AOT; one
shape-spec proves same-workload by construction.

## Out of scope (exigere — log, don't silently cover)
- Building semantic prefix-canonicalization (V2's potential fix) — this cell MEASURES the gap; if V2 shows
  Clara winning, the canonicalization is a named follow-on perf stone, not this cell.
- `K` beyond 2 and `N` beyond 500 — log the swept range; extend only if a crossover appears at the edge.

## Done
The crossover map (V1 + V2 × N, us vs Clara) + a verdict: **we hold / we win / or the V2 cell where Clara's
canonicalization edges us (→ named follow-on)**. Plus the network-size confirmation (V1 sublinear) as the
regression guard. Honest range-not-swept logged.
