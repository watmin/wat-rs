# BRIEF — Stone 237.5 — `:wat::core::conforms?` general type-conformance primitive

**Status:** READY TO SPAWN. `model: "sonnet"`.

## What to do

Mint `:wat::core::conforms?` — a runtime primitive that answers "does this value conform to this type expression?" — recursive over the `TypeExpr` grammar. This is the FOUNDATION; Stone 237.6's `is-<Name>?` auto-mint composes over it. Make the FM 2-bis probe (12 contracts) go 12/12.

```
(:wat::core::conforms? <value> :TypeExpr) -> :wat::core::bool
```

NOT new mechanism territory — it composes substrate pieces that all already exist (verified). The work is the recursive walker + dispatch registration + check scheme.

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.5.md` — sub-DESIGN: the algorithm, the error contract, out-of-scope.
2. `tests/probe_arc237_stone5_conforms.rs` — **LOAD-BEARING** 12 contracts; ALL must PASS. This is the contract you satisfy. (Pre-stone: 11/12 fail on `UnknownFunction(:wat::core::conforms?)` — that absence is the only gap.)
3. `src/runtime.rs:5112` (`":wat::core::type" => eval_type`) — the dispatch-registration + `eval_*` shape to MIRROR for `:wat::core::conforms?`.
4. `src/runtime.rs:7530` — the canonical **runtime TypeEnv access pattern**: `let types = sym.types().ok_or_else(|| RuntimeError::MalformedForm { reason: "... requires the type registry ...", .. })?;`. `conforms?` uses this to resolve type names.
5. `src/types.rs:3031` (`collect_union_members(union, env) -> Vec<TypeExpr>`) — union resolution.
6. `src/types.rs:67` (`enum TypeExpr`: `Path` / `Parametric{head,args}` / `Fn` / `Var` / `Tuple`) — the grammar to recurse over. The `:TypeExpr` arg arrives parsed as a TypeExpr.
7. `src/runtime.rs:653` (`Value::wat__Record { class_fqdn, .. }`) + `:1219` (`Value::type_name()`) — value→concrete-type extraction for the nominal arm.
8. `src/types.rs` alias resolution (`expand_alias` / `AliasDef`) — for the alias arm.
9. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.2.md` — most recent comparable substrate-mint SCORE; mirror its structural shape.

## Implementation sketch

### Runtime (`src/runtime.rs`)

Register in the eval dispatch (mirror `:wat::core::type`):
```rust
":wat::core::conforms?" => eval_conforms(args, list_span, env, sym),
```

`eval_conforms`: arity 2 (`value`, `type-expr`). Eval arg 0 → the value. Arg 1 is a TYPE-position keyword → resolve to a `TypeExpr` (mirror how the type-expr arg reaches `eval_*`; the `:TypeExpr` is the literal keyword AST — parse it to `TypeExpr` the same way type-position args are handled). Then `conforms(&value, &texpr, types)` where `types = sym.types().ok_or_else(...)?`:

```rust
fn conforms(value: &Value, texpr: &TypeExpr, types: &TypeEnv) -> Result<bool, RuntimeError> {
    match texpr {
        TypeExpr::Path(name) => match types.get(name) {
            Some(TypeDef::Alias(a))  => conforms(value, &a.target, types),       // resolve + recurse
            Some(TypeDef::Union(u))  => Ok(collect_union_members(u, types).iter()
                                              .any(|m| conforms(value, m, types).unwrap_or(false))),
            Some(TypeDef::Struct/Enum/Newtype) => Ok(concrete_type_name(value) == *name),
            None if is_builtin_primitive(name) => Ok(concrete_type_name(value) == *name),
            None => Err(/* unknown type name — see error contract */),
        },
        TypeExpr::Parametric { head, args } => { /* classifier match on `head` + recurse elements */ }
        TypeExpr::Tuple(elems) => { /* Value::Tuple, same arity, each position conforms */ }
        TypeExpr::Fn { .. } | TypeExpr::Var(_) => Err(/* unsupported — see error contract */),
    }
}
```
- `concrete_type_name(value)`: `Value::wat__Record { class_fqdn, .. }` → `class_fqdn` (already colon-free FQDN); else `value.type_name()` (strip any leading colon to match the `Path` string form — verify how `Path(name)` stores it: with or without leading `:`; match accordingly).
- Parametric collection heads: `:wat::core::Vector`, `:wat::core::List`, `:wat::core::HashSet`, `:wat::core::HashMap`. Confirm the value's classifier matches, then recurse element-wise on `args` (Vector<T>: each element conforms `args[0]`; HashMap<K,V>: keys conform `args[0]`, values `args[1]`). Empty collection → `true` vacuously.

### Check (`src/check.rs`)

Inference scheme for `conforms?`: `(:fn(:T, :TypeExpr) -> :wat::core::bool)`. The 2nd arg is type-position (a type keyword), not value-position — mirror how `:wat::core::type`'s arg OR a `-> :T` slot is treated by the checker so it doesn't try to value-infer the type keyword.

### Error contract (pin exactly)

- well-formed type, value doesn't match → `Ok(false)`.
- unknown/unregistered type name, `:Any`, `Fn` type, `Var` → `Err` with a clean diagnostic naming the offending type expression. NOT `false`. (Probe 12 asserts unknown-name → `is_err`.)

## Discipline

- Modify `src/runtime.rs` + `src/check.rs` ONLY.
- NO new `Value` variant. NO holon-rs (STOP-5).
- Do NOT build `is-<Name>?` auto-mint — that's Stone 237.6.
- Do NOT implement Fn-type structural conformance — error "unsupported" per the contract.
- The `:TypeExpr` arg is taken directly as a type keyword (labels-are-ASTs; no String→keyword wrapping).

## STOP triggers (REJECTION — not permission to defer)

1. Compile errors not traced to a probe-named contract.
2. Lib baseline drops below 827.
3. 100 min elapsed (STOP-3); 150 min (STOP-4 hard kill).
4. holon-rs touched (STOP-5).
5. Files outside `src/runtime.rs` + `src/check.rs` touched.
6. Probe doesn't reach 12/12.
7. Any prior arc-237 probe (237.1–237.4) regresses.
8. You find yourself wanting a new `Value` variant or a parallel type registry — STOP; `sym.types()` is the access path (runtime.rs:7530 precedent).

## FM 2-bis evidence

Probe at `tests/probe_arc237_stone5_conforms.rs` (committed `4fef6ce9`) — 12 contracts. Pre-stone: 11/12 fail on `UnknownFunction(:wat::core::conforms?)`; the records/vectors/unions/alias/nested-union-vector all type-check + construct cleanly, so the ONLY gap is the primitive. Post-stone: 12/12 PASS.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.5.md` (NEW). 12-row scorecard verbatim + final `conforms?` signature + the recursive arm table + line counts per file + cascade depth + honest deltas. Mirror Stone 237.2 SCORE structural shape.

## Calibration

New primitive + recursive walker over a 5-variant grammar + runtime TypeEnv access. Heavier than 237.4 (diagnostics), lighter than 237.2 (new Value variant + dispatch mechanism). **Target band: 40–75 min Mode A; 100 STOP-3; 150 STOP-4.** Per `feedback_stone_briefs_cite_prior_score`: mirror Stone 237.2 SCORE shape.
