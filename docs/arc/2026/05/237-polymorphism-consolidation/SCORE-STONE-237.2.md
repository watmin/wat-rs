# SCORE — Stone 237.2 — mint `:wat::core::defclause` substrate primitive

**Date:** 2026-05-25
**Status:** COMPLETE — 12/12 probe PASS. All 13 scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **defclause probe 12/12 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` (at ceiling; no new warnings above baseline) |
| 5 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
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

### New structs (src/runtime.rs)

```rust
#[derive(Debug, Clone)]
pub struct Clause {
    pub args: Vec<(String, crate::types::TypeExpr)>,   // (binding-name, declared-type)
    pub return_type: crate::types::TypeExpr,             // per-clause declared return
    pub body: Arc<WatAST>,
}

#[derive(Debug, Clone)]
pub struct ClauseSet {
    pub name: String,
    pub clauses: Vec<Clause>,
    pub shared_return: Option<crate::types::TypeExpr>,   // Option A top-level -> :T
}
```

### New Value variant (src/runtime.rs)

```rust
pub enum Value {
    // ... existing variants ...
    /// Stone 237.2 — `:wat::core::defclause` multi-arity dispatcher.
    wat__core__clauses(Arc<ClauseSet>),
}
```

Compiled cleanly under `#[wat_value]` proc-macro seal — `Arc<ClauseSet>` is a container variant (not `Arc<Self>`), so no `allow_wrapping` annotation was needed.

### New RuntimeError variant (src/runtime.rs — TEMPORARY)

```rust
RuntimeError::NoMatchingClauseRuntime {
    name: String,
    called_arity: usize,
    called_args: Vec<ValueSnapshot>,
    attempted_clauses: Vec<String>,
    span: Span,
}
```

Stone 237.4 will refine to rich EDN-serialized shape.

### New CheckError variant (src/check.rs)

```rust
CheckError::NoMatchingClauseAtCallSite {
    name: String,
    called_arity: usize,
    called_arg_types: Vec<String>,
    attempted_clauses: Vec<(usize, Vec<String>)>,
    span: Span,
}
```

### New public functions (src/runtime.rs)

| Function | Signature | Purpose |
|----------|-----------|---------|
| `parse_defclause_form` | `(form: &WatAST) -> Result<(String, Arc<ClauseSet>), RuntimeError>` | Top-level parser; detects Option A vs B; validates arg triples + reserved prefix + non-empty |
| `is_defclause_form` | `(form: &WatAST) -> bool` | Quick head-keyword check |

### New private functions (src/runtime.rs)

| Function | Purpose |
|----------|---------|
| `parse_defclause_args` | Parses `[name <- :T ...]` triples; rejects literal-pattern name slots |
| `parse_defclause_clause` | Parses one clause list form `([args] [-> :T] body)` |
| `eval_call_to_defclause` | Evaluates args then dispatches to `eval_call_to_defclause_with_vals` |
| `eval_call_to_defclause_with_vals` | Core: arity match → type match → bind scope → eval body |
| `value_matches_type_by_name` | Runtime type matcher; delegates to `val_type_path` |
| `val_type_path` | Maps Value variant to canonical type-keyword path |

### New private function (src/check.rs)

| Function | Purpose |
|----------|---------|
| `infer_defclause` | Per-clause body type-check; reconstitutes full form + calls `parse_defclause_form` |

### New CheckEnv field (src/check.rs)

```rust
pub defclause_registrations: HashMap<String, Vec<(Vec<TypeExpr>, TypeExpr)>>,
```

Populated by `collect_splice_defs_ctx` when it sees `:wat::core::defclause` at top level. Consumed by `infer_list` call-site dispatch BEFORE the generic `env.get(canonical_k)` scheme path.

### Surface form (both options)

```wat
;; Option A — shared return at top (all clauses must return :T)
(:wat::core::defclause :my::name -> :T
  ([arg <- :T1] body)
  ([arg <- :T1 b <- :T2] body))

;; Option B — per-clause return types (each clause has its own -> :Tn)
(:wat::core::defclause :my::name
  ([arg <- :T1] -> :RetA body)
  ([arg <- :T1 b <- :T2] -> :RetB body))
```

---

## Line count

| File | Post-stone lines | Net added (estimated) |
|------|------------------|-----------------------|
| `src/runtime.rs` | 32,425 | +620 (ClauseSet/Clause, 6 fns, NoMatchingClauseRuntime, render_value arm, register_runtime_defs arm, dispatch arm) |
| `src/check.rs` | 20,747 | +230 (NoMatchingClauseAtCallSite + Display + diagnostic, defclause_registrations field + methods, infer_defclause fn, infer_list arm + call-site dispatch block, collect_splice_defs_ctx arm) |
| `src/closure_extract.rs` | 2,475 | +7 (cascade arm for wat__core__clauses) |
| `src/edn_shim.rs` | 2,406 | +7 (cascade arm for wat__core__clauses) |
| `src/runtime_error_edn.rs` | 404 | +28 (NoMatchingClauseRuntime EDN arm + variant_name arm) |

