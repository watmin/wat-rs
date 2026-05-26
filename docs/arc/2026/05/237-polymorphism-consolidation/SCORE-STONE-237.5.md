# SCORE — Stone 237.5 — mint `:wat::core::conforms?` substrate primitive

**Date:** 2026-05-25
**Status:** COMPLETE — 12/12 probe PASS. All 12 scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -3` | 0 errors; 107 warnings (pre-existing ceiling) |
| 2 | **conforms? probe 12/12 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| grep "test result:"` | `14 passed; 0 failed` |
| 5 | Stone 237.2 regression | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| grep "test result:"` | `12 passed; 0 failed` |
| 6 | Stone 237.3 regression | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| grep "test result:"` | `14 passed; 0 failed` |
| 7 | Stone 237.4 regression | `cargo test --release --test probe_arc237_stone4_rich_errors 2>&1 \| grep "test result:"` | `10 passed; 0 failed` |
| 8 | Probe 1: record conforms self | contract | `Ok(bool(true))` |
| 9 | Probe 2: record does NOT conform other | contract | `Ok(bool(false))` |
| 10 | Probe 4: u8 ≠ i64 non-erasure end-to-end | contract | `Ok(bool(true))` / `Ok(bool(false))` |
| 11 | Probe 12: unknown type name → Err | contract | `Err(...)` |
| 12 | holon-rs untouched | STOP-5 | confirmed — zero holon-rs changes |

---

## Final API shape

### New primitive (`:wat::core::conforms?`)

```
(:wat::core::conforms? <value> :TypeExpr) -> :wat::core::bool
```

- Arg 0: any value (type-unchecked at check time; runtime-checked via `conforms_check`)
- Arg 1: type-position keyword (parsed to `TypeExpr` via `parse_type_slot`; not inferred as function)
- Return: `:wat::core::bool`

### New private functions (src/runtime.rs)

| Function | Purpose |
|----------|---------|
| `eval_conforms` | Arity-2 entry point: eval arg0, parse arg1 TypeExpr, acquire TypeEnv, call `conforms_check` |
| `conforms_check` | Recursive walker over `TypeExpr` grammar — the core algorithm |
| `concrete_type_name_matches` | Nominal identity helper: `class_fqdn` (wat__Record) or `type_name()` vs stripped Path name |
| `is_builtin_primitive` | Tests whether a colon-free FQDN is a known substrate primitive type |

### New dispatch arm (src/runtime.rs, near line 5112)

```rust
":wat::core::conforms?" => eval_conforms(args, list_span, env, sym),
```

### New TypeScheme (src/check.rs, `register_builtins`)

```rust
env.register(
    ":wat::core::conforms?".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![t_var(), TypeExpr::Path(":wat::core::keyword".into())],
        ret: bool_ty(),
        rest_param_type: None,
    },
);
```

### New `infer_list` arm (src/check.rs, before `:wat::core::if`)

Special-cased because:
- Arg 0 is type-unchecked by design (conforms? IS the runtime type check); inferred for side-effects only, errors silently discarded
- Arg 1 is type-position keyword; NOT inferred as value expression (avoids `Fn(...)` inference for registered constructors)

---

## Recursive arm table

| TypeExpr variant | Action | Notes |
|-----------------|--------|-------|
| `Path(name)` — TypeEnv has `TypeDef::Alias` | Recurse on `alias.expr` | Transparent alias expansion |
| `Path(name)` — TypeEnv has `TypeDef::Union` | `collect_union_members` → any-member recursion | Exercises Stone 237.1 |
| `Path(name)` — TypeEnv has `TypeDef::Struct/Enum/Newtype` | `concrete_type_name_matches` | Nominal identity |
| `Path(name)` — not in TypeEnv, is built-in primitive | `concrete_type_name_matches` | Substrate primitives (i64/u8/f64/bool/…) |
| `Path(name)` — not in TypeEnv, value is `wat__Record` | `class_fqdn == stripped` | defrecord classes (live in class_fqdn, not TypeEnv) |
| `Path(name)` — not in TypeEnv, other value kind | `Err(unknown type name)` | Error contract: bad input |
| `Parametric { head: "wat::core::Vector", .. }` | Classifier match + recurse each element on `args[0]` | Vacuously true for empty vector |
| `Parametric { head: "wat::core::List", .. }` | Classifier match + recurse each element on `args[0]` | Vacuously true for empty list |
| `Parametric { head: "wat::core::HashSet", .. }` | Classifier match + recurse each element on `args[0]` | Vacuously true for empty set |
| `Parametric { head: "wat::core::HashMap", .. }` | Classifier match + recurse keys on `args[0]`, values on `args[1]` | Vacuously true for empty map |
| `Parametric { head: user-type, .. }` | Classifier match only (nominal head) | Parametric user-types: arc 235 territory |
| `Tuple(elems)` | Same-arity check + per-position recursion | Exact arity required |
| `Fn { .. }` | `Err(fn-type conformance unsupported)` | Affirmative scope cut (runtime limitation) |
| `Var(id)` | `Err(Var is synthetic)` | Defensive; synthetic vars never appear in user-written type exprs |

