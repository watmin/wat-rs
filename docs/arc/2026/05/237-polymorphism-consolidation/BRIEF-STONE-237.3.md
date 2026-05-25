# BRIEF — Stone 237.3 — `:guard` + `:ensure` clause-keywords

**Status:** READY TO SPAWN.

## What to do

Layer `:guard` + `:ensure` clause-keywords on Stone 237.2's defclause foundation. Parser additions for both keyword positions; type-check rules (`:guard` returns `:bool`; `:ensure` is 1-arity `:fn` taking declared return type, returns `:bool`); dispatch extension (`:guard` evaluates BEFORE body for clause-selection; `:ensure` evaluates AFTER body for post-condition).

Purely additive — Stone 237.2's clauses without `:guard`/`:ensure` continue dispatching via arity+type only (Stone 237.2's 12 probes must stay GREEN).

TEMPORARY postcondition error variant (rich `:PostconditionFailed` EDN-serialized refines in Stone 237.4).

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.3.md` — sub-DESIGN with all locked decisions
2. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN.md` — arc umbrella
3. `tests/probe_arc237_stone3_guard_ensure.rs` — **LOAD-BEARING** 14 probes; ALL must PASS
4. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.2.md` — defclause foundation ship record + cascade pattern (extension reuses Stone 237.2's `Value::wat__core__clauses` + `eval_call_to_defclause`)
5. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.1.md` — typeunion + bounded-existential unify (relevant for `:guard` referencing typeunion-typed args; resolved member type bound via Stone 237.1's `subst` recording)
6. Stone 237.2's `Clause` struct in `src/runtime.rs` — extend with `Option<WatAST>` for guard + ensure_fn
7. Stone 237.2's `eval_call_to_defclause` in `src/runtime.rs` — extend dispatch loop with guard-eval + ensure-check steps

## Implementation sketch

### Clause shape extension

```wat
;; Full shape (Stone 237.3)
(args :guard expr :ensure :fn body)

;; Existing minimal (Stone 237.2)
(args body)

;; Partial shapes (Stone 237.3)
(args :guard expr body)
(args :ensure :fn body)
```

Keyword order FIXED: `args → :guard? → :ensure? → body`. Per Path C.

### AST extension (`src/runtime.rs`)

Extend Stone 237.2's `Clause`:

```rust
pub struct Clause {
    pub args: Vec<(String, TypeExpr)>,
    pub return_type: TypeExpr,
    pub guard: Option<WatAST>,        // NEW Stone 237.3
    pub ensure_fn: Option<WatAST>,    // NEW Stone 237.3 (a :fn form)
    pub body: WatAST,
}
```

`Option<WatAST>` preserves backward compat — Stone 237.2 clauses without guards/ensures continue to work.

### Parser extension

After parsing `args` Vector literal in each clause:
1. Peek for `:guard` keyword → if present, parse next form as guard expr
2. Peek for `:ensure` keyword → if present, parse next form as `:fn` form
3. Remaining = body

Reject:
- Multiple `:guard` in same clause → `EnsureFnInvalid` or specific variant
- Multiple `:ensure`
- `:ensure` before `:guard` (order violation)
- `:guard` or `:ensure` AFTER body (body must be terminal)

### Type-checker rules (`src/check.rs`)

For each clause during register_defclause:

**`:guard` validation:**
- Build local scope from clause args
- Infer guard expr type
- Unify with `:wat::core::bool`
- Mismatch → `CheckError::GuardExprNotBoolean { defclause_name, clause_index, got_type, span }`

**`:ensure :fn` validation:**
- Verify it's a `:wat::core::fn` form
- Verify arity == 1
- Verify arg type matches clause's declared return type
- Verify return type is `:wat::core::bool`
- Any failure → `CheckError::EnsureFnInvalid { defclause_name, clause_index, reason, span }` where `reason` describes which check failed

### Evaluator extension (`src/runtime.rs`)

Extend Stone 237.2's `eval_call_to_defclause` dispatch loop:

```rust
for clause in clauses {
    // 1. Arity match (Stone 237.2)
    if clause.args.len() != call_args.len() { continue; }

    // 2. Arg type match (Stone 237.2 via unify)
    if !args_unify_with_clause_types(...) { continue; }

    // 3. Bind clause args
    let scope = bind_args(...);

    // 4. NEW: :guard evaluation
    if let Some(guard) = &clause.guard {
        let guard_result = eval_inner(guard, &scope, sym)?;  // errors propagate
        if !is_bool_true(guard_result) { continue; }          // false → skip
    }

    // 5. Body evaluation (Stone 237.2)
    let result = eval_inner(&clause.body, &scope, sym)?;

    // 6. NEW: :ensure check
    if let Some(ensure_fn) = &clause.ensure_fn {
        let ensure_result = apply_fn(ensure_fn, vec![result.clone()], sym)?;
        if !is_bool_true(ensure_result) {
            return Err(RuntimeError::PostconditionFailedRuntime { ... });
        }
    }

    return Ok(result);
}
// All clauses fell through
Err(RuntimeError::NoMatchingClauseRuntime { ... })  // Stone 237.2 variant
```

### NEW error variants

```rust
// src/check.rs
CheckError::GuardExprNotBoolean {
    defclause_name: String,
    clause_index: usize,
    got_type: String,
    span: Span,
}

CheckError::EnsureFnInvalid {
    defclause_name: String,
    clause_index: usize,
    reason: String,        // "must be :wat::core::fn" | "arity must be 1" | "arg type :T must match return type :T" | "return type must be :bool"
    span: Span,
}

// src/runtime.rs (TEMPORARY — Stone 237.4 refines)
RuntimeError::PostconditionFailedRuntime {
    defclause_name: String,
    clause_index: usize,
    returned_value: ValueSnapshot,
    span: Span,
}
```

### Closure-extraction (`src/closure_extract.rs`)

Stone 237.2's defclause pattern arm already handles clause bodies. Extend to also walk optional `:guard` expr + `:ensure :fn` body if present (both may close over outer scope).

## Discipline

- Modify `src/runtime.rs` + `src/check.rs` + (maybe) `src/closure_extract.rs`
- May cascade to `src/edn_shim.rs` + `src/runtime_error_edn.rs` for new error variants (Stone 237.2 pattern)
- DO NOT touch holon-rs (STOP-5)
- DO NOT commit (orchestrator commits)
- DO NOT mint rich `:PostconditionFailed` EDN-serialized variant (Stone 237.4)
- DO NOT add variadic rest (Stone 237.5)
- DO NOT change keyword-order semantics (locked: args → :guard? → :ensure? → body)
- DO NOT allow multiple `:guard` per clause (locked: one; compose with :and)

## STOP triggers (REJECTION — NOT permission to defer)

1. Unexpected compile errors not traced to a probe-named contract
2. Lib baseline drops below 827
3. Clippy concerns: NOT a ceiling concern per user direction 2026-05-25 (arc 109 closure sweeps; Stone 237.3 may add warnings without rejection)
4. 180 min elapsed (STOP-3)
5. 240 min elapsed (STOP-4 hard kill — partial-state-grading)
6. holon-rs touched (STOP-5)
7. Files outside src/runtime.rs + src/check.rs + src/closure_extract.rs + src/edn_shim.rs + src/runtime_error_edn.rs touched
8. Probe doesn't 14/14 PASS
9. Stone 237.1 OR Stone 237.2 regression (would mean substrate damage)
10. Arc 234 / arc 236 regression
11. Keyword-order enforcement broken (probe 13 failing means parser doesn't enforce locked order)
12. Multiple-guard rejection broken (probe 12 failing means parser allows multi-:guard)

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.3.md` (NEW). 13-row scorecard verbatim + final API shape + line counts + cascade depth + honest deltas. Mirror Stone 237.2 SCORE structural shape.

## FM 2-bis evidence

Probe at `tests/probe_arc237_stone3_guard_ensure.rs` (committed at `5c8c8d5a`) — 14 contracts. Pre-stone: 5/14 PASS (accidental — 5 "expected error" probes pass for wrong reasons); 9/14 FAIL (load-bearing: guard dispatch + ensure check + factorial demo + complex demo + full shape). Post-stone: 14/14 PASS.

## Calibration anchor

Stone 237.2 (defclause foundation; new Value variant + eval dispatch + closure-extract cascade) shipped at ~30.5 min in 90-150 band. Stone 237.3 (clause-keyword extensions; no new Value variant; reuses Stone 237.2's eval_call_to_defclause; adds 2 new error variants + 2 dispatch steps) is COMPARABLE complexity. Likely 30-60 min actual per pre-emption-discipline trend.

**Target band: 90-150 min Mode A; 240 STOP.**

Per `feedback_stone_briefs_cite_prior_score`: mirror Stone 237.2 SCORE structural shape for the cascade + honest-deltas sections.
