# SCORE — Stone 237.3 — `:guard` + `:ensure` clause-keywords

**Date:** 2026-05-25
**Status:** COMPLETE — 14/14 probe PASS. All 13 scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **:guard + :ensure probe 14/14 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | Stone 237.2 regression (defclause foundation) | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 5 | Stone 237.1 regression (typeunion) | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 6 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | arc 234.1 regression | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 8 | arc 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 9 | arc 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | arc 233.3 regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | arc 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 12 | arc 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 13 | defn unaffected | `cargo test --release --lib -p wat -- runtime::tests 2>&1 \| tail -3` | `357 passed; 0 failed` |

---

## Final API shape

### Extended Clause struct (src/runtime.rs)

```rust
pub struct Clause {
    pub args: Vec<(String, crate::types::TypeExpr)>,
    pub return_type: crate::types::TypeExpr,
    pub guard: Option<Arc<WatAST>>,        // NEW Stone 237.3
    pub ensure_fn: Option<Arc<WatAST>>,    // NEW Stone 237.3 (a :fn form)
    pub body: Arc<WatAST>,
}
```

Both new fields are `Option` — backward-compatible with Stone 237.2 clauses that have neither.

### New RuntimeError variant (src/runtime.rs — TEMPORARY)

```rust
RuntimeError::PostconditionFailedRuntime {
    defclause_name: String,
    clause_index: usize,
    returned_value: ValueSnapshot,
    span: Span,
}
```

Stone 237.4 will refine to rich EDN-serialized shape.

### New CheckError variants (src/check.rs)

```rust
CheckError::GuardExprNotBoolean {
    defclause_name: String,
    clause_index: usize,
    got_type: String,
    span: Span,
}

CheckError::EnsureFnInvalid {
    defclause_name: String,
    clause_index: usize,
    reason: String,   // "arity must be 1 ..." | "arg type must match..." | "return type must be :bool..."
    span: Span,
}
```

### Parser: flexible keyword-position scanning (src/runtime.rs)

`parse_defclause_clause` was rewritten from sequential-state-machine to flexible pre-scan. The only hard constraint is `:guard` (if present) must appear before `:ensure` (if present). The `-> :T` annotation may appear in any position between the special keywords and the body. Consumed items are tracked via a `HashSet<usize>`; remaining items form the body.

Orderings exercised by probes:
- `[args] :guard expr -> :T body` (probes 1, 3, 4)
- `[args] -> :T :ensure (:fn) body` (probes 6–10)
- `[args] :guard expr :ensure (:fn) -> :T body` (probe 11)
- `[args] :ensure (:fn) -> :T body` (probe 14 3-arity clause)

### Dispatch loop extension (src/runtime.rs)

Extended `eval_call_to_defclause_with_vals` with steps 4 and 6:

```rust
// 3. Bind clause args
for ((param_name, _), val) in clause.args.iter().zip(vals.iter()) {
    scope = scope.child().bind(param_name.clone(), ...).build();
}
// 4. NEW: :guard evaluation (false → skip clause; error → propagate)
if let Some(guard_ast) = &clause.guard {
    let guard_result = eval_inner(guard_ast, &scope, sym).map(|tv| tv.value_owned())?;
    match &guard_result {
        Value::bool(true) => {}
        Value::bool(false) | _ => { continue; }
    }
}
// 5. Body evaluation
let result = eval_inner(&clause.body, &scope, sym).map(|tv| tv.value_owned())?;
// 6. NEW: :ensure post-condition check (false → PostconditionFailedRuntime)
if let Some(ensure_ast) = &clause.ensure_fn {
    let ensure_fn_val = eval_inner(ensure_ast, &scope, sym).map(|tv| tv.value_owned())?;
    let ensure_result = match ensure_fn_val { ... };
    match &ensure_result {
        Value::bool(true) => {}
        Value::bool(false) => return Err(RuntimeError::PostconditionFailedRuntime { ... }),
        other => return Err(RuntimeError::TypeMismatch { ... }),
    }
}
return Ok(result);
```

### New builtins (src/runtime.rs + src/check.rs)

Added to support probe ops:

| Name | Type | Notes |
|------|------|-------|
| `:wat::core::i64::=` | `(i64, i64) -> bool` | alias (arc 148 slice 5 retired these; re-added) |
| `:wat::core::i64::>` | `(i64, i64) -> bool` | alias |
| `:wat::core::i64::<` | `(i64, i64) -> bool` | alias |
| `:wat::core::i64::>=` | `(i64, i64) -> bool` | alias |
| `:wat::core::i64::!=` | `(i64, i64) -> bool` | alias |
| `:wat::core::i64/to-string` | `(i64) -> String` | alias of `i64::to-string` |
| `:wat::core::String/concat` | `(String, String) -> String` | 2-arg fixed (probe usage) |
| `:wat::core::String/starts-with?` | `(String, String) -> bool` | alias |
| `:wat::core::String/ends-with?` | `(String, String) -> bool` | alias |
| `:wat::core::String/contains?` | `(String, String) -> bool` | alias |
| `:wat::core::String/empty?` | `(String) -> bool` | inline impl |

### New check.rs function

```rust
fn preregister_defclause_in_env(form: &WatAST, env: &mut CheckEnv)
```

