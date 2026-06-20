# DESIGN — Stone 8: accumulators (`acc/…`) — the AccumulateNode

**Status:** STRIKE-READY (8-i drawn; 8-a/8-b follow)
**Depends on:** the join machinery (`token-element-compatible?`, alpha-memory) + the filter-pass shape
(6b/7). The LAST capability stone — closes the tier ④ Clara gap.

## Why

Aggregation: "fire on a *summary* of matching facts." `(?n <- (acc/count) :from (:Event …))` — bind `?n`
to the count of matching Events. Clara's accumulators; the DDoS primitive the builder named — "a minimum
finding set to activate" — is `(acc/count) :from …` + `(where (>= ?n N))`, i.e. it **composes** from
stone 6 + 8, not a separate feature.

## The scope decision (four-questions, chosen — same shape as 6/7)

**Pure replay dissolves the `retract-fn`.** Clara's `AccumulateNode` "uses `:retract-fn` OR
**re-accumulates**" (CLARA-REF §7). We always re-accumulate (each fire recomputes from facts), so the
`retract-fn` (incremental aggregate update on support loss — the hard part, hazards #3/#5) is the
*optimization*, not the correctness path → **banked** (`8-perf`). Same dissolution as TM (P4c) and
negation's two-sided delta (stone 7).

**One node, not two** (deviate-UP). Clara splits `AccumulateNode` (no cross-refs) +
`AccumulateWithJoinFilterNode` (cross-refs ancestor bindings). Our `token-element-compatible?` IS the
join-filter, so **one `AccumulateNode`** gathers the compatible `:from` elements per token and folds them.

**Re-accumulate is correct + simple:** an AccumulateNode is the JOIN family with an aggregate fold —
gather the token-compatible elements from the `:from` alpha-memory, fold them with the accumulator into a
reduced value, **extend the token** with the result var, pass downstream. (Negation gathers-and-tests-empty;
accumulate gathers-and-folds. Same `token-element-compatible?` gather.)

## Surface (PROPOSED — the one thing to confirm; Clara-faithful)

```
(:wat::rete::defrule :alert::busy
  :when
  [(:weather::Station (?loc <- :location))
   (?n <- (:wat::rete::acc::count) :from (:weather::Reading (?loc <- :location)))]
  :then
  (:wat::rete::insert (:alert::Busy ?loc ?n)))
```

`(?result <- (<accumulator>) :from (:FactType <clause>…))` — binds `?result` to the accumulator's reduced
value over the `:from` facts compatible with the token's bindings (shared `?loc`). Detected in
`compile-condition` by a `?`-symbol head + `<-` (vs keyword head for type/where/not).

**Value accumulators take a BOUND `?var` (the primitive), not a `:field` keyword (four-questions, decided).**
The `:from` condition binds the field to a `?var` like every other condition; the accumulator folds that
var (the wat folds read it from each element's bindings map — no field-read magic, uniform with the DSL):

```
(?n     <- (:wat::rete::acc::count)         :from (:net::Packet (?ip <- :src)))
(?total <- (:wat::rete::acc::sum ?bytes)    :from (:net::Packet (?ip <- :src) (?bytes <- :size)))
(?avg   <- (:wat::rete::acc::mean ?bytes)   :from (:net::Packet (?ip <- :src) (?bytes <- :size)))
(?ports <- (:wat::rete::acc::distinct ?port):from (:net::Packet (?ip <- :src) (?port  <- :dst-port)))
(?pkts  <- (:wat::rete::acc::all)           :from (:net::Packet (?ip <- :src)))
```

`count`/`all` take no value arg. The Clara-familiar `:field` form (`(acc/sum :size)`) is **sugar** over
this primitive — `(acc/sum :size)` macro-expands to gensym `?g` + inject `(?g <- :size)` into `:from` +
`(acc/sum ?g)` — the kwargs-is-always-a-macro pattern: lean uniform substrate, ergonomics on top. Banked
`8-field-sugar`. (The `acc/` namespace shorthand is also banked sugar; v1 uses the FQDN.)

## The accumulator set (DESIGN.md:351 — KEEP ALL; all simple folds over the compatible set)

`count` · `sum :field` · `min :field` · `max :field` · `mean :field` · `distinct :field` · `all` ·
`group-by :field`. Each is a trivial fold once the node + gather work. **Empty-set behavior** (the
`::not-reduced` subtlety): `count`/`sum` emit their identity (0) on empty; `all`/`distinct`→empty
collection; `group-by`→empty map; `min`/`max`/`mean` emit **no token** on empty (no min of nothing) —
each accumulator declares its empty result (or its absence). v1 implements these; the richer
`::not-reduced` four-case edge handling is banked if a probe surfaces a gap.

## The accumulators are simple pure folds, composed (builder, 2026-06-20)

The "complex" accumulator surface is just **named pure folds over the gathered set** — the complexity is
*composition under a simple surface*, not special engine primitives:

- `count` = `length` · `sum` = `(reduce + 0 (map :field …))` · `min`/`max` = `reduce` with `min`/`max`
- **`mean` = `(/ (sum …) (count …))`** — literally composed from two other accumulators
- `distinct` = dedupe the field values · `all` = the gathered set · `group-by` = `reduce` into a map

A pure fold is a capability wat **already has** (`foldl`/`reduce`/`map` + `+`/`min`/`max`), so by the
"does a macro need it? — Rust-vs-wat-helper" boundary these are **pure wat fns**, NOT a Rust dispatch.
The AccumulateNode only needs to **gather** the token-compatible elements (the join machinery) and **apply**
the named fold. Fast Rust folds are a *measured* perf follow-on (`8-perf-folds`, differential-checked) —
the spec-first doctrine: ship the wat folds (the oracle/spec), measure, then Rust-ify only if hot.

## Decomposition (mirrors stone 6)

- **8-i — the wat accumulator fold library + apply, standalone.** A small wat library at
  `:wat::rete::acc::` — `count`/`sum`/`min`/`max`/`mean`(=sum∘count)/`distinct`/`all`/`group-by` as
  pure wat fns `(<acc> <args…> <elements: PV<Element>>) -> :Value`, plus a thin `apply-accumulator`
  (resolve the acc form + call it on the gathered elements; empty-set → the acc's `initial-value` or
  no-token). NO Rust dispatch (the folds are wat). Probed directly: each fold over a literal element set
  returns the right value; `mean` = the composition; empty-set behaviour per accumulator.
- **8-a — AccumulateNode in the ORACLE** (`rete.wat`): the `AccumulateNode` record + `Node` variant +
  `node-children` arm + `compile-condition` accumulator-branch (mint alpha for `:from`, mint AccumulateNode
  carrying result-var + acc-form + from-alpha-id, wire parent→accumulate) + an accumulate pass in
  `fire-once` (gather token-compatible `:from` elements → `apply-accumulator` (8-i) → extend token with the
  result var → pass; no token on empty-no-token accumulators). Probed via `fire-rules-spec`.
- **8-b — AccumulateNode in the NATIVE kernel** + the differential (native==oracle on an `acc/` rule).

## Out of scope = rejected (banked, named)

- **`retract-fn` / incremental aggregate update** — `8-perf` (replay re-accumulates; correctness needs no
  retract-fn).
- **`:field` form** (`(acc/sum :size)`) — `8-field-sugar`; a defrule-macro expansion to the `?var`
  primitive (gensym + inject the `:from` bind). v1 ships the `?var` primitive.
- **`acc/` namespace shorthand** (alias) — `8-sugar`; v1 uses the `:wat::rete::acc::` FQDN.
- **Custom `accumulator`** (user reduce/combine/init/convert) + **`reduce-to-accum`** — `8-custom`; v1 ships
  the built-in set. (Gated by 6a's purity fence — user fold fns must be pure∧deterministic.)
- **`:returns-fact`** (min/max return the fact, not the field value) — `8-returns-fact`.
- **`exists`** — NOT an accumulator (corrected): it's the NegationNode *flipped* (pass iff ≥1 compatible),
  a stone-7 sibling → `7-exists`. (Supersedes the stale `:exists ≡ (:not (:not X))` note — that never
  compiled in our engine: `:not`'s inner must be a fact-pattern, not a nested `:not`.)
- **Rich `::not-reduced` four-case handling** — banked unless a probe surfaces a concrete gap.

## Files (8-i — the wat fold library, NO Rust dispatch)

- `wat/rete.wat` — the `:wat::rete::acc::*` fold fns (`count`/`sum`/`min`/`max`/`mean`/`distinct`/`all`/
  `group-by`; `mean` = `(/ (sum…) (count…))`) over a `PV<Element>`, reading the bound `?var` from each
  element's bindings; + `apply-accumulator` (resolve the acc form's head + `?var` arg, call the fold,
  empty-set → `initial-value`-or-no-token). Pure wat; no new Rust primitive.
- `tests/probe_arc278_8i_accumulator_folds.rs` — the RED probe (each fold over a literal Element set).
