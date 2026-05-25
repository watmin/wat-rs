# SCORE — Stone 234.3c.fix-narrow-fallthrough

**Status:** COMPLETE. 11/11 PASS.

**Date:** 2026-05-24.

---

## Scorecard

| # | Row | Verification | Result |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe 4/4 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -5` | `4 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 5 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.2c regression | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` (≤ 54) |

All 11/11 PASS.

---

## Implementation

**File changed:** `src/check.rs` only. ~50 lines modified in the `None =>` arm of `env.get(canonical_k)` lookup.

### Change summary

Replaced:
```rust
for arg in args {
    let _ = infer(arg, env, locals, fresh, subst, errors);
}
if args.len() == 1 {
    return Some(fresh.fresh());
}
return None;
```

With (per D2 — collect instead of discard, then discriminate):
```rust
let arg_types: Vec<Option<TypeExpr>> = args
    .iter()
    .map(|arg| infer(arg, env, locals, fresh, subst, errors))
    .collect();
if args.len() == 1 && !k.starts_with(":wat::") {
    let resolved = arg_types[0].as_ref().map(|t| apply_subst(t, subst));
    let acceptable = match &resolved {
        None => true,
        Some(TypeExpr::Var(_)) => true,
        Some(TypeExpr::Path(p)) if p == ":wat::Record" => true,
        Some(TypeExpr::Path(p)) => matches!(
            env.types().get(p.as_str()),
            Some(crate::types::TypeDef::Struct(_))
        ),
        Some(TypeExpr::Parametric { head, .. }) if head == "wat::core::HashMap" => true,
        Some(_) => false,
    };
    if acceptable {
        return Some(fresh.fresh());
    }
    errors.push(CheckError::UnknownCallee {
        callee: k.clone(),
        span: head_span.clone(),
    });
    return None;
}
return None;
```

### Receiver-discrimination predicate used

`env.types().get(p.as_str())` returns `Option<&TypeDef>`; matched against `Some(crate::types::TypeDef::Struct(_))`. `TypeEnv::get` is the existing registry query. No `is_struct` helper existed; the match was inlined per existing patterns at lines 6385, 6405, 6518 in check.rs.

### Substrate-internal guard (!k.starts_with(":wat::"))

The initial narrowing (without the guard) immediately broke probes 2, 3, 4 due to `UnknownCallee { callee: ":wat::core::struct-new" }`. Investigation: `(:wat::core::struct-new :my::T)` (zero-field struct constructor) is a 1-arg call where the receiver is a keyword `:my::T` (type `keyword`), which is concrete non-record → rejected by the narrowing.

Resolution: the DESIGN identifies the narrowing as applying to "user keyword accessors". Substrate-internal primitives (`:wat::core::struct-new`, `:wat::core::struct-field`, etc.) that happen to have 1-arg forms and no registered CheckEnv scheme are NOT user keyword accessors. The guard `!k.starts_with(":wat::")` cleanly separates user-namespace keywords from substrate internals. The guard is conservative (prefers permissive for any `:wat::*` key without a scheme); this matches prior behavior for those keys.

### Cascade depth

1 arm in 1 function in 1 file. No consumers required fixing. Lib baseline stayed at exactly 827. No lib test relied on the over-permissive behavior for user-namespace keywords.

### Consumer-test reliance on over-permissive behavior

None surfaced. The 827 lib tests all pass. The narrowing exclusively targets user-namespace keyword calls with concrete non-accessor receivers (e.g., `(:bogus 42)` where 42 : i64). No existing test was relying on that path passing check.

### Probe 4 status

PASS (4/4 total). Probe 4 uses `(v :wat::Record)` as parameter type — inferred as `TypeExpr::Path(":wat::Record")` — which is explicitly allowed. No follow-up needed.

### Probe 1 mechanism

`(:bogus x)` where `x` is `42` (i64) in a let binding. `infer(x)` returns `locals.get("x")` = `Some(TypeExpr::Path(":wat::core::i64"))`. After `apply_subst`, resolved = `Some(Path(":wat::core::i64"))`. `":bogus"` does not start with `:wat::`, so the guard passes. Match: not `:wat::Record`, not in TypeEnv as Struct, not HashMap → `acceptable = false`. `CheckError::UnknownCallee { callee: ":bogus" }` is pushed. `startup_from_source` returns `Err(StartupError::Check(...))`. Error message contains "unknown callee: :bogus" → `lower.contains("unknown")` → probe 1 PASS.

---

## Working tree on completion

```
 M src/check.rs
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3c.fix-narrow-fallthrough.md
```
