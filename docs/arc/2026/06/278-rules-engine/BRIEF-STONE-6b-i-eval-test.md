# BRIEF — Stone 6b-i: `eval-test` (the `where` runtime evaluator)

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

Add `(:wat::rete::eval-test <quoted-expr: :wat::WatAST> <bindings: :wat::core::PersistentMap>) ->
:wat::core::bool` — the runtime evaluator for `where`/`:test` predicates. It evaluates the expr against a
token's merged bindings (`?var → value`) by building a CHILD `Environment` that binds each `?var` to its
value, then calling `eval_inner`; the result MUST be `Value::bool` (else a `TypeMismatch`). This is a
standalone primitive — **no compile/network changes** (TestNode + the fence are 6b-ii). Contract +
rationale: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-6b-where-test.md` — read it first.

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-6b-where-test.md` — the contract (the mechanism section).
2. `src/rete/purity.rs` `eval_pure_predicate` (~line 330) AND `src/rete/matcher.rs` `eval_alpha_match`
   (:85-150) — the sibling pattern: arity-check, eval `args[0]` → `Value::wat__WatAST(a)` → `(**a).clone()`,
   eval `args[1]` → the typed value, return. **Copy this shape.**
3. `src/value/environment.rs` (:108-198) — the `Environment` API: `env.child() -> EnvBuilder`,
   `.bind_unknown_span(name: impl Into<String>, tv: TrackedValue) -> EnvBuilder`, `.build() -> Environment`,
   `.lookup`. This is how you build the test env.
4. `src/runtime.rs` `eval_let` / `bind_let_binding` (~:6485) — the worked model for building a scope and
   for **constructing a `TrackedValue` from a `Value`** (ground the exact constructor — `TrackedValue::from`
   / `::untracked` / similar — do NOT guess it).
5. How to iterate a `:wat::core::PersistentMap` value: it is `Value::wat__core__PersistentMap(rpds::HashTrieMapSync<Value, Value>)`;
   `.iter()` yields `(&Value, &Value)`. Keys are `Value::String("?x")`. Ground the variant + iteration in
   `src/rete/matcher.rs` (it already reads these maps) or `src/collection/`.
6. `src/runtime.rs:~4020` — the rete dispatch arms (`pure?`/`deterministic?`/`alpha-match`/…). Add
   `":wat::rete::eval-test" => crate::rete::matcher::eval_test(args, list_span, env, sym),`.
7. `src/check.rs:~18994` — the rete TypeSchemes (`pure?`/`deterministic?`). Add one for `:wat::rete::eval-test`:
   params `[:wat::WatAST, :wat::core::PersistentMap]`, ret `:wat::core::bool`.
8. `tests/probe_arc278_6b_eval_test.rs` — the 7 assertions to green (do NOT edit it).

## Implementation sketch (fill it; don't invent the shape)

In `src/rete/matcher.rs`:
```rust
pub(crate) fn eval_test(
    args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::eval-test";
    // arity 2 (else ArityMismatch)
    // expr_ast: eval args[0] → Value::wat__WatAST(a) → (**a).clone()  (else TypeMismatch)
    // bindings: eval args[1] → Value::wat__core__PersistentMap(map)   (else TypeMismatch)
    // build a child env binding each ?var → value:
    let mut b = env.child();
    for (k, v) in map.iter() {
        let name = match k { Value::String(s) => s.as_str().to_string(), _ => continue };
        b = b.bind_unknown_span(name, /* TrackedValue from v.clone() — ground the ctor */);
    }
    let test_env = b.build();
    // eval the expr in test_env; result MUST be bool
    match crate::runtime::eval_inner(&expr_ast, &test_env, sym)?.value_owned() {
        Value::bool(x) => Ok(Value::bool(x)),
        other => Err(/* TypeMismatch: OP, expected ":wat::core::bool (a where predicate)", got other */),
    }
}
```

## Blast radius (bounded)

- Edit: `src/rete/matcher.rs` (+`eval_test`), `src/runtime.rs` (one dispatch arm), `src/check.rs` (one
  TypeScheme). NOTHING else. NO `rete.wat`, NO `kernel.rs`, NO `purity.rs`, NO new `Value` variant.

## STOP triggers (halt + surface; do not improvise)

1. **If `eval_inner` does not resolve a `?`-prefixed `Symbol` from the child-env binding** (e.g. the
   resolve layer rejects `?`-names, or `lookup("?x")` misses) — STOP and report exactly what happens. This
   is the load-bearing assumption (the quoted expr's `?x` must resolve to the bound value). Do NOT invent a
   substitution workaround; surface it.
2. If a `TrackedValue` cannot be constructed from a `Value` via a clean existing constructor — STOP, report
   the API you found.
3. If greening any assertion needs touching `rete.wat`/`kernel.rs`/`purity.rs` — STOP (that's 6b-ii scope).

## Done = green

`cargo test --release -p wat --test probe_arc278_6b_eval_test` → 7/7. Then the floors (EXPECTATIONS).