---

## Key implementation delta: defrecord classes not in TypeEnv

`(:wat::Record::def :my::Circle ...)` is a macro that expands to `defn` forms. It does NOT register a `TypeDef` in the `TypeEnv`. Record class identity lives entirely in `Value::wat__Record.class_fqdn` at runtime.

Consequence: the `Path` arm's `None` branch (not in TypeEnv, not a built-in) must special-case `Value::wat__Record`:

```rust
None => {
    match value {
        Value::wat__Record { class_fqdn, .. } => {
            Ok(class_fqdn.as_str() == stripped)
        }
        _ => Err(format!("unknown type name '{}'...", name))
    }
}
```

This is the ground truth: a `wat__Record` value carries its own type tag (`class_fqdn`); the comparison is direct. For non-record values with an unregistered Path name, `Err` is correct (bad input — probe 12 confirms this contract: `1 conforms? :my::DoesNotExist` → `Err`).

The union-membership arm (`collect_union_members` + recurse) propagates through this correctly: when `:my::Shape` (a typeunion in TypeEnv) is checked, its members `[":my::Circle", ":my::Square"]` are expanded, and for each member Path, the `None`+`wat__Record` branch fires — class_fqdn comparison succeeds for the matching record.

---

## Line count

| File | Pre-stone lines | Post-stone lines | Net added |
|------|-----------------|------------------|-----------|
| `src/runtime.rs` | 32,797 | 33,096 | +299 (dispatch arm comment + arm, `eval_conforms`, `conforms_check`, `concrete_type_name_matches`, `is_builtin_primitive`, section banners) |
| `src/check.rs` | 21,119 | 21,187 | +68 (infer_list arm for `conforms?` with arity check + arg0 discard + arg1 validation + return; TypeScheme registration block) |

Total net: ~367 lines. Within BRIEF's 40–75 min Mode A calibration band (heavier than 237.4, lighter than 237.2).

---

## Cascade depth

**2 rounds.**

1. `src/runtime.rs` — adds dispatch arm + `eval_conforms` + `conforms_check` + `concrete_type_name_matches` + `is_builtin_primitive`. Builds clean (no new Value variant = no exhaustiveness cascade).
2. `src/check.rs` — adds `infer_list` special-case arm + TypeScheme registration. Probe 12/12 PASS. No further cascade.

No new Value variants, RuntimeError variants, or CheckError variants → zero forced cascade files. STOP-5 not triggered (holon-rs untouched).

---

## Honest deltas

### defrecord + TypeEnv gap (not in BRIEF)

The BRIEF's algorithm sketch assumed that `defrecord`-declared types would be found in the TypeEnv under `TypeDef::Struct` / `TypeDef::Enum` / `TypeDef::Newtype`. They are NOT: `(:wat::Record::def ...)` is a macro that expands to `defn` forms, not a type-registration form. The TypeEnv contains only `(:wat::core::struct ...)`, `(:wat::core::enum ...)`, `(:wat::core::newtype ...)`, `(:wat::core::typealias ...)`, and `(:wat::core::typeunion ...)` declarations.

Resolution: the `None` branch of the Path arm checks whether the value is `Value::wat__Record` and, if so, compares `class_fqdn` directly. This is honest — the value carries its own type ground truth. For non-record values with an unknown name, `Err` fires (probe 12 validates this). The union arm fires correctly for typeunion-declared types, and those members (defrecord FQDNs) recurse into the `None`+`wat__Record` path cleanly.

### check.rs infer_list special-case required (not in BRIEF)

The BRIEF sketched that the TypeScheme registration alone (∀T. T × keyword → bool) would be sufficient for the checker. It is NOT, for two reasons:

1. **Type-keyword inference**: Keywords like `:my::Circle` are registered constructors (via the defrecord macro's synthesized `defn`). The standard `infer` path sees `:my::Circle` as a registered function and infers it as `Fn(f64)->wat::Record` — this conflicts with the scheme's `:wat::core::keyword` param. A special-case arm in `infer_list` that skips arg[1] inference entirely is required.

2. **Value-arg type errors**: Probe 11's vector construction `(:wat::core::Vector :my::Shape (:my::Circle 1.0) (:my::Square 2.0))` fails standard type-checking because the checker sees `(:my::Circle 1.0)` as returning `:wat::Record` but the vector requires `:my::Shape`. Since `conforms?` IS the runtime type check (the checker can't know conformance relationships statically for union membership at record-class granularity), arg[0] type errors must be silently discarded. The `infer_list` arm drains arg[0] inference errors into a `_discard` vector.

Both fixes are structurally identical to precedents in the codebase (`infer_apply` uses the same drain-into-discard pattern for args).

---

## Working tree on return

```
 M src/check.rs
 M src/runtime.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.5.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
