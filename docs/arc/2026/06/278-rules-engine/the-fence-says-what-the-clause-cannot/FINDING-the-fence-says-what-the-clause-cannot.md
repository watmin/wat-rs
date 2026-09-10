# FINDING — the fence admits predicates the clause position refuses

## Origin

The `compute` host (`sns-sqs`) reported `src/rete/validate/mod.rs:282`'s empty arm:

```rust
// Design call 3 — a `where` fence's outer shape is already confirmed by the
// classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
ReteClauseShape::Where(_) => {}
```

Its own finding names the cost precisely, and the diagnosis is **correct**:

> *"`durable-binder-nsubs` joins **three `Node` patterns on one shared `?svc`**. For a form with C
> children that materialises **~C³ tuples** before `?hi = 0` or `?sn = ":demo::topic"` ever runs —
> and those two would have cut it to almost nothing."*

## 1 — The cost is real, and measured in a second engine

A Clara 0.24.0 lab (three `Node` patterns joining on one shared var; the two predicates written
first inline, then as `:test` after the join):

| C | C³ | inline (alpha) | `:test` (beta) | ratio |
|---|---|---|---|---|
| 12 | 1,728 | 19.6 ms | 20.4 ms | 1.04× |
| 24 | 13,824 | 37.0 ms | 51.7 ms | 1.40× |
| 36 | 46,656 | 47.8 ms | **101.7 ms** | **2.13×** |

Match counts identical at every C. **The gap widens with C**, so this is the join blowup, not noise.

⭐ **Clara does not hoist** — if it did, the two lines would not diverge. **And Clara does not refuse
the slow form**: no warning, no error. So the footgun is inherent to rete, and there is **no
precedent in the reference implementation for refusing it.**

## 2 — The two forms are NOT interchangeable

A check was built (recoverable at `f47a9fccc`) refusing a fence whose every var is bound by exactly
one ordinary condition. It flagged **221 sites in 60 of 77 files**. A `wat-fix` codemod
(`wat-scripts/fixes/hoist-where-into-condition.wat`) migrated the `tests/` half — 63 sites, 32 files,
dry-run diffed, idempotent, re-verified clean against the check-enabled binary.

**The floor then went red with 24 failures, none traceable to the codemod.** Four mechanisms:

| n | mechanism |
|---|---|
| **9** | an inline clause's grammar **does not admit an arbitrary user-function call** the way a fence does — `expr_is_provably_boolean` never covers foreign calls |
| **2** | negative fixtures asserting the fence's *specific* Law-A/totality diagnostic get a generic `MalformedClause` once inlined |
| **1** | a termination-bounding fence is **not the same input to the round-cap verifier** once inlined — a genuine admit→refuse regression |
| **1** | an alpha condition will not compile a `reduce`+`fn` closure the way a beta test does |
| 10 | one file's scaffolding depended on the fence's compiled export node, unrelated to hoisting |

⛔ **So the equivalence the report proved on one probe does not generalise.** And the termination case
means even a *compiler-side* hoist would not be semantically free.

## 3 — Clara says more than we can

The same lab, testing what each engine admits **inline**:

| shape | Clara | wat |
|---|---|---|
| a bare user fn — `(interesting? kind)` | ✅ | ⛔ |
| a fn nested in a comparison — `(> (score kind) 5)` | ✅ | ⛔ |
| a closure / HOF — `(some #(= % kind) [...])` | ✅ | ⛔ |

**In Clara the slow form is a choice. In wat it is sometimes the only sayable thing.** That reframes
the whole finding: **26 of 27 recorded fixes use the trailing fence** not because 26 authors erred,
but because the fast form was unreachable for that predicate.

⚠ **The 26/27 figure is NOT evidence about ergonomics.** We are the only wat users; it is one
exemplar copied, not twenty-six independent judgements. It is quoted here only to be struck.

## 4 — The expressivity corpus found the axis and closed the wrong half

`wat-scripts/perf/grid/where-*.wat` is 38 files. One of them, `where-inline-computed.wat`, exists
*because of this class* and says so:

> *"Every one of the other 36 `where-*` axes writes its predicate in a **fence**. Measured
> 2026-08-28: the wat side of the whole grid contained **ZERO** inline constraints… fix-list entry F
> lived in the other one: an inline constraint whose operand was a nested call compiled to a
> permanent, **SILENT never-match**… **39 of 77 vocabulary rows wide.** A shape corpus that only ever
> spells things one way cannot see a defect in the other spelling."*

That fixture pairs each predicate **across both positions and requires them to agree.**

⛔ **Agreement is not admissibility.** Pairing only ever covers predicates that work in *both*
positions — the ones that work in **only one** never enter the corpus to be paired. **A corpus built
from expressible cases cannot see the inexpressible.** That is the half still open, and it is where
the nine failures live.
