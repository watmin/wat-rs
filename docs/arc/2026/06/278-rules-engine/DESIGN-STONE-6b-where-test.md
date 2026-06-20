# DESIGN — Stone 6b: `where`/`:test` (the TestNode) over the 6a fence

**Status:** STRIKE-READY (6b-i drawn; 6b-ii follows)
**Depends on:** 6a (`pure?` + `deterministic?` — committed `78877c77`)
**Consumed by:** 7 (negation conditions may carry tests), 8 (accumulator `acc/`-in-LHS predicates).

## Why

A rule's LHS needs predicates beyond equality joins: `(where (:wat::core::string::starts-with? ?path "/admin"))`,
`(where (:wat::core::> (:wat::core::- ?hi ?lo) 10))`, `(where (:my::suspicious? ?req))`. This is Clara's
`:test`. The builder's grounding: *"who reaches for a performant rete and not use their own fn for
filtering?"* — so `where` must accept **any pure ∧ deterministic boolean expression** (intrinsics AND
user fns), which is exactly what 6a now classifies.

## Surface (DESIGN.md:314 — locked)

`(:wat::rete::where <expr>)` — a condition in `defrule`'s `:when` vector, alongside type conditions:

```
(:wat::rete::defrule :alert::big-gap
  :when
  [(:weather::Temperature (?hi <- :celsius) (:location "inland"))
   (:weather::Temperature (?lo <- :celsius) (:location "coast"))
   (:wat::rete::where (:wat::core::> (:wat::core::- ?hi ?lo) 10))]
  :then
  (:wat::rete::insert (:alert::BigGap ?hi ?lo)))
```

`<expr>` evaluates to `:bool` against the token's merged bindings (`?hi`, `?lo` from the prior
conditions). It must be **pure ∧ deterministic** — enforced at COMPILE (the fence), so a side-effecting
or non-deterministic condition is *uncompilable*, not a fire-time surprise.

## The mechanism — `eval-test`, evaluate against the merged bindings

`eval-test` is the runtime evaluator, a shared Rust primitive (oracle + native both call it; same pattern
as `alpha-match`/`eval-insert`):

```
(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) -> :wat::core::bool
```

Implementation: build a CHILD `Environment` binding each `?var → value` from `bindings` (the
`Environment::child().bind_unknown_span(name, tv).build()` API; `eval_inner` resolves `Symbol("?c")` via
`env.lookup`), then `eval_inner(expr, &test_env, sym)`. The result MUST be `Value::bool` (else
`TypeMismatch` — a `where` is a predicate). Because the fence proved the expr pure ∧ deterministic at
compile, fire-time `eval_inner` is safe (no IO, deterministic) — no runtime purity mode needed.

> The bindings keys are the `?`-prefixed names exactly as stored in the token (`"?hi"`); the expr
> references them as `Symbol("?hi")` → `env.lookup("?hi")`. Keys match by construction.

## Scope decision — TestNode now; ExpressionJoinNode banked (deviate UP, name the deferral)

DESIGN.md stone 6 named "TestNode + ExpressionJoinNode." **TestNode is the general mechanism**; an inline
cross-condition non-equality constraint `(:Type (?a <- :f) (> ?a ?prior))` is *functionally* a `where`
placed after both vars bind — so `(where (> ?a ?prior))` covers it. **ExpressionJoinNode** (fusing the
filter INTO the hash-join node so it filters during the join instead of as a separate downstream node) is
a **perf optimization** of the same semantics → banked as a named follow-on (`6b-perf`), not a capability
gap. Shipping TestNode is deviate-UP (the general, hard part); the fusion is the optimization.

## Decomposition

- **6b-i — `eval-test` (the runtime evaluator), standalone.** The Rust primitive only — eval a bool expr
  against a bindings `PersistentMap` via a built child env. No compile changes (no entanglement with
  `compile-condition`). Independently testable: call `eval-test` directly with a quoted expr + a
  `PersistentMap` of `?var → value`.
- **6b-ii-a — TestNode in the ORACLE + the compile fence.** The `:wat::rete::TestNode` record (`[id expr
  children]`) + `Node` defenum variant + `compile-condition` top-branch (a `(where expr)` cond → **fence**
  `pure? ∧ deterministic?` (raise on fail) → mint a TestNode wired parent→test, advance parent=test-id) +
  a **test-pass** in `fire-once` (between hash-join and production: for each TestNode, filter
  `beta-memory[parent]` by `eval-test(expr, token.bindings)` into `beta-memory[test-id]`). Probed via the
  oracle (`fire-rules-spec`): `(where (> ?c 0))` passes Temp(5) / blocks Temp(-5); a user-fn predicate
  passes; an impure `where` fails to compile.
- **6b-ii-b — TestNode in the NATIVE kernel + the differential.** The `kernel.rs` test-pass (the same
  filter over the native `WorkingMemory`) so `fire-rules'` honors TestNode + the **differential** probe
  (native derived facts == oracle, with a `where` in the rule). North-star: cold-and-windy + `(where (> ?c
  -50))` fires / `(where (> ?c 100))` does not, native==oracle.

## Out of scope = rejected

- **ExpressionJoinNode** (inline-constraint-fused-into-join) → banked `6b-perf` (named above).
- **A runtime purity mode** — not needed; purity is proven at compile (6a fence), fire trusts it.
- **Compile-time bool-typing of `<expr>`** — `eval-test` runtime-checks `Value::bool`; a static
  bool-type check is a nicety, banked.

## Files (6b-i — eval-test only)

- `src/rete/matcher.rs` — `eval_test` (the primitive; `Environment::child().bind_unknown_span(…).build()`
  over the bindings map, then `eval_inner`; result must be `Value::bool` else `TypeMismatch`).
- `src/runtime.rs` — one dispatch arm `:wat::rete::eval-test` (forced-minimal, beside the sibling rete arms).
- `src/check.rs` — one TypeScheme (`[:wat::WatAST, :wat::core::PersistentMap] -> :wat::core::bool`).
- `tests/probe_arc278_6b_eval_test.rs` — the RED probe.

(6b-ii adds: the `TestNode` record + `compile-condition` `where`-branch + the pure∧det fence +
oracle/native fire passes + the differential probe.)
