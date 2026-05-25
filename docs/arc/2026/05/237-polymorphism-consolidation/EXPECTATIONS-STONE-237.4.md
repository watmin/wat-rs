# EXPECTATIONS — Stone 237.4

Mode A: 12/12 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **rich-errors probe 10/10 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone4_rich_errors 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | arc 233.3 EDN regression (CRITICAL — touches runtime_error_edn.rs) | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 5 | Stone 237.3 regression | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 6 | Stone 237.2 regression | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 7 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 8 | `*Runtime` names gone | `grep -c "NoMatchingClauseRuntime\|PostconditionFailedRuntime" src/*.rs` | `0` |
| 9 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 10 | arc 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 11 | arc 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 12 | defn unaffected | `cargo test --release --lib -p wat -- runtime::tests 2>&1 \| tail -3` | all passing |

**Clippy NOT a ceiling concern** per user direction (arc 109 closure sweeps).

## Prediction

**Target:** 45-90 min Mode A. **Upper:** 120 min (STOP-3). **Hard kill:** 180 min (STOP-4).

Surface estimate:
- `src/runtime.rs`: ~120-200 lines net (ClauseAttempt + ClauseFailureReason structs/enum + variant rename+enrich + Display arms + dispatch-loop accumulation + construction-site updates)
- `src/runtime_error_edn.rs`: ~40-80 lines net (rename + enrich 2 EDN arms + ClauseAttempt/ClauseFailureReason serialization + variant_name)
- **Total: ~160-280 lines** across 2 files

Confidence: HIGH. arc 233.3 established the EDN-serialization pattern; this stone follows it. The dispatch-loop accumulation is the only new logic; it ADDS to the existing loop without changing dispatch semantics.

## Risks

1. **Dispatch-loop failure-reason accumulation.** The loop must record WHY each clause skipped (arity / type / guard). Each skip-point needs its specific reason. Probes 5/6/7 verify each fires. Trap-door 1 from sub-DESIGN.

2. **GuardFalse vs guard-error.** A `:guard false` records GuardFalse + continues; a `:guard` that RAISES propagates the error (no attempt recorded). Preserve per Stone 237.3 trap-door 2.

3. **Rename cascade.** `*Runtime` names in: variant def, Display arms, EDN arms, variant_name, construction sites, + any test pattern-matches. Grep all; HARD CUT. Probe 8 verifies 0 survivors in src/.

4. **arc 233.3 EDN probe regression.** This stone touches `runtime_error_edn.rs`. The 233.3 probe (5 contracts) MUST stay green — verify the existing 28-variant arms are untouched. Row 4 is CRITICAL.

5. **`ClauseFailureReason` EDN serialization.** Probe 10 requires all three discriminants (ArityMismatch / ArgTypeMismatch / GuardFalse) appear in the serialized NoMatchingClause EDN. Each sub-enum variant needs an EDN representation.

6. **dual-span extraction for PostconditionFailed.** `body_span` from clause body AST; `ensure_span` from `:ensure :fn` AST. Both captured at construction time.

7. **ensure_expr_snapshot rendering.** The `:ensure :fn` AST rendered to a String at construction (probe 8 marker test). Use the existing AST-to-string rendering (whatever Display/render the codebase uses for WatAST).

8. **Test rot LOW-MEDIUM.** Rename touches multiple sites; behavior unchanged. The 237.2/237.3 probes test is_err (behavior), not variant names, so they should stay green — but verify.

## Out-of-scope (REJECTED)

- Variadic rest-binder (Stone 237.5)
- Widest-contagion type-checker rule (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)
- AnyBanned error message update (Stone 237.8)
- INSCRIPTION (Stone 237.9)
- Check-side `CheckError::NoMatchingClauseAtCallSite` enrichment (leave as-is unless probe demands)
- holon-rs touched (STOP-5)

## SCORE

`SCORE-STONE-237.4.md` (NEW). 12-row scorecard verbatim + final API shape (ClauseAttempt/ClauseFailureReason field names) + line count per file + cascade depth (expected 2-3 rounds) + honest deltas.

Mirror Stone 237.3 SCORE structural shape.
