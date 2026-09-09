# DESIGN — the oracle sees negation only at the top level; the engine recurses

**Status:** drawn 2026-09-08. Resolves **`C1`** (conferre, L1) — the last behavioural L1 in
targets 1–2.

## The divergence, verified by reading both sides

**Oracle** (`wat/rete/oracle/stratify.wat:138-160`) folds over the LHS forms, takes each form's
head, and collects negated types **only when `hd == ":wat::rete::not"`**. A form whose head is
`and` or `or` contributes **nothing**, however deep a `not` sits inside it.

**Engine** (`src/rete/kernel/stratify.rs:103-122`) threads an `under_not` flag:

```rust
ReteClauseShape::Not(inner)               => negate_types(inner, out, true),
ReteClauseShape::And(xs) | ReteClauseShape::Or(xs) => { for x in xs { negate_types(x, out, under_not) } }
ReteClauseShape::FactBind { type_head, .. } if under_not => out.push(type_head.to_string()),
```

**On `(or A (and B (not C)))` the oracle contributes nothing and the engine pushes `C`** — different
negation edges, therefore different stratum numbers.

⭐ **The engine is the semantically correct side.** A `not` nested inside `and`/`or` *is* a negation
dependency: the rule depends on `C`'s absence. The oracle is the reference implementation and here
the reference is the wrong one.

## ⭐⭐ The shape is IN THE CORPUS — twice, with a comment naming it

Scanned all **927** `:when` clause vectors under `wat/`, `tests/`, `wat-scripts/`, `src/` for a
top-level `and`/`or` containing a `not`. **Two hits, both in
`wat-scripts/perf/grid/where-nested-combinators.wat`:**

```
;; 6 — an `:and` carrying a `:not`, as one arm of an `:or`. World 3 yields TWO activations.
(:wat::rete::defquery :wnc::q6 :params []
  :when [(:wat::rete::or (:wnc::A) (:wat::rete::and (:wnc::B) (:wat::rete::not (:wnc::C))))])
;; 7 — two negations conjoined.
(:wat::rete::defquery :wnc::q7 :params []
  :when [(:wat::rete::and (:wat::rete::not (:wnc::A)) (:wat::rete::not (:wnc::B)))])
```

`q6` is `C1`'s example **verbatim**, in a file whose whole purpose is nested combinators, in a grid
axis that compares oracle against native.

## Why no gate catches it — and this is the durable half

1. **The grid compares FACTS, not stratum numbers.** `stratify.rs:29-30` says so in its own words:
   *"THE FACTS AGREE … The two engines reach that agreement by DIFFERENT routes."* Stratification
   changes *order*; where the fixpoint is order-insensitive the facts still match.
2. **The one stratum-number differential walks no corpus.** `src/rete/kernel/tests/stratify_numbers.rs`
   (`native_stratify_numbers_against_the_oracle_scratch`) evaluates **two hand-written rule vectors**
   — `(:l23::ok) (:l23::neg)` and `(:l23::ok) (:l23::tally)`. It can only catch a divergence its
   author thought to write, and its name says `_scratch`.

**So the divergence is live, present in the corpus, and invisible by construction.** That is `3P1`'s
class one level down: an instrument that answers only the questions its corpus asks.

## ⚠ THE OPEN QUESTION THE PROBE MUST SETTLE FIRST

**`q6`/`q7` are `defquery`, not `defrule`.** Stratification orders *producers*, and a query has no
`:then`, so `rule_produces` returns empty for it. **Whether a query's own stratum is computed at all,
and whether anything reads it, is NOT established** — `rule_negates(&lhs)` is called at
`arm.rs:1148` over whatever the rules vector holds, but that does not prove a query is in it.

⛔ **Write the disconfirming probe FIRST.** Build a rule — not a query — of the shape
`(or A (and B (not C)))` with `C` derived, run both engines in one process the way
`stratify_numbers.rs` does, and print both stratum maps. **If the numbers agree, the divergence is
unreachable for rules and this strike shrinks to a comment.**

## What this delivers, if the probe confirms

1. **The oracle recurses** — `rule-negates` threads an under-not flag through `and`/`or`, mirroring
   `negate_types`. The oracle is `wat/`, so it is `include_str!`'d: **rebuild before testing.**
2. **The stratum differential stops being a scratch** — it gains the nested shape at minimum, and
   preferably reads real fixtures rather than two inline vectors.

## The one contract decision, pinned

⛔ **FIX THE ORACLE, NOT THE ENGINE.** The engine's behaviour is correct and is what ships; changing
it to match a less-correct reference would be curing a divergence by breaking the right side. The
oracle exists to be a checkable statement of intent, and here it understates the intent.

## Out of scope = rejected

- **Making the grid compare stratum numbers.** Real, and a different strike — the grid's contract is
  facts.
- **`3P1`'s axis-pair coverage.** Same class, its own row.
