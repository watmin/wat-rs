# EXPECTATIONS — Stone 237.3

Mode A: 13/13 PASS.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **:guard + :ensure probe 14/14 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc237_stone3_guard_ensure 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 3 | Lib baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 237.2 regression (defclause foundation) | `cargo test --release --test probe_arc237_stone2_defclause_substrate 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 5 | Stone 237.1 regression (typeunion) | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 6 | arc 234.0 regression | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | arc 234.1 regression | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 8 | arc 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 9 | arc 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | arc 233.3 regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | arc 236.0 regression | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 12 | arc 234.2a regression | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 13 | defn unaffected | `cargo test --release --lib -p wat -- runtime::tests 2>&1 \| tail -3` | all passing |

**Clippy NOT a ceiling concern** per user direction 2026-05-25 — arc 109 closure sweeps the workspace clean; Stone 237.3 may add warnings without rejection.

## Prediction

**Target:** 90-150 min Mode A. **Upper:** 180 min (STOP-3). **Hard kill:** 240 min (STOP-4 — partial-state-grading).

Surface estimate:
- `src/runtime.rs`: ~150-250 lines net (Clause struct extension + parser additions + dispatch loop guard+ensure steps + RuntimeError variant + Eq/Hash/Display impls)
- `src/check.rs`: ~120-200 lines net (per-clause guard+ensure type-check + 2 CheckError variants)
- `src/closure_extract.rs`: ~20-40 lines net (extend defclause arm for optional guard + ensure_fn)
- `src/edn_shim.rs` + `src/runtime_error_edn.rs`: ~20-40 lines net (cascade arms for new RuntimeError variant)
- **Total: ~310-530 lines** across 3-5 files

Confidence: HIGH. Stone 237.2 established the foundation; Stone 237.3 extends along documented seams (Clause struct, dispatch loop). No new Value variants. No new Eval-dispatch entity kinds.

## Risks

1. **`:guard` scope-binding ordering.** Args must bind before guard evaluates. Trap-door 1 from sub-DESIGN.

2. **`:guard` false vs runtime error distinction.** false → skip clause; error → propagate. The `eval_inner(guard, ...)?` pattern propagates errors; only Bool(false) triggers skip. Trap-door 2.

3. **`:ensure :fn` body extraction.** Closure-extract walker must handle the `:fn` form's body. Same trap-door as Stone 237.2's clause body closure (already handled).

4. **`:ensure :fn` arity validation.** Probe 8 verifies. Type-check must reject 0-arity AND 2+-arity.

5. **`:ensure :fn` arg-type matches declared return.** Probe 9 verifies. Cross-check during register_defclause.

6. **`:ensure :fn` return-type must be :bool.** Probe 10 verifies.

7. **Order enforcement at parse.** Probe 13 verifies. `:ensure` before `:guard` rejects at parse time (not type-check).

8. **Multiple-guard rejection at parse.** Probe 12 verifies. Single `:guard` per clause; multiple → parse error.

9. **Recursive defclause calls.** Factorial demo (probe 4) requires recursion through the new dispatch loop. Stone 237.2 already supports recursive defclause calls; verify still works with guards added.

10. **Same-arity-different-guards dispatch.** Probe 14 has 2 same-arity clauses differing only in guards. First-match-wins per locked decision. Verify second clause fires when first guard false.

11. **Test rot LOW.** Purely additive: clauses without guards/ensures continue Stone 237.2 path; new optional Clause fields default to None.

12. **typeunion + guard interaction.** If a clause's arg is typeunion-typed (`[x <- :Numeric]`) AND has a guard referencing the resolved member's method (e.g., `:wat::core::i64::> x 0`), Stone 237.1's `unify_union_with_other` resolution binds `x` to the matched concrete type for the guard's dispatch. Should work transparently; no special handling in 237.3.

## Out-of-scope (REJECTED)

- Rich `:PostconditionFailed` EDN-serialized variant (Stone 237.4)
- Rich `:NoMatchingClause` EDN refinement (Stone 237.4)
- Variadic rest-binder `& rest <- :Vector<:Type>` (Stone 237.5)
- Widest-contagion type-checker rule for kind-typed returns (Stone 237.5)
- arc 146 Dispatch migration (Stone 237.6)
- Arithmetic special-case retirement (Stone 237.7)
- AnyBanned error message update (Stone 237.8)
- INSCRIPTION (Stone 237.9)
- Multiple `:guard` per clause (locked: compose with :and)
- `:ensure` before `:guard` (locked: order fixed)
- `:guard` or `:ensure` AFTER body (locked: body must be terminal)
- holon-rs touched (STOP-5)

## SCORE

`SCORE-STONE-237.3.md` (NEW). 13-row scorecard verbatim + final API shape (any naming adjustments — e.g., CheckError variant field names) + line count per file + cascade depth (expected 2-4 rounds) + honest deltas.

Mirror Stone 237.2 SCORE structural shape.
