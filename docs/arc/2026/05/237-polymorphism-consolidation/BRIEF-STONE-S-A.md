# BRIEF — Stone S-A — the is-a hierarchy mechanism (`typesub` + `subtype?`)

**Status:** READY TO SPAWN. `model: "sonnet"`.

## What to do

Mint the substrate's **is-a hierarchy** — Clojure's `derive`/`isa?` axis. Three
pieces, all additive, NO `unify`-site edits, NO `conforms?` change:

1. `TypeEnv` gains a `typesub` child→parent edge-registry + `register_subtype` (cycle-rejecting).
2. `pub fn is_subtype(sub, sup, env) -> bool` — the directional, transitive, reflexive walk over that registry.
3. `:wat::core::subtype?` wat primitive (`keyword × keyword -> bool`) over `is_subtype`, + seed the two built-in roots: `:wat::holon::Record typesub :wat::Record`.

Make `tests/probe_arc237_sA_hierarchy.rs` go **10/10**. It is committed
(`f77517ff`) and pins the exact API. This is the working contract you satisfy.

**NOT new-paradigm territory — it mirrors the proven `:wat::core::conforms?`
mint (Stone 237.5) almost exactly.** Copy that shape.

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-A-records-hierarchy.md` — the sub-DESIGN: algorithm, error contract, the **Proven-moves template + trap-doors section**, out-of-scope. READ THE REFINEMENT BANNER AT TOP (conforms? is NOT in scope).
2. `tests/probe_arc237_sA_hierarchy.rs` — **LOAD-BEARING** 10 contracts. The API you must build is exactly what this file calls: `env.register_subtype(child, parent) -> Result`, `wat::types::is_subtype(sub, sup, &env) -> bool`, `:wat::core::subtype?`, and `TypeEnv::with_builtins()` carrying the seeded root edge. Pre-stone: 6 compile errors (the missing API). Post-stone: 10/10.
3. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.5.md` — the comparable mint; **mirror its SCORE structure exactly**.
4. `src/types.rs:177` (`UnionDef`) + `:1267` (`:wat::Record` opaque-struct registration in `register_builtin_types`) + `:2975` (`check_union_no_cycle` — the cycle-check pattern to mirror for `register_subtype`) + `:3031` (`collect_union_members` — the free-fn placement to mirror for `is_subtype`; do NOT call it).
5. `src/runtime.rs:5291` (`":wat::core::conforms?" => eval_conforms` dispatch arm — add `subtype?` beside it) + `:16087` (`eval_conforms` — the `eval_X` shape to mirror; how it acquires `sym.types()` and reads its type-keyword arg).
6. `src/check.rs:5561` (`":wat::core::conforms?"` `infer_list` special-case arm — mirror for `subtype?`) + `:19317` (`conforms?` `TypeScheme` in `register_builtins` — add `subtype?` beside it).

## Implementation sketch

### `src/types.rs`

```rust
// On TypeEnv: the new edge-registry (child FQDN → parent FQDNs).
// (a HashMap<String, Vec<String>> field, default empty)

pub fn register_subtype(&mut self, child: &str, parent: &str) -> Result<(), TypeError> {
    // Reject if adding child→parent closes a cycle: i.e. parent already
    // (transitively) is_subtype-of child. Mirror check_union_no_cycle (:2975).
    if is_subtype(parent, child, self) {
        return Err(/* a cycle-rejection TypeError — mirror CyclicUnion shape:
                      a new variant CyclicSubtype { child, parent, span } OR
                      reuse an existing cyclic variant if it fits cleanly */);
    }
    self.subtype_edges.entry(child.to_string()).or_default().push(parent.to_string());
    Ok(())
}

// Free fn (mirror collect_union_members placement). NOT collect_union_members.
pub fn is_subtype(sub: &str, sup: &str, env: &TypeEnv) -> bool {
    if sub == sup { return true; }                       // reflexive
    let mut visited = std::collections::HashSet::new();
    let mut stack: Vec<&str> = env.subtype_parents(sub); // direct parents
    while let Some(p) = stack.pop() {
        if p == sup { return true; }
        if !visited.insert(p.to_string()) { continue; }
        stack.extend(env.subtype_parents(p));            // transitive
    }
    false
}
```

In `register_builtin_types` (near `:1267`), after `:wat::Record`:
- register `:wat::holon::Record` as an opaque zero-field `TypeDef::Struct` (mirror `:wat::Record` exactly), THEN
- `env.register_subtype(":wat::holon::Record", ":wat::Record")` (use the builtin/privileged path; `unwrap()`/`expect()` is fine here — it cannot cycle).

NOTE (expected, not a surprise): registering `:wat::holon::Record` as a struct
means `register_type_predicates` will synthesize `:wat::holon::is-Record?` for it
(same as `:wat::Record` already gets `:wat::is-Record?`). That's correct — it is a
type. Mention it in the SCORE honest-deltas.

### `src/runtime.rs`

