# EXPECTATIONS — Stone 237.2

Mode A: 13/13 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **defclause probe 12/12 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 5 | Stone 237.1 regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 6 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | arc 234.1 regression | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 8 | arc 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 9 | arc 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | arc 233.3 regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | arc 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 12 | arc 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 13 | defn unaffected | `cargo test --release --lib -p wat -- runtime::tests 2>&1 \| tail -3` | all passing (existing defn behavior untouched) |

## Prediction

**Target:** 90-150 min Mode A. **Upper:** 180 min (STOP-3). **Hard kill:** 240 min (STOP-4 — partial-state-grading per `feedback_partial_state_grading`).

Surface estimate:
- `src/runtime.rs`: ~250-400 lines net (ClauseSet + Clause structs + Value::wat__core__clauses variant + Eq/Hash/Display/HolonRep impls + eval_defclause_form + eval_call_to_defclause + RuntimeError variant)
- `src/check.rs`: ~200-300 lines net (register_defclause + per-clause body check + call-site dispatch via unify + CheckError variant)
- `src/types.rs`: ~80-150 lines net (parse_defclause + AST extraction + classify_type_decl dispatch arm — IF parser sits there; may sit in src/runtime.rs/macros.rs instead per existing form-dispatch pattern)
- `src/closure_extract.rs`: ~30-60 lines net (defclause pattern arm if walker needs it)
- **Total: ~560-910 lines** across 3-4 files

Confidence: MEDIUM. Value variant + Eq/Hash/Display has high precedent (arc 234.1). Eval dispatch is genuinely new. Closure-extract integration is the unknown.

## Risks

1. **`#[wat_value]` proc-macro seal interaction.** `Value::wat__core__clauses(Arc<ClauseSet>)` is a container variant, not a wrapping variant. Should compile cleanly. If the seal rejects: opt-in via `allow_wrapping` with explicit reason. Sonnet should NOT alter the seal logic itself (arc 233 Stone 233.2.l doctrine).

2. **Closure extraction walker.** arc 170's `src/closure_extract.rs` walks AST to extract closures. Clause bodies may close over outer scope. If the walker doesn't pattern-match `:wat::core::defclause` head, clause bodies won't be properly extracted. Sonnet must add a defclause arm if needed. Trap-door 2 from sub-DESIGN.

3. **Per-clause body type-check.** Each clause body must type-check against THAT CLAUSE'S declared return type. The check-env must bind clause args to local scope correctly before checking body. If the existing defn check-env machinery is reused, this should be transparent.

4. **Call-site dispatch ambiguity.** Two clauses with same arity + overlapping type signatures: first-match-wins per locked decision. Probe 3 verifies. No fancy disambiguation in 237.2.

5. **typeunion integration.** Stone 237.1's `unify_union_with_other` arm should fire transparently when call's arg is typeunion-typed. Probe 4 validates end-to-end. If integration fails, the issue is in call-site type-check threading (not in typeunion machinery itself).

6. **Symbol table lookup.** Names bound to `Value::wat__core__clauses` need to be resolvable as callables at runtime. The existing `Value::wat__core__fn` dispatch mechanism is the template — extend the call-path resolver to handle the new variant.

7. **Test rot LOW.** defclause is purely additive — no existing primitive touched, no existing function modified (parser + check + eval ADD arms; don't replace).

8. **Pre-stone passing probes (1, 8, 12) WILL change behavior.** Probe 1 currently passes because defclause is silently no-op'd. Post-stone it MUST pass because defclause is properly registered + the source type-checks. Probes 8 + 12 currently pass with generic errors; post-stone they must error with the SPECIFIC reasons (NoMatchingClause for 8; binding-contract violation for 12).

## Out-of-scope (REJECTED)

- `:guard` keyword parsing + type-check + dispatch eval (Stone 237.3)
- `:ensure` keyword parsing + type-check (Stone 237.3)
- Rich `:NoMatchingClause` + `:PostconditionFailed` errors per arc 233.3 EDN-shape (Stone 237.4)
- Variadic rest-binder `& rest <- :Vector<:Type>` (Stone 237.5)
- Widest-contagion type-checker rule for kind-typed returns (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)
- AnyBanned error message update (Stone 237.8)
- INSCRIPTION (Stone 237.9)
- Parametric defclause (`defclause :foo<T> ...`) — out of arc 237 entirely
- Open extension (adding clauses to existing defclause name AFTER initial declaration) — defprotocol territory (arc 232.1)
- holon-rs touched (STOP-5)

## SCORE

`SCORE-STONE-237.2.md` (NEW). 13-row scorecard verbatim + final API shape (any naming adjustments — e.g., `ClauseSet` / `Clause` field names) + line count per file + cascade depth (expected 3-5 rounds — runtime adds Value variant, exhaustiveness fixes cascade, check + parser cascade, closure-extract may cascade) + honest deltas.

Per `feedback_stone_briefs_cite_prior_score`: mirror Stone 234.1's SCORE for the Value-variant-mint sections; mirror Stone 237.1's SCORE for the unifier-integration sections.
