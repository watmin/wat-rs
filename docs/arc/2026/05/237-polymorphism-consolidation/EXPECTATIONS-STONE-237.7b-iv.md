# EXPECTATIONS — Stone 237.7b-iv — `get` ∀T intrinsic with custom arm. Orchestrator scores on independent re-run.

## Independent runtime prediction

**15–25 min Mode A.** Mirrors `infer_conj` (`2d3259ae`) with three diffs:
asymmetric arg1-unify targets (i64 for Vec, K for HashMap), Option-wrapped
return, no HashSet arm. The custom-arm recipe is thrice-proven (contains? + conj
+ this is the third Tier-B mirror). Wakeup time-box: **2× upper = 50 min.**

## Scorecard (independent re-run — RAW commands, no wrapper script)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **probe green (regression guard, LOAD-BEARING)** | `cargo test --release --test probe_arc237_7b_intrinsic_typing 2>&1 \| grep "test result"` | `7 passed; 0 failed` |
| 3a | **test-build (the gate part 1, LOAD-BEARING)** | `cargo build --release --tests --workspace 2>&1 \| grep -c "^error"` | 0 |
| 3b | **lib baseline (the gate part 2, LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 4 | **MECHANISM — decl gone** | `grep -c "define-dispatch :wat::core::get" wat/core.wat` | 0 |
| 5 | **MECHANISM — custom arm + eval** | `grep -c '":wat::core::get"' src/check.rs src/runtime.rs` | ≥ 2 |
| 6 | **MECHANISM — infer_get helper** | `grep -c "fn infer_get" src/check.rs` | 1 |
| 7 | **TRAP — Vector arm uses i64_ty (NOT X)** | `awk '/fn infer_get/,/^}/' src/check.rs \| grep -c "i64_ty"` | ≥ 1 |
| 8 | **TRAP — HashMap arm uses K not V** | (covered by probe via the chain of types and the Option<element-of-coll> precision) | — |
| 9 | **MECHANISM — return Option-wrapped** | `awk '/fn infer_get/,/^}/' src/check.rs \| grep -c "wat::core::Option"` | ≥ 2 (one per collection arm) |
| 10 | NO HashSet arm in infer_get | `awk '/fn infer_get/,/^}/' src/check.rs \| grep -c "wat::core::HashSet"` | 0 |
| 11 | NO List arm in eval_get | `awk '/fn eval_get/,/^}/' src/runtime.rs \| grep -c "wat__core__List"` | 0 |
| 12 | scope | `git status --short` | src/check.rs + src/runtime.rs + wat/core.wat + the SCORE; NO holon-rs; NO probe edits |

**FM-9:** independently re-run rows 2 + 3a + 3b, and rows 4/5/6/7/9/10/11
(mechanism actually changed + traps avoided + no scope creep). The probe is the
regression guard (especially `get_vector_precise_element_typing` — flipping it
means the Option-wrap or element-precision is broken).

## Mode classification
- **Mode A:** all rows green; ≤ STOP-2; pattern is the third mirror.
- **Mode B:** probe red (`get_vector_precise_element_typing` flip = silent
  precision loss); Vector arm unifies with X instead of i64; HashMap arm
  unifies with V instead of K; return loses Option wrap; HashSet/List arm
  added; registry/other-ops/holon-rs touched; wrapper script invoked. Any →
  re-brief.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` +
  Mode-B-time.

## On green
Atomic commit: `src/check.rs` + `src/runtime.rs` + `wat/core.wat` +
`SCORE-STONE-237.7b-iv.md`. Advance: **237.7b-iv shipped — the four collection
ops are all ∀T intrinsics now. Tier-B custom-arm recipe proven on all three
variants (concrete-bool / type-preserving / Option-wrapped).** Remaining in arc
237.7: `assoc` (multi-impl HashMap + Record, the records-doctrine slice) →
237.7c, then `DispatchRegistry` deletion after 237.8 arithmetic.
