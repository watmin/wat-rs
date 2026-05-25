# EXPECTATIONS — Stone 237.1

Mode A: 12/12 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **typeunion probe 14/14 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 5 | arc 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | arc 234.1 regression | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 8 | arc 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 9 | arc 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | arc 233.3 regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | arc 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 12 | typealias unaffected | `cargo test --release --lib -p wat -- types::tests --no-fail-fast 2>&1 \| tail -3` | all passing (existing typealias machinery untouched) |

## Prediction

**Target:** 60-120 min Mode A. **Upper:** 180 min (STOP-3). **Hard kill:** 240 min (STOP-4 — partial-state-grading per `feedback_partial_state_grading`).

Surface estimate:
- `src/types.rs`: ~150-200 lines net (UnionDef struct + 4 TypeError variants + parse_typeunion + register_union + cycle detection + member validation + Display impls)
- `src/check.rs`: ~80-120 lines net (typeunion arms in unify + reduce extension + substitution handling)
- **Total: ~230-320 lines** across both files

Confidence: MEDIUM-HIGH. Registration + parser plumbing parallels typealias exactly (high confidence). Unifier extension is genuinely new substrate (medium confidence — bounded-existential typing is the heaviest piece).

## Risks

1. **Unifier extension touches the hot path.** `unify` is called per type-check site. typeunion arm must short-circuit cleanly for non-typeunion types to avoid perf regression. Verify with lib test runtime (should NOT measurably degrade).

2. **`reduce` step ordering.** Must preserve walk-Var → expand-alias → check-union-reference order. Var binding wins over union resolution (synthetic vars must bind to concretes, not unions). Trap-door 1 from sub-DESIGN.

3. **Substitution semantics.** Resolved typeunion member must persist in `subst` so subsequent unify(union, OtherMember) correctly FAILS. Trap-door 2.

4. **Recursive typeunion expansion.** :Baz [:Foo :bool] where :Foo [:i64 :f64] — resolution must walk through :Foo to find :i64. Cycle-check at registration prevents infinite recursion. Probe 14 verifies this end-to-end.

5. **Member type-check at registration time.** Probes 5+6 verify Fn/Var rejection. Sonnet must emit `InvalidUnionMember` with `reason` field populated explaining why (Fn = weird dispatch; Var = synthetic).

6. **Test-rot risk LOW.** typeunion is purely additive — no existing primitive touched, no existing function modified (except `unify` + `reduce` which gain NEW arms, not modified arms).

7. **Probe 13 expected failure mode.** Stone 237.1 must reject `(:my::identity "hello")` where `:my::identity` takes `:my::IorF` (typeunion of i64+f64). The exact error variant doesn't matter (TypeMismatch is fine); what matters is that startup_from_source returns Err.

## Out-of-scope (REJECTED)

- `:wat::core::defclause` primitive (Stone 237.2)
- Per-clause return types (Stone 237.2)
- `:guard` / `:ensure` semantics (Stone 237.3)
- `:NoMatchingClause` / `:PostconditionFailed` errors (Stone 237.4)
- Variadic rest-binder with typeunion-typed Vector (Stone 237.5)
- Widest-contagion type-checker rule (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)
- AnyBanned error message update (Stone 237.8)
- INSCRIPTION (Stone 237.9)
- Parametric typeunions (`typeunion :Result<T,E> ...`) — out of arc 237 entirely
- Reflection of typeunion via `:wat::core::type` polymorphic primitive — future opportunity; not 237.1's scope
- holon-rs touched (STOP-5)

## SCORE

`SCORE-STONE-237.1.md` (NEW). 12-row scorecard verbatim + final API shape (any naming adjustments from sketch — e.g., if `UnionDef`'s field names diverge from the sketch, document) + line count per file + cascade depth (expected 2-3 rounds — types.rs adds first, check.rs cascades) + honest deltas.

Per `feedback_stone_briefs_cite_prior_score`: mirror the structural shape of Stone 236.0's SCORE for the 11-row + API + cascade-depth + honest-deltas sections.
