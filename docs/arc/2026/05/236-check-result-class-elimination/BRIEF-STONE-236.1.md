# BRIEF — Stone 236.1 — primary `fn infer()` signature flip

**Status:** READY TO SPAWN.

**Predecessor:** Stone 236.0 (CheckResult<T> newtype foundation).

## What to do

Flip the primary `fn infer()` at `src/check.rs` line 4863 from `Option<TypeExpr>` + `errors: &mut Vec<CheckError>` parameter to `CheckResult<TypeExpr>` return.

Cascade: ~126 internal call sites adapt via the `.drain_errors_into(errors)` bridge helper from Stone 236.0.

NO sibling infer_* function signatures change. They stay legacy + bridge at the primary-infer-call site. Sibling-flip is 236.2.

ONE file: `src/check.rs`.

## Read in order

1. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.1.md` — 10 locked decisions + 8 trap-doors
2. `docs/arc/2026/05/236-check-result-class-elimination/EXPECTATIONS-STONE-236.1.md` — scorecard
3. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — arc umbrella + failure-mode rationale
4. `src/check.rs` line 4863 — primary fn infer() (target)
5. `src/check.rs` line ~996 — CheckResult<T> + drain_errors_into helper (your bridge tool)

## Implementation pattern

### Primary infer body translation

```rust
// BEFORE:
fn infer(ast, env, locals, fresh, subst, errors: &mut Vec<CheckError>) -> Option<TypeExpr> {
    match ast {
        WatAST::IntLit(_, _) => Some(TypeExpr::Path(":wat::core::i64".into())),
        // ... other arms ...
        _ => {
            errors.push(CheckError::SomeVariant { ... });
            None
        }
    }
}

// AFTER:
fn infer(ast, env, locals, fresh, subst) -> CheckResult<TypeExpr> {
    let mut local_errors: Vec<CheckError> = Vec::new();
    match ast {
        WatAST::IntLit(_, _) => CheckResult::ok(TypeExpr::Path(":wat::core::i64".into())),
        // ... other arms ...
        _ => {
            local_errors.push(CheckError::SomeVariant { ... });
            CheckResult::errs(local_errors)
        }
    }
}
```

### Recursive calls inside primary infer

```rust
// BEFORE:
let sub_ty = infer(&sub_ast, env, locals, fresh, subst, errors)?;

// AFTER:
let sub_ty = infer(&sub_ast, env, locals, fresh, subst).drain_errors_into(&mut local_errors)?;
```

### Sibling calls inside primary infer

Siblings (`infer_list`, `infer_let`, etc.) STILL take `&mut Vec<CheckError>`. Primary passes its local_errors:

```rust
let result = infer_list(args, env, locals, fresh, subst, &mut local_errors);
```

### EXTERNAL call sites (the cascade — ~126 sites)

Every site that calls primary `infer(arg, env, locals, fresh, subst, errors)` becomes:

```rust
// BEFORE:
let ty = infer(arg, env, locals, fresh, subst, errors)?;
let _ = infer(arg, env, locals, fresh, subst, errors);

// AFTER:
let ty = infer(arg, env, locals, fresh, subst).drain_errors_into(errors)?;
let _ = infer(arg, env, locals, fresh, subst).drain_errors_into(errors);
```

The `errors: &mut Vec<CheckError>` is in scope at these sites (parent function's parameter). Mechanical sed-like translation; substrate-as-teacher cascade surfaces each site.

## THE HARVEST (D3 in sub-DESIGN)

Every `return None` in the primary infer body MUST be classified into one of three:

1. **Silent ON PURPOSE** (rare) → `CheckResult::ok(fresh.fresh())` (polymorphic placeholder); document inline why
2. **Error path missing diagnostic** → add appropriate CheckError variant push, then `CheckResult::errs(local_errors)`
3. **Error path with diagnostic already** → straight `CheckResult::errs(local_errors)` conversion

Add an inline comment naming the classification:
```rust
// HARVEST (236.1): no diagnostic on this path; specific error added.
local_errors.push(CheckError::...);
return CheckResult::errs(local_errors);
```

SCORE captures the count of each classification (1/2/3). This is the failure-class harvest data — the answer to "how many silent failures lived in primary infer."

## Discipline

- `src/check.rs` ONLY (STOP-5)
- DO NOT touch: any other file, any sibling infer_* signature (that's 236.2), wat sources, probes
- DO NOT commit (orchestrator commits)
- DO NOT mint transitional dual-channel shim like `infer_v2()` (D8 HARD CUT)
- DO NOT preserve the `errors: &mut Vec<CheckError>` parameter on primary infer (D1 — full flip)
- DO NOT touch holon-rs (STOP-4)

## Lib baseline handling

If lib baseline drops by 1-2 tests AND the drops trace to HARVEST classifications that surface previously-silent failures (D3 case 2 — adding diagnostic where there was silence): **expected change**; surface in SCORE; orchestrator reviews per case. NOT auto-STOP-2.

If lib baseline drops > 5 tests OR drops trace to migration errors: STOP-2.

## STOP triggers (REJECTION)

1. unexpected compile errors not tracing to signature flip / cascade
2. lib baseline drops > 5 (or 1-2 if NOT from harvest classification)
3. 120 min elapsed
4. holon-rs touched
5. Rust changes outside src/check.rs
6. sibling infer_* signature flipped (STOP-6; that's 236.2)
7. arc 234 regression
8. clippy > 54
9. transitional dual-channel shim minted

## SCORE doc

`docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.1.md` (NEW).

Capture:
- 11-row scorecard verbatim outputs
- HARVEST classification counts (D3): how many sites fell into 1/2/3
- New CheckError variants added (if any; should be few)
- Cascade depth: compile rounds + iteration cycles
- Per-classification narrative: which sites were silent-on-purpose vs silent-by-defect
- Honest deltas
- Any lib-test changes (expected behavior shifts from harvest)
