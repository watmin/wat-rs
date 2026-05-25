# BRIEF — Stone 236.2 — sibling `infer_*` fn signature flip

**Status:** READY TO SPAWN.

**Predecessor:** Stone 236.1 (primary `fn infer()` flipped; ~156 call sites cascaded; HARVEST 2/0/1; SHIPPED `f06549ad`).

## What to do

Flip ALL 47 sibling `infer_*` functions in `src/check.rs` from `Option<TypeExpr>` + `errors: &mut Vec<CheckError>` parameter to `CheckResult<TypeExpr>` return.

Cascade: ~111 internal sibling-call sites adapt via the `.drain_errors_into(...)` bridge helper from Stone 236.0. Primary `fn infer()`'s legacy `&mut local_errors` calls to siblings flip to bridge-form simultaneously.

NO new primitives. NO new probe files. NO transitional dual-channel shim. NO touching files outside `src/check.rs`.

ONE file: `src/check.rs`.

## Read in order

1. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.2.md` — 10 locked decisions + 12 trap-doors
2. `docs/arc/2026/05/236-check-result-class-elimination/EXPECTATIONS-STONE-236.2.md` — 12-row scorecard
3. `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.1.md` — **predecessor pattern; mirror exactly** (cascade record, HARVEST methodology, iteration pattern across 7 logical passes)
4. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.1.md` — primary fn flip context (siblings now apply same shape)
5. `docs/arc/2026/05/236-check-result-class-elimination/DESIGN.md` — arc umbrella + failure-mode rationale
6. `src/check.rs` line 1040-1206 — **CheckResult migration-pattern docstring** (uses `infer_legacy` / `infer_new` / `infer_something` / `infer_something_inner` as worked examples; copy this pattern verbatim)
7. `src/check.rs` line ~996 — CheckResult<T> + drain_errors_into helper (your bridge tool from 236.0)
8. `src/check.rs` line 5056-13164 — the 47 sibling `infer_*` defs (the target surface)

## The 47 siblings (disk truth from `^fn infer_` grep)

```
5056   infer_some_constructor             8506   infer_config_set_bool
5103   infer_ok_constructor               8943   infer_try
5151   infer_err_constructor              9053   infer_option_try
5199   infer_list                         9157   infer_option_expect
6472   infer_match                        9258   infer_result_expect
7751   infer_if                           9369   infer_kernel_readln
7879   infer_do                           9448   infer_apply
7917   infer_cond                         9541   infer_program_env_get
8090   infer_let                          9644   infer_program_env_expect_get
8248   infer_def                          9744   infer_program_env_get_default
8369   infer_def_restricted               9867   infer_program_env_dig
                                          9978   infer_program_env_expect_dig
10082  infer_program_env_dig_default     10578   infer_spawn
10695  infer_positional_accessor         10765   infer_drop
10817  infer_make_queue                  11173   infer_hashset_constructor
11264  infer_comparison                  11333   infer_arithmetic
11437  infer_record_of                   11548   infer_polymorphic_time_arith
11653  infer_form_matches                11929   infer_polymorphic_holon_pair_to_f64
11992  infer_holon_bind                  12054   infer_holon_bundle
12136  infer_polymorphic_holon_pair_to_bool  12194  infer_polymorphic_holon_pair_to_path
12249  infer_polymorphic_holon_to_i64    12335   infer_hashmap_constructor
12470  infer_tuple_constructor           12509   infer_string_concat
12549  infer_dispatch_call               12827   infer_list_constructor
12919  infer_fn                          13164   infer_boolean_shortcircuit
```

47 fns total. All flip uniformly. Drop the trailing `errors: &mut Vec<CheckError>` parameter; return `CheckResult<TypeExpr>`.

## Implementation pattern

### Sibling body translation

```rust
// BEFORE (uniform shape across all 47):
fn infer_<verb>(
    /* fn-specific params */,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
    errors: &mut Vec<CheckError>,
) -> Option<TypeExpr> {
    match ast {
        SomeArm => Some(TypeExpr::Path(...)),
        ErrorArm => {
            errors.push(CheckError::SomeVariant { ... });
            None
        }
        SilentArm => None,           // ← HARVEST point
    }
}

// AFTER:
fn infer_<verb>(
    /* fn-specific params */,
    env: &CheckEnv,
    locals: &HashMap<String, TypeExpr>,
    fresh: &mut InferCtx,
    subst: &mut Subst,
) -> CheckResult<TypeExpr> {
    let mut local_errors: Vec<CheckError> = Vec::new();
    match ast {
        SomeArm => CheckResult::ok(TypeExpr::Path(...)),
        ErrorArm => {
            local_errors.push(CheckError::SomeVariant { ... });
            CheckResult::errs(local_errors)
        }
        SilentArm => {
            // HARVEST (236.2): classify per D3 — silent-on-purpose | missing-diagnostic | had-diagnostic
            ...
        }
    }
}
```

