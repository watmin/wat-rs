# BRIEF — Stone 237.4 — rich `:NoMatchingClause` + `:PostconditionFailed` diagnostics

**Status:** READY TO SPAWN.

## What to do

Promote the TEMPORARY error variants from Stones 237.2 + 237.3 to RICH diagnostics per arc 233.3 EDN-shape. Mint `ClauseAttempt` struct + `ClauseFailureReason` enum. HARD-CUT rename `NoMatchingClauseRuntime` → `NoMatchingClause` (structured attempt list) + `PostconditionFailedRuntime` → `PostconditionFailed` (ensure-expr snapshot + dual spans). Clean EDN tags. Make the dispatch loop ACCUMULATE per-clause failure reasons.

Diagnostic-richness work — well-trodden per arc 233's 28-variant precedent. NOT new mechanism.

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.4.md` — sub-DESIGN with current-state diagnosis + locked decisions + trap-door audit
2. `tests/probe_arc237_stone4_rich_errors.rs` — **LOAD-BEARING** 10 probes; ALL must PASS
3. `tests/probe_stone_233_3_runtime_error_edn.rs` — the construct-and-inspect probe technique + EDN-tag convention to mirror
4. `src/runtime.rs:2216` (`NoMatchingClauseRuntime`) + `:2231` (`PostconditionFailedRuntime`) — variants to rename + enrich
5. `src/runtime.rs:7172` + `:7197` — construction sites in the dispatch loop (these build the errors)
6. `src/runtime_error_edn.rs:251` + `:267` — EDN arms to refine + clean tags
7. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.3.md` — most recent ship; runtime_error_edn.rs touched there

## Implementation sketch

### New types (`src/runtime.rs`)

```rust
pub struct ClauseAttempt {
    pub clause_index: usize,
    pub declared_arity: usize,
    pub declared_arg_types: Vec<String>,
    pub failure_reason: ClauseFailureReason,
}

pub enum ClauseFailureReason {
    ArityMismatch { expected: usize, got: usize },
    ArgTypeMismatch { position: usize, expected: String, got: String },
    GuardFalse,
}
```

### Variant refinements (`src/runtime.rs`)

```rust
// RENAMED from NoMatchingClauseRuntime; attempted_clauses now structured
NoMatchingClause {
    name: String,
    called_arity: usize,
    called_args: Vec<ValueSnapshot>,
    attempted_clauses: Vec<ClauseAttempt>,   // was Vec<String>
    span: Span,
}

// RENAMED from PostconditionFailedRuntime; + ensure snapshot + dual spans
PostconditionFailed {
    defclause_name: String,
    clause_index: usize,
    ensure_expr_snapshot: String,            // NEW
    returned_value: ValueSnapshot,
    body_span: Span,                         // NEW
    ensure_span: Span,                       // NEW
}
```

### Dispatch-loop change (load-bearing — `src/runtime.rs` eval_call_to_defclause)

The Stone 237.2/237.3 loop `continue`s on each non-matching clause. NOW it must accumulate a `ClauseAttempt`:

```rust
let mut attempts: Vec<ClauseAttempt> = Vec::new();
for (idx, clause) in clauses.iter().enumerate() {
    // arity
    if clause.args.len() != call_args.len() {
        attempts.push(ClauseAttempt {
            clause_index: idx,
            declared_arity: clause.args.len(),
            declared_arg_types: clause.args.iter().map(|(_, t)| format_type(t)).collect(),
            failure_reason: ClauseFailureReason::ArityMismatch {
                expected: clause.args.len(), got: call_args.len(),
            },
        });
        continue;
    }
    // arg type — record ArgTypeMismatch at the first failing position
    if let Some((pos, expected, got)) = first_type_mismatch(...) {
        attempts.push(ClauseAttempt { ..., failure_reason: ArgTypeMismatch { position: pos, expected, got } });
        continue;
    }
    // bind args
    let scope = bind_args(...);
    // guard
    if let Some(guard) = &clause.guard {
        let guard_result = eval_inner(guard, &scope, sym)?;  // errors propagate
        if !is_bool_true(guard_result) {
            attempts.push(ClauseAttempt { ..., failure_reason: GuardFalse });
            continue;
        }
    }
    // body + ensure (Stone 237.3) ...
    return Ok(result);
}
// all skipped → rich error
Err(RuntimeError::NoMatchingClause {
    name, called_arity, called_args: snapshots, attempted_clauses: attempts, span,
})
```

