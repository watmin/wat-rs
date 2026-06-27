# BRIEF — Stone 6a: purity inference (`:wat::rete::pure?`)

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat` (the binary is orchestrator-only; you may `cargo build`/`cargo test`).**

## The work (one paragraph)

Build a **default-deny purity classifier** for rete's capability tier and expose it as `(:wat::rete::pure? <quoted-expr>) -> :bool`. A new home `src/rete/purity.rs` holds `is_pure_expr(ast, sym) -> bool` and `is_pure_fn(fqdn, sym) -> bool`: an expression is pure iff every function head in it is *proven* pure — a known-pure intrinsic, or a user fn whose body is *transitively* pure — and **anything unproven is rejected** (effectful namespaces, the non-deterministic `Uuid/v4`/`v5`, unknown heads). Wire it as one dispatch arm beside the sibling rete primitives. The contract + the full decision rationale are in `DESIGN-STONE-6a-purity-inference.md` — read it first.

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-6a-purity-inference.md` — the contract (default-deny, the per-head decision, the allow-list, cycle handling). Authoritative.
2. `src/runtime.rs:22519` — `is_effectful_op` (the known-impure namespace seed). **Change its signature `fn` → `pub(crate) fn`** (one visibility edit, no logic) so `purity.rs` consumes it (single source of truth for the effect surface).
3. `src/rete/matcher.rs:85-150` (`eval_alpha_match`) — the sibling primitive pattern: how a rete primitive evaluates `args[0]` to a `WatAST` value (from `quote`) and validates it. **Copy this shape** for `eval_pure_predicate`.
4. `src/runtime.rs:3997-4019` — the rete dispatch arms (`alpha-match`, `eval-insert`, `step-payload'`). **Add** `":wat::rete::pure?" => crate::rete::purity::eval_pure_predicate(args, list_span, env, sym),` here (~4001).
5. `src/rete/mod.rs` — add `mod purity;` (match the existing `mod` style, pub(crate) as siblings).
6. `src/check.rs` — find the TypeScheme for `:wat::rete::step-payload'` / the sibling rete primitives; **add one** for `:wat::rete::pure?` : argument `:wat::WatAST`, return `:wat::core::bool`. (Forced-minimal megafile touch — one scheme entry beside the siblings.)
7. How a user fn's **body AST** is reached from `sym` — ground via `step_user_call` (`runtime.rs`, called at ~22506) or `apply_function`; `is_pure_fn` reads the fn's body to recurse. Read enough to extract the body slice correctly; do not guess the field shape.
8. `tests/probe_arc278_6a_purity.rs` — the RED probe (the 8 contract assertions). Make it GREEN. Do not edit it except to un-`#[ignore]` if you add ignores (you should not need to).

## Implementation sketch (fill it; don't invent the shape)

In `src/rete/purity.rs`:
```rust
// Non-deterministic intrinsics that sit OUTSIDE is_effectful_op's namespaces.
// "pure" = deterministic fn of facts; randomness is impure even with no IO.
// ONLY Uuid/v4 is random. Uuid/v5 (SHA1 of ns+name), from-string, to-string, nil are
// all DETERMINISTIC ⇒ pure (they belong on the allow-list, NOT here).
fn is_nondeterministic(head: &str) -> bool {
    matches!(head, ":wat::core::Uuid/v4")
}

// The curated pure-intrinsic allow-list (default-deny: unknown ⇒ NOT pure).
// Enumerate from the dispatch table (dispatch_keyword_head_value) within these categories ONLY:
//   - prefix-pure namespaces: ":wat::core::string::", ":wat::core::regex::"
//   - explicit pure ":wat::core::" ops: arithmetic (+ - * / mod ...), comparison (< > <= >= = not=),
//     boolean (and or not), collection/map/vector readers+predicates (get contains? length empty?
//     nth first second third keys vals ...), type predicates.
// INCLUDE the deterministic Uuid ops (Uuid/v5, from-string, to-string, nil); EXCLUDE only
// Uuid/v4 (random, handled by is_nondeterministic) and anything not in these categories.
fn is_pure_intrinsic(head: &str) -> bool { /* ... */ }

pub(crate) fn is_pure_fn(fqdn: &str, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    if seen.contains(fqdn) { return true; }      // back-edge: no NEW impurity (purity fixpoint)
    seen.insert(fqdn.to_string());
    // look up the fn body AST in sym, then is_pure_expr(body, sym, seen)
}

pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    // literals / keywords / symbols (incl ?vars) → true (data, not a call)
    // List with a keyword/symbol head H:
    //   is_effectful_op(H) || is_nondeterministic(H)  → false
    //   sym.functions has H (user fn)                 → is_pure_fn(H, sym, seen) && all args pure
    //   is_pure_intrinsic(H)                          → all args pure
    //   else (unknown)                                → false   (DEFAULT-DENY)
    // quote/quasiquote sub-forms are DATA → pure (do not recurse into them as calls)
    // vectors/maps → recurse element-wise
}

pub(crate) fn eval_pure_predicate(args, list_span, env, sym) -> Result<Value, EvalBreak> {
    // arity 1; eval args[0] → expect Value::wat__WatAST(a) (else TypeMismatch, like eval_alpha_match);
    // Ok(Value::bool(is_pure_expr(a.as_ref(), sym, &mut HashSet::new())))
}
```
Provide a public `is_pure_expr`/`is_pure_fn` wrapper (or a 2-arg form that seeds the `seen` set) so 6b can call it without threading `seen`.

## Blast radius (bounded)

- NEW: `src/rete/purity.rs`. Edit: `src/rete/mod.rs` (+`mod purity;`), `src/runtime.rs` (one visibility change + one dispatch arm — NOTHING else; it is a megafile under forced-minimal-touch discipline), `src/check.rs` (one TypeScheme — forced-minimal). NO change to `matcher.rs`/`kernel.rs`/`collect.rs`/`rete.wat`/`core.wat`.
- No new dependency. No new `Value` variant. No mutation primitive.

## STOP triggers (halt and surface — do not improvise)

1. If a user fn's body AST cannot be reached cleanly from `sym` (the field shape differs from what `step_user_call` implies) — STOP and report what `sym.functions` actually stores; do not fabricate an accessor.
2. If making `is_effectful_op` `pub(crate)` requires touching its *logic* (not just visibility) — STOP; the seam was supposed to be a one-line visibility change.
3. If the `pure?` TypeScheme can't be expressed as `:wat::WatAST -> :wat::core::bool` beside the sibling rete schemes — STOP and report the sibling scheme shape you found.
4. If any probe assertion can only be made green by weakening the classifier to default-ALLOW (e.g. dropping the `Uuid/v4` or transitive checks) — STOP. Default-deny + the transitive/Uuid checks are the contract; report the obstacle.

## Done = the probe is green + the floors hold

`cargo test --release -p wat --test probe_arc278_6a_purity` → 8/8. Then the floors below (EXPECTATIONS).
