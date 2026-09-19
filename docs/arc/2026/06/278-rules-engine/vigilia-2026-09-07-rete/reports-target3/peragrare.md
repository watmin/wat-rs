## PERAGRARE — Cast Report (TARGET 3 — the grid, and the corpus that birthed this ward)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> Census committed at `wat-scripts/perf/grid/peragrare-census.sh`, beside the corpus it counts.

**CORPUS:** 54 `.wat` under `wat-scripts/perf/grid/` = **16 members** (the instrument's own `DISCOVERED` glob, `*.wat` minus `where-*`) | 38 never offered | **0 dropped by the instrument**
   membership: reaches the three-way comparison iff `check-grid-three-way.sh`'s discovery loop picks it up. The 38 `where-*.wat` are an **honest exclusion** — the script's own `case "$stem" in where-*) continue ;; esac` — handed instead to two *named* siblings (`check-where-shapes.sh`, `check-query-compat.sh`) asking different questions, not silently dropped.

**AXES:** 5, each derived from what could flip the oracle/native/Clara set comparison — three are this exact corpus's own founding-defect mechanisms, two more are open seams the corpus's own header comments name:

```
H  head-kind             (2: record, userfn)            <- stratum-by-produced-type vs by-fn-name
R  retract-duplicate     (2: absent, present)           <- remove-one vs remove-all
A  accumulate-source     (3: none, base, derived)       <- AccumulateNode :from a type the same
                                                           ruleset derives
L  leading-nonmonotonic  (3: none, leading, nonleading) <- parentless :exists/accumulate
                                                           re-emitting per fixpoint round
N  negation-consumed     (3: na, not-consumed, consumed)<- a POSITIVE rule consuming a
                                                           negation-gated fact (task #94)
not modelled: join-shape (asym-join/fanout/node-share width & asymmetry), cascade-depth
  (deep-cascade), record-genericity (parametric-erasure) — cut for size/time this session,
  not because undrivable.
```

**⛔ Named limitation (coordinate assignment):** H/R/A/L/N are **hand-derived and cited** (per-fixture citations in the census header and table comments), not a generic parser's output. A line-window text classifier was built and tested this session; it worked for R and A but produced false positives/negatives on H and L against this corpus's *second* LHS/RHS authoring idiom (named let-bound variables referenced by bare symbol, vs inline quoted forms) — a `:rhs` line's own bare variable-name argument pulled in an unrelated neighbouring `defn` call as a false "userfn" head. Rather than ship a classifier known to misclassify, the moving pin instead exercises the one part of the census that **is** mechanical end-to-end — the cross-product **aggregation** — and every coordinate is cross-checked at run time (`--verify`) against comment-stripped presence/absence facts that a future corpus edit would break loudly rather than silently.

**FINDINGS** (5 unvisited, L1 — each names a compound of two independently-proven-in-isolation mechanisms that no fixture stages together):

| cell (H,R,A,L,N) | verdict | defect that would hide there |
|---|---|---|
| (userfn,absent,derived,\*,na) | unvisited L1 | A user-fn `:then` head whose LHS accumulates over a type the same ruleset derives. `userfn-head.wat` proves stratum-by-produced-type only with `A=none`; `accum-over-derived.wat` proves supersession of stale accumulate tallies only with `H=record`. Never combined — a stratifier that gets *either* right in isolation could still mis-rank when a user-fn's output IS the accumulate's re-derived source. |
| (record,present,\*,leading,\*) | unvisited L1 | Retracting one of several duplicate facts that also feeds a **leading** (parentless) accumulate/exists — the exact cumulative-beta memory the 2026-08-24 defect was about. `retract-multiplicity.wat` proves remove-one only against a plain two-condition join (`L=none`). A remove-one that mis-decrements a leading accumulate's round-scoped state is invisible here. |
| (record,absent,derived,leading,na) | unvisited L1 | A **leading** accumulate whose `:from` source is itself derived — `accum-over-derived.wat` is nonleading; `accum-lead-rule-cascade.wat`'s leading accumulate sources a base type. Compounds supersession correctness with round-scoping correctness; each cure was proven only where the other pressure is absent. |
| (\*,\*,\*,leading,consumed) | unvisited L1 | A positive consumer downstream of a **leading** negation/exists gate. `neg-consumer.wat` and `userfn-head.wat` both prove positive-consumption stratum propagation, but both gates are **nonleading** `:not`. If a leading gate's per-round re-emission also multiplied a *positive consumer's* activations, nothing here would see it. |
| (record,present,base\|derived,\*,na) | unvisited L1 | Retracting one of several duplicate facts that is itself the accumulate's source. `retract-multiplicity.wat` has `A=none`; every `A≠none` fixture has `R=absent`. If a built-in `count`/`sum` maintains incremental per-token state rather than a full batch re-fold, remove-one discipline is unverified against an `AccumulateNode`'s internal state. |

**Read, not hollow (9 of 9 visited cells):** every populated cell's fixture demonstrably reads the axis/axes that make it distinct — e.g. `userfn-head.wat`'s witness carries both Rate and Out *"so a witness carrying Out alone cannot tell 'Out dropped' from 'nothing derived'"* (reads H **and** N); `accum-lead-rule-cascade.wat`'s witness asserts count-constancy specifically to catch a round-count leak (reads L). **No hollow cell found.**

**Secondary note (not a peragrare finding — an instrument-robustness aside):** the "BOTH is a hard failure" guard (`gen-<axis>.sh` + `<axis>.clj` coexisting) is real code (verified) and currently untriggered (verified: all 16 members are exactly-one-of), but no Rust test drives `check-grid-three-way.sh` at all — the guard has never been proven under mutation.

```
CELLS IN GRID:   108 = read 9 | hollow 0 | empty 99 | exempt 0
FINDINGS:          5 = unvisited (L1) 5 + hollow (L2) 0 + silently-dropped (L1) 0
                        + instrument-inert 0 + census-void 0
EMPTY, NOT FILED: 94
HYPOTHESES:        0 filed (no new .wat fixture was built to check one — forbidden by this cast's rules)
COST: ~2h. Reading the instrument and all 16 fixtures in full; building, debugging and finally
      scoping down a mechanical classifier (three iterations, two real bugs found — an awk `\b`
      portability failure and a `set -e`/`pipefail` interaction that silently truncated output —
      both fixed and left in the committed self-checks); re-deriving the stated coverage numbers.
VERDICT: does not span. All three founding defects (userfn-head, retract-multiplicity,
      accum-over-derived) are now closed as isolated axes, but the corpus stages each single
      mechanism alone — no fixture combines any two of {user-fn head, duplicate-retract, derived
      accumulate source, leading non-monotonic, positive-consumer-of-a-gate}.
```