The `PostconditionFailed` construction site (`runtime.rs:7172` area) gains `ensure_expr_snapshot` (render the `:ensure :fn` AST to a string) + `body_span` (from clause body AST) + `ensure_span` (from `:ensure :fn` AST).

### EDN serialization (`src/runtime_error_edn.rs`)

Rename the 2 arms; clean tags `#wat.kernel/NoMatchingClause` + `#wat.kernel/PostconditionFailed`; serialize `ClauseAttempt` + `ClauseFailureReason` (each failure-reason variant gets an EDN representation — `ArityMismatch` / `ArgTypeMismatch` / `GuardFalse` discriminants must appear in the serialized form per probe 10). Update `variant_name` arms.

### Display arms (`src/runtime.rs`)

Update the human-facing Display for both renamed variants — render the structured attempt list as a teaching message (per docs/SUBSTRATE-AS-TEACHER.md): "no clause of :my::process matched (3 args); clause 0 skipped (arity 2 ≠ 3); clause 1 skipped (arg 0: expected :i64, got :String); clause 2 skipped (guard false)".

## Discipline

- Modify `src/runtime.rs` + `src/runtime_error_edn.rs`
- May touch `src/check.rs` ONLY if probe demands check-side richness (sub-DESIGN decision: leave `CheckError::NoMatchingClauseAtCallSite` as-is)
- DO NOT touch holon-rs (STOP-5)
- DO NOT commit
- HARD CUT the `*Runtime` names — no aliases (per arc 234.6 discipline)
- DO NOT add variadic rest (Stone 237.5)
- DO NOT change dispatch SEMANTICS (only ADD failure-reason accumulation; first-match-wins unchanged)

## STOP triggers (REJECTION — NOT permission to defer)

1. Unexpected compile errors not traced to a probe-named contract
2. Lib baseline drops below 827
3. Clippy: NOT a ceiling concern per user direction (arc 109 closure sweeps)
4. 120 min elapsed (STOP-3)
5. 180 min elapsed (STOP-4 hard kill)
6. holon-rs touched (STOP-5)
7. Files outside src/runtime.rs + src/runtime_error_edn.rs (+ optionally src/check.rs) touched
8. Probe doesn't 10/10 PASS
9. Stone 237.1 / 237.2 / 237.3 regression
10. arc 233.3 EDN probe regression (this stone touches runtime_error_edn.rs — that probe MUST stay green)
11. `*Runtime` names survive anywhere (grep src/ must return 0 for `NoMatchingClauseRuntime` + `PostconditionFailedRuntime`)

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-237.4.md` (NEW). Scorecard verbatim + final API shape + line counts + cascade depth + honest deltas. Mirror Stone 237.3 SCORE shape.

## FM 2-bis evidence

Probe at `tests/probe_arc237_stone4_rich_errors.rs` (committed at `cfde70a9`) — 10 contracts. Pre-stone: file fails to compile (ClauseAttempt + ClauseFailureReason + renamed variants don't exist). Post-stone: 10/10 PASS.

## Calibration anchor

Stone 233.3 (Errors-as-EDN, 28 variants) is the closest precedent — pure diagnostic-richness. Stone 237.4 mints 1 struct + 1 enum + renames/enriches 2 variants + accumulates failure reasons in the dispatch loop. LIGHTER than Stones 237.2/237.3 (no new Value variant; no new dispatch mechanism).

**Target band: 45-90 min Mode A; 180 STOP.** Likely 30-50 min actual.

Per `feedback_stone_briefs_cite_prior_score`: mirror Stone 237.3 SCORE structural shape.