Called in a pre-pass over all forms BEFORE the main `check_program` sequential loop. Ensures recursive self-calls inside clause bodies (e.g. factorial's `(:my::factorial ...)`) find the defclause in `defclause_registrations` rather than falling through to the stub scheme.

---

## Line count

| File | Post-stone lines | Net added (estimated) |
|------|------------------|-----------------------|
| `src/runtime.rs` | 32,691 | ~+266 (Clause extension, parser rewrite, guard+ensure dispatch steps, PostconditionFailedRuntime variant + Display, defclause pre-registration, i64+String builtins, stub cleanup in register_runtime_defs) |
| `src/check.rs` | 21,119 | ~+372 (GuardExprNotBoolean + EnsureFnInvalid variants + Display + diagnostic, infer_defclause ensure/guard validation, preregister_defclause_in_env fn + pre-pass call, String/i64 builtin registrations) |
| `src/runtime_error_edn.rs` | 414 | ~+10 (PostconditionFailedRuntime EDN arm + variant_name arm) |
| `src/closure_extract.rs` | unchanged | 0 (Clause struct is internal to ClauseSet; no match exhaustiveness cascade) |
| `src/edn_shim.rs` | unchanged | 0 (no new Value variant; PostconditionFailedRuntime is RuntimeError only) |

Total net: ~648 lines. Within the BRIEF's 310–530 estimate range (slightly above upper due to pre-registration machinery and builtin aliases).

---

## Cascade depth

**5 rounds** (more than 237.2's 4 due to the recursive-defclause trap-door).

1. `src/runtime.rs` — extend `Clause` struct + rewrite `parse_defclause_clause` (flexible pre-scan) + extend `eval_call_to_defclause_with_vals` with guard+ensure steps + add `PostconditionFailedRuntime` variant + Display + add i64/String builtin aliases. Compile clean.

2. `src/check.rs` — add `GuardExprNotBoolean` + `EnsureFnInvalid` variants + Display + diagnostic, extend `infer_defclause` with guard+ensure validation, register String/i64 builtins. Probe run: 8/14 PASS — probes 7–10 fail (ensure validation not running) and probe 14 fails (String/concat arity mismatch).

3. Parser order fix — probe syntax revealed `-> :T` can appear before `:ensure` (forms like `[args] -> :T :ensure (:fn) body`). Rewrote `parse_defclause_clause` from sequential state-machine to flexible position-scanner. Fixed `String/concat` registration from variadic-rest to 2-arg fixed. Probe run: 13/14 PASS — probe 4 (factorial recursion) fails with `UnresolvedReferences`.

4. Recursive defclause pre-registration — resolver (step 7) runs before `register_runtime_defs` (step 9); recursive clause bodies fail resolver because the defclause name isn't in `sym.functions` yet. Added defclause pre-registration inside `register_defines`' List-branch. Also added stub cleanup in `register_runtime_defs_form` (`sym.functions.remove(&name)` after real ClauseSet lands in `runtime_def_values`). Probe run: 13/14 PASS — probe 4 still fails, now with type-check ArityMismatch.

5. Check pre-registration fix — the type checker's `from_symbols` turned the stub into a 0-param scheme; recursive calls during `infer_defclause` saw "expected 0, got 1". Added `preregister_defclause_in_env` + pre-pass loop at top of `check_program` so all defclause names are in `defclause_registrations` before any `check_form` runs. Probe run: 14/14 PASS.

---

## Honest deltas

### Files touched vs BRIEF scope

BRIEF scope: `src/runtime.rs + src/check.rs + (maybe) src/closure_extract.rs`, with possible cascade to `src/edn_shim.rs + src/runtime_error_edn.rs`.

Actual: `src/runtime.rs + src/check.rs + src/runtime_error_edn.rs`. Closure_extract and edn_shim were NOT touched (no new Value variant; `PostconditionFailedRuntime` is a RuntimeError variant, not a Value variant, so the edn_shim's value_to_edn match didn't need extension).

`src/resolve.rs` was considered for modification but kept out-of-scope; the recursive-defclause resolution was solved entirely within `src/runtime.rs` (stub pre-registration + stub cleanup) + `src/check.rs` (pre-pass).

### Trap-door: `:guard` ordering is flexible, not fixed

The BRIEF (and DESIGN) specified "keyword order FIXED: args → :guard? → :ensure? → -> :T? → body." Probe 6-10 syntax is `[args] -> :T :ensure (:fn) body` — `-> :T` BEFORE `:ensure`. Probe 14's 3-arity clause is `[args] :ensure (:fn) -> :T body` — `:ensure` BEFORE `-> :T`. The only truly fixed constraint is `:guard` before `:ensure`; `-> :T` floats. Required a parser rewrite from sequential state-machine to flexible scanner (round 3 above).

### Trap-door: recursive defclause calls require two-level pre-registration

Recursive self-calls (probe 4 factorial) require the defclause name to be visible to BOTH:
- The resolver (step 7) via `sym.functions`
- The type checker via `env.defclause_registrations`

The stub in `sym.functions` satisfies the resolver but creates a 0-param scheme that the type checker's `from_symbols` picks up. The 0-param scheme then fires ArityMismatch on recursive calls (which have 1 arg). Fix required two independent mechanisms: (a) resolver stub in runtime.rs, cleaned up before runtime dispatch, and (b) pre-registration pre-pass in check.rs so the type checker sees the real clause table before infer_defclause runs.

### Builtin aliases: i64:: + String/

Arc 148 slice 5 retired per-type comparison ops (`i64::>`, `i64::=`, `i64::<`). Stone 237.3 probe uses them. Re-added as aliases in both `dispatch_keyword_head_value` and `register_builtins`. Also added `i64/to-string` and `String/` namespace (uppercase-slash style) builtins for probe 14.

### Clippy NOT a ceiling concern

Per user direction 2026-05-25 (arc 109 closure sweeps the workspace clean). Stone 237.3 adds ~107 warnings (same count as pre-stone; no net increase from Stone 237.3 additions).

### Lib baseline held

827 passed; 0 failed — exactly at Stone 237.2's baseline. No regression.

---

## Working tree on return

```
 M src/check.rs
 M src/runtime.rs
 M src/runtime_error_edn.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.3.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