Total net: ~892 lines. Lands within the BRIEF's 560–910 upper bound.

---

## Cascade depth

**4 rounds.**

1. `src/runtime.rs` — adds `Clause`/`ClauseSet` structs + `Value::wat__core__clauses` variant + `RuntimeError::NoMatchingClauseRuntime` + Display arms + `parse_defclause_form` + `eval_call_to_defclause` + `register_runtime_defs` arm + `dispatch_keyword_head_value` arm + `render_value` arm. Compile reveals 3 non-exhaustive pattern errors.
2. `src/closure_extract.rs` + `src/edn_shim.rs` — mandatory match exhaustiveness fixes for `Value::wat__core__clauses`. Compile reveals 2 more errors in `runtime_error_edn.rs`.
3. `src/runtime_error_edn.rs` — two match sites for `RuntimeError::NoMatchingClauseRuntime`. Compile clean (0 errors).
4. `src/check.rs` — `CheckError::NoMatchingClauseAtCallSite` variant + Display + diagnostic, `CheckEnv::defclause_registrations` field + `register_defclause`/`get_defclause_clauses` methods, `infer_defclause` function, `infer_list` declaration arm + call-site dispatch block, `collect_splice_defs_ctx` registration arm. Probe 12/12 PASS on first attempt.

---

## Honest deltas

### Files outside scope touched

The BRIEF listed `src/runtime.rs + src/check.rs + src/types.rs + src/closure_extract.rs` as the expected scope, noting closure_extract "is the expected match-exhaustiveness cascade." Two additional files were touched by forced exhaustiveness:

- `src/edn_shim.rs` — `value_to_edn_with` match over `Value` required `wat__core__clauses` arm.
- `src/runtime_error_edn.rs` — two match sites over `RuntimeError` required `NoMatchingClauseRuntime` arms.

Both are minimal single-arm additions with zero new logic. This is expected cascade from adding new enum variants; identical to Stone 237.1's cascade pattern.

`src/types.rs` was NOT touched — the BRIEF listed it as an expected modification site but the parser was implemented entirely in `src/runtime.rs` (matching the existing `register_runtime_defs_form` pattern), making a `src/types.rs` change unnecessary.

### Check.rs two-phase pattern

The BRIEF sketched `register_defclause` writing into a CheckEnv field for call-site dispatch. Implementation follows the `def`/`extract_def_binding` two-phase pattern: `collect_splice_defs_ctx` sees the form FIRST (via pre-pass from `check_program`'s sequential loop) and calls `crate::runtime::parse_defclause_form` to populate `defclause_registrations`. The call-site dispatch in `infer_list` then reads from that map. This is correct because `check_program` calls `collect_splice_defs_ctx` before inferring subsequent forms.

### Execution order: runtime_def_values unavailable at check time

`CheckEnv::from_symbols` cannot read `sym.runtime_def_values` for defclause registrations because `register_runtime_defs` runs AFTER `check_program` in the freeze pipeline (step 8 vs step 9 in `startup_from_source`). The two-phase AST parsing approach sidesteps this entirely — `parse_defclause_form` operates on the raw AST, not on the evaluated runtime values.

### Clippy count at ceiling

Post-stone clippy count is 54 (exactly at the ≤54 ceiling). The Stone 237.2 additions contributed 0 new clippy warnings beyond what was already present. The count appears at ceiling because Stone 237.1 introduced new code that triggered warnings not yet cleaned up.

### Typeunion integration (Probe 4)

Probe 4 (typeunion-typed arg) passed without any new check.rs unifier changes. Stone 237.1's `unify_union_with_other` arms fire transparently when the call-site type-check in `infer_list`'s defclause dispatch block calls `unify(arg_ty, expected_ty, ...)`. The integration requires zero glue code — the unifier is the shared substrate.

### defined_values sentinel for defclause names

`register_defclause` inserts `TypeExpr::Var(u64::MAX)` as a sentinel in `env.defined_values` so that keyword references to a defclause name in non-call position (e.g. as a value argument) don't produce `UnknownCallee`. The actual call-site return type comes from `defclause_registrations`, not from `defined_values`. The sentinel is an implementation detail; no user-visible surface.

---

## Working tree on return

```
 M src/check.rs
 M src/closure_extract.rs
 M src/edn_shim.rs
 M src/runtime.rs
 M src/runtime_error_edn.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.2.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