### Internal sibling-call bridge (sibling calling another sibling — concentrated in `infer_list`)

```rust
// BEFORE (inside e.g. infer_list body):
let ty = infer_let(args, env, locals, fresh, subst, errors)?;
let _ = infer_arithmetic(args, env, locals, fresh, subst, errors);

// AFTER:
let ty = infer_let(args, env, locals, fresh, subst).drain_errors_into(&mut local_errors)?;
let _ = infer_arithmetic(args, env, locals, fresh, subst).drain_errors_into(&mut local_errors);
```

### Primary fn infer's call-site flip (the post-236.1 bridge update)

Primary fn infer (already CheckResult-returning) calls siblings via legacy `&mut local_errors` (Stone 236.1 D4 left this as the bridging shape). Now siblings flipped:

```rust
// CURRENT (236.1 state inside primary infer body):
let result = infer_list(args, env, locals, fresh, subst, &mut local_errors);

// AFTER (236.2):
let result = infer_list(args, env, locals, fresh, subst).drain_errors_into(&mut local_errors);
```

Stone 236.1 left inline comments at these sites naming the 236.2 dependency (e.g., *"sibling-delegation pending 236.2"*). Remove those comments as part of the flip — they're no longer load-bearing.

## THE HARVEST (D3 in sub-DESIGN)

Every `return None` in every sibling body MUST be classified into one of three (mirror 236.1's primary fn methodology):

1. **Silent ON PURPOSE** (e.g., Symbol-arm "type unknown") → `CheckResult::ok(fresh.fresh())` (polymorphic placeholder); document inline why
2. **Error path missing diagnostic** → add appropriate CheckError variant push, then `CheckResult::errs(local_errors)`
3. **Error path with diagnostic already** → straight `CheckResult::errs(local_errors)` conversion

Add an inline comment naming the classification:
```rust
// HARVEST (236.2): no diagnostic on this path; specific error added.
local_errors.push(CheckError::...);
return CheckResult::errs(local_errors);

// HARVEST (236.2): silent-by-intent — polymorphic placeholder.
return CheckResult::ok(fresh.fresh());

// HARVEST (236.2): existing diagnostic; straight conversion.
return CheckResult::errs(local_errors);
```

**Pre-warning:** Stone 236.1 SCORE explicitly named *"the silent failures live in the sibling functions — 236.2's territory."* Expect Classification 2 count > 0 (sibling fns likely have silent failures finally getting diagnostics minted). 0-5 new CheckError variants may be needed.

SCORE captures aggregate counts per classification (Classification 1 / 2 / 3 summed across all 47 siblings). This is the failure-class harvest data — answers "how many silent failures lived in the sibling fns total."

## Discipline

- `src/check.rs` ONLY (STOP-5)
- DO NOT touch: any other file, the primary `fn infer()` signature (already shipped by 236.1; STOP-6), wat sources, probes
- DO NOT commit (orchestrator commits)
- DO NOT mint transitional dual-channel shim like `infer_<verb>_v2()` (D8 HARD CUT)
- DO NOT preserve the `errors: &mut Vec<CheckError>` parameter on ANY of the 47 siblings (D1 — full flip across all)
- DO NOT touch holon-rs (STOP-4)
- DO NOT add new probe files (D6; existing scorecard regression-guards are sufficient)

## Lib baseline handling

If lib baseline drops by 1-5 tests AND the drops trace to HARVEST Classification 2 (adding diagnostic where there was silence): **expected change**; surface in SCORE; orchestrator reviews per case. NOT auto-STOP-2.

If lib baseline drops > 5 tests OR drops trace to migration errors: STOP-2.

Threshold is wider than 236.1's (1-2) because 236.1 SCORE foreshadowed higher harvest yield in siblings.

## STOP triggers (REJECTION)

1. Unexpected compile errors not tracing to signature flip / cascade
2. Lib baseline drops > 5 (or > 5 if NOT from harvest classification)
3. **180 min elapsed** (Mode A target 90 min; STOP-3 is 2× upper-bound)
4. holon-rs touched
5. Rust changes outside src/check.rs
6. Primary `fn infer()` signature touched (236.1 already shipped; out of scope)
7. arc 234 regression
8. clippy > 54
9. Transitional dual-channel shim minted

## SCORE doc

`docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.2.md` (NEW).

Capture:
- 12-row scorecard verbatim outputs
- HARVEST aggregate counts (D3): Classification 1 / 2 / 3 summed across all 47 siblings
- Per-fn HARVEST table (which siblings contributed which classification counts)
- New CheckError variants added (if any)
- Cascade depth: compile rounds + iteration cycles
- Per-classification narrative: which sites were silent-on-purpose vs silent-by-defect-now-named
- Honest deltas
- Any lib-test changes (expected behavior shifts from harvest)
- Rank-up evidence vs Stone 236.1 (was the predecessor SCORE doc you mirrored useful?)
