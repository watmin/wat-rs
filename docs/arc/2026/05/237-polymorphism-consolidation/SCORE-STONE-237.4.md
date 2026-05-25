# SCORE — Stone 237.4 — rich `:NoMatchingClause` + `:PostconditionFailed` diagnostics

**Date:** 2026-05-25
**Status:** COMPLETE — 10/10 probe PASS. All 12 scorecard rows green.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **rich-errors probe 10/10 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone4_rich_errors 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 4 | arc 233.3 EDN regression (CRITICAL — touches runtime_error_edn.rs) | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 5 | Stone 237.3 regression | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 6 | Stone 237.2 regression | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 7 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 8 | `*Runtime` names gone | `grep -c "NoMatchingClauseRuntime\|PostconditionFailedRuntime" src/*.rs` | `0` |
| 9 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 10 | arc 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 11 | arc 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 12 | defn unaffected | `cargo test --release --lib -p wat -- runtime::tests 2>&1 \| tail -3` | `357 passed; 0 failed` |

---

## Final API shape

### New types (src/runtime.rs)

```rust
/// Stone 237.4 — per-clause failure reason for defclause dispatch.
pub struct ClauseAttempt {
    pub clause_index: usize,
    pub declared_arity: usize,
    pub declared_arg_types: Vec<String>,
    pub failure_reason: ClauseFailureReason,
}

/// Stone 237.4 — discriminant for why a defclause clause was skipped.
pub enum ClauseFailureReason {
    ArityMismatch { expected: usize, got: usize },
    ArgTypeMismatch { position: usize, expected: String, got: String },
    GuardFalse,
}
```

### Renamed + enriched RuntimeError variants (src/runtime.rs)

```rust
// HARD CUT rename from NoMatchingClauseRuntime;
// attempted_clauses promoted Vec<String> → Vec<ClauseAttempt>
NoMatchingClause {
    name: String,
    called_arity: usize,
    called_args: Vec<ValueSnapshot>,
    attempted_clauses: Vec<ClauseAttempt>,
    span: Span,
}

// HARD CUT rename from PostconditionFailedRuntime;
// + ensure_expr_snapshot + dual spans
PostconditionFailed {
    defclause_name: String,
    clause_index: usize,
    ensure_expr_snapshot: String,
    returned_value: ValueSnapshot,
    body_span: Span,
    ensure_span: Span,
}
```

### Clean EDN tags (src/runtime_error_edn.rs)

- `#wat.kernel/NoMatchingClause` (was `#wat.kernel/NoMatchingClauseRuntime`)
- `#wat.kernel/PostconditionFailed` (was `#wat.kernel/PostconditionFailedRuntime`)
- `#wat.kernel/ClauseAttempt` — new sub-tagged map per skipped clause
- `#wat.kernel/ArityMismatch`, `#wat.kernel/ArgTypeMismatch`, `#wat.kernel/GuardFalse` — per `ClauseFailureReason` variant

### Dispatch loop (src/runtime.rs `eval_call_to_defclause_with_vals`)

Replaced `Vec<String>` accumulation with `Vec<ClauseAttempt>`. Each skip path now records its specific reason:
- Arity skip → `ClauseFailureReason::ArityMismatch { expected, got }`
- Type skip → `ClauseFailureReason::ArgTypeMismatch { position, expected, got }` (first failing position)
- Guard-false skip → `ClauseFailureReason::GuardFalse`

`PostconditionFailed` construction site captures `ensure_expr_snapshot = format!("{:?}", ensure_ast)`, `body_span = clause.body.span().clone()`, `ensure_span = ensure_ast.span().clone()`.

---

## Line count

| File | Post-stone lines | Net added |
|------|-----------------|-----------|
| `src/runtime.rs` | 32,797 | ~+106 (ClauseAttempt + ClauseFailureReason structs; variant rename+enrich; Display arms; dispatch-loop accumulation + PostconditionFailed construction-site upgrade) |
| `src/runtime_error_edn.rs` | 460 | ~+46 (2 EDN arms renamed+enriched; clause_attempt_to_edn + clause_failure_reason_to_edn helpers; import update; variant_name arms) |
| `src/check.rs` | unchanged | 0 (check-side left as-is per DESIGN decision) |

Total net: ~152 lines. Within the BRIEF's 160–280 line estimate range.

---

## Cascade depth

**1 round.** Single-pass implementation.

1. `src/runtime.rs` — mint `ClauseAttempt` + `ClauseFailureReason`; rename + enrich 2 RuntimeError variants; update Display arms; rewrite dispatch loop to accumulate structured attempts; upgrade PostconditionFailed construction site.
2. `src/runtime_error_edn.rs` — update import; rename + enrich 2 EDN arms; add `clause_attempt_to_edn` + `clause_failure_reason_to_edn` helpers; update `variant_name` arms.

Compile clean on first attempt. Probe run: 10/10 PASS. No additional rounds needed.

---

## Honest deltas

### Files touched vs BRIEF scope

BRIEF scope: `src/runtime.rs + src/runtime_error_edn.rs` (+ optionally `src/check.rs`).

Actual: `src/runtime.rs + src/runtime_error_edn.rs`. `src/check.rs` was NOT touched — probe contracts were satisfied without check-side enrichment, per DESIGN decision.

### One surviving *Runtime name — doc comment

`grep -c` reported 1 survivor in `runtime.rs` — a doc comment in the `Clause` struct's `ensure_fn` field: `/// PostconditionFailedRuntime. Runtime error propagates.` Updated to `PostconditionFailed`. The hard-cut rule applies to all occurrences including doc comments; zero survivors confirmed post-fix.

### Dispatch-loop: type-mismatch detection strategy

The BRIEF's `first_type_mismatch(...)` sketch was implemented via `enumerate().find_map(...)` over `clause.args.zip(vals)`. This finds the FIRST failing position, records it as `ArgTypeMismatch { position, expected, got }`. Subsequent failing positions are not recorded (first-failing-position is the diagnostic convention per arc 233 teaching-values doctrine).

### ensure_expr_snapshot rendering

Used `format!("{:?}", ensure_ast)` — WatAST implements `Debug` but not `Display`. The Debug rendering is verbose but captures the full structure. Probe 8 verifies the snapshot text surfaces in EDN; probe contracts are satisfied by the `ENSURE_MARKER_TEXT` sentinel.

### val_type_path for Struct/Enum fallback

`val_type_path` returns `"<struct>"` / `"<enum>"` for dynamic-name types (Struct, Enum). These will produce `ArgTypeMismatch { got: "<struct>", ... }` which is less informative than the actual class name. This is a pre-existing limitation of `val_type_path` (not introduced by Stone 237.4); a future stone can refine the Struct/Enum path to carry the dynamic type name. Documented here as a known gap.

### Clippy NOT a ceiling concern

Per user direction (arc 109 closure sweeps). Warning count unchanged from Stone 237.3 (107 lib warnings, all pre-existing).

### Lib baseline held

827 passed; 0 failed — identical to Stone 237.3 baseline. No regression.

---

## Working tree on return

```
 M src/runtime.rs
 M src/runtime_error_edn.rs
?? docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.4.md
```

holon-rs untouched. STOP-5 not triggered. DO NOT commit (orchestrator commits).