Dispatch arm beside conforms? (`:5291`):
```rust
":wat::core::subtype?" => eval_subtype(args, list_span, env, sym),
```

`eval_subtype` (mirror `eval_conforms` @16087):
- arity-2 (else `RuntimeError::ArityMismatch`-style, mirror eval_conforms).
- BOTH args are **type-position keywords taken literally** (NOT evaluated as
  values). Extract the keyword path string from each `WatAST::Keyword`; if either
  arg isn't a keyword AST → error.
- acquire `let types = sym.types().ok_or_else(|| ...)?;` (the runtime TypeEnv
  access pattern eval_conforms uses).
- **Validate both names are known** (in `types` OR `is_builtin_primitive(stripped)`)
  → else `Err(unknown type name …)`. This keeps `false` honest (probe 10). Mirror
  conforms?'s unknown-name error contract.
- `Ok(Value::bool(is_subtype(a, b, types)))`.

Do NOT touch `conforms_check` / `eval_conforms` (S-B).

### `src/check.rs`

`register_builtins` TypeScheme beside conforms? (`:19317`):
```rust
env.register(":wat::core::subtype?".into(), TypeScheme {
    type_params: vec![],                                 // no T — both args are keywords
    params: vec![keyword_ty(), keyword_ty()],            // :wat::core::keyword × :wat::core::keyword
    ret: bool_ty(),
    rest_param_type: None,
});
```
(use the existing `keyword_ty()`/`bool_ty()` helpers if present; else
`TypeExpr::Path(":wat::core::keyword".into())` / `":wat::core::bool"`.)

`infer_list` special-case arm beside conforms? (`:5561`) — **load-bearing**:
```rust
":wat::core::subtype?" => {
    // arity-2 check; push CheckError on mismatch
    // validate args[0] AND args[1] are WatAST::Keyword(_,_); skip inference on both
    //   (the type-keyword-infers-as-Fn trap: a keyword naming a constructor would
    //    otherwise infer as Fn(...) and fail unification with :keyword)
    return CheckResult::ok(bool result type);  // mirror the conforms? arm's return
}
```

## Discipline

- Modify **`src/types.rs` + `src/runtime.rs` + `src/check.rs` ONLY**.
- NO new `Value` variant. NO `conforms?` change (S-B). NO `unify`-site edits (S-A1). NO holon-rs (STOP-5).
- `is_subtype` walks the **new `subtype_edges` registry** — it must NOT call `collect_union_members`. The hierarchy is a distinct relation from union membership.
- A new `TypeError` variant for cycle rejection (e.g. `CyclicSubtype`) is allowed if no existing variant fits; if you add one, fix any exhaustiveness cascade it forces (expected: the `TypeError` Display/match sites — mirror how `CyclicUnion` is handled).

## STOP triggers (REJECTION — not permission to defer)

1. Compile errors not traced to a probe-named contract.
2. Lib baseline drops below 827.
3. 90 min elapsed (STOP-3); 120 min (STOP-4 hard kill).
4. holon-rs touched (STOP-5).
5. Files outside `src/types.rs` + `src/runtime.rs` + `src/check.rs` touched (a forced `TypeError` exhaustiveness cascade into the same files' match arms is fine; a NEW file is not).
6. Probe doesn't reach 10/10.
7. Any arc-237 predecessor probe regresses (237.1 / 237.5 / 237.6).
8. You find yourself making `is_subtype` call `collect_union_members`, OR adding a new `Value` variant, OR touching `conforms_check`/`unify` — STOP; none of those is in scope.

## FM 2-bis evidence

`tests/probe_arc237_sA_hierarchy.rs` (committed `f77517ff`) — 10 contracts.
Pre-stone: 6 compile errors (`wat::types::is_subtype` unresolved + 5×
`register_subtype` no-method); the harness imports (`eval_in_frozen`,
`parse_one!`, `with_builtins`, `Value`) all resolve. Post-stone: 10/10 PASS.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-A.md` (NEW). Mirror
Stone 237.5 SCORE structure: scorecard (compile-clean row; **S-A probe 10/10
LOAD-BEARING**; lib baseline 827; 237.1/237.5/237.6 regression guards; holon-rs
untouched) → Final API shape → Line count → Cascade depth → Honest deltas (incl.
the `:wat::holon::is-Record?` auto-synthesis note + any `TypeError` cascade) →
Working tree on return. DO NOT commit (orchestrator commits).

## Calibration

Mirror of the proven 237.5 mint, simpler walker (flat parent-chain, no grammar
recursion) + a `TypeEnv` registry field + two seeded roots. Sits in the
234.0/237.5 tier (38 min / in-band-of-40–75). **Target band: 40–70 min Mode A;
90 STOP-3; 120 STOP-4. Cascade: 2 rounds (types.rs → runtime.rs/check.rs), 0–1
forced files (only if a new `TypeError` variant cascades within the 3 files).**
Per `feedback_stone_briefs_cite_prior_score`: SCORE-STONE-237.5 is the shape.
