# EXPECTATIONS — compile the `where` predicate

Written **before** the strike, so the result cannot move the goalposts. Every row is scored against
the orchestrator's own re-run, never the rider's report.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | **The differential holds on the VERDICT** | the new corpus differential test | every (predicate, bindings) pair: `compiled == eval_test_core`, `Ok(bool)` identical |
| 2 | **The differential holds on the ERROR** | same test, the error arm | a non-bool predicate raises the SAME located `TypeMismatch`; an arithmetic raise propagates identically |
| 3 | **The fallback is COUNTED, not hidden** | `node_share_filter_eval_census` | `filter:test-env-builds == filter:test-interp-fallback`, both printed; `filter:test-evals > 0` |
| 4 | **The timing row** | `node_share_where_cost_decomposition` | arm B falls from ~540 ns/eval toward arm E's ~21; **B must at least halve** |
| 5 | **The phase moves** | `node_share_fire_phase_census` `[50 200]` | `filter` falls materially from 6.83 ms |
| 6 | **Setup stays bounded** | same census | `SETUP: indexes` does not exceed the alpha-tree stone's budget |
| 7 | **Accuracy unmoved** | the nine grid axes | `:accuracy :match` on every one; `:derived` byte-identical |
| 8 | **The floor** | `cargo nextest run --release` | 4253/4253 (4251 + the two probe tests already landed + the new differential), 0 failed — read the **Summary line** |
| 9 | **Clippy** | `cargo clippy --release --all-targets` | 0 |
| 10 | **`eval_test_core` still has a caller** | `grep` | it is `Op::Interp`'s body; if it has zero callers the fallback was not built |
| 11 | **Nothing under `wat/` moved** | `git diff --stat` | zero `.wat` files touched |

## Independent prediction

**Runtime: 45–75 min** for a sonnet rider. The exemplar is 445 lines and directly copyable; the op
set is ~8 variants; the differential harness is the largest single piece. Wakeup scheduled at **2×
the upper bound = 150 min**.

**Row 4 is the one I am least sure of, and it is stated as a band on purpose.** Step 0 measured the
floor at 21 ns/eval with a hand-written closure that reads one binding and does four arithmetic ops.
The compiled executor carries a `WOp` tree walk, a `Value` clone per slot load, and an enum dispatch
per node — so it will land **above** the floor, and I will not pretend to know where. "At least
halve" is the falsifiable claim; anything better is upside.

## Trap doors — named in advance

- **The accessor's runtime class lookup could eat the win.** `Field` does a `sym.types()` get plus a
  field-name scan per call. On the arena's predicates that is most of the work. If arm B barely
  moves on accessor-heavy predicates while moving hugely on node-share's arithmetic one, that is the
  cause, and the disposition is the inline cache (out of scope here, its own stone) — **not** a
  quiet widening of this strike.
- **`Op::Interp`'s env build could dominate on the fix/clojure rulesets.** 24 of 55 corpus
  predicates have no key expression and several are user-fn calls that must fall back. Row 3 is what
  makes this visible instead of letting row 4's node-share number speak for the whole corpus.
- **The differential could be vacuous.** If `corpus_pairs` is empty or tiny the test passes proving
  nothing. It must assert a floor on the pair count, and the count must be printed.
- **Row 5 could read inside the noise.** `filter` at 6.83 ms is a single-run figure. Interleave, gate
  on load (`until awk '{exit !($1 < 1.5)}' /proc/loadavg; do sleep 10; done`), and never compare
  across batches.

## What a Mode B looks like here

Shipping with `Op::Fail` instead of `Op::Interp` for unmodelled shapes; a differential that compares
booleans but not errors; row 3 reported as "env-builds → 0" without the fallback counter beside it.
Each of those is the stone passing its gate while narrowing what `where` accepts — the exact failure
`compiled_cond`'s own doc warned about, one module over.
