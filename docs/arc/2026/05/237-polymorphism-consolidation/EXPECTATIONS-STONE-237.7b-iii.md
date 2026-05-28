# EXPECTATIONS — Stone 237.7b-iii — `conj` ∀T intrinsic with custom arm. Orchestrator scores on independent re-run.

## Independent runtime prediction

**15–25 min Mode A.** Strict simplification of 7b-ii (which took ~7min sonnet
time per the prior SCORE): 2 collection types instead of 3, type-preserving
return instead of bool, no HashMap-K-vs-V trap. The recipe is twice-proven
(length + empty? for Tier A; contains? for the Tier-B custom-arm shape). Wakeup
time-box: **2× upper = 50 min.**

## Scorecard (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **probe green (regression guard, LOAD-BEARING)** | `cargo test --release --test probe_arc237_7b_intrinsic_typing 2>&1 \| grep "test result"` | `7 passed; 0 failed` |
| 3 | **green-gate (LOAD-BEARING)** | `./scripts/green-gate.sh 2>&1 \| tail -3` | `green-gate: PASS`, lib `>= 834 passed; 0 failed` |
| 4 | **MECHANISM — decl gone** | `grep -c "define-dispatch :wat::core::conj" wat/core.wat` | 0 |
| 5 | **MECHANISM — custom arm + eval** | `grep -c '":wat::core::conj"' src/check.rs src/runtime.rs` | ≥ 2 (custom arm + eval arm) |
| 6 | **MECHANISM — infer_conj helper exists** | `grep -c "fn infer_conj" src/check.rs` | 1 |
| 7 | **MECHANISM — type-preservation + wrong-elem reject** | (covered by row 2: `conj_vector_preserves_collection_type` + `conj_wrong_element_rejected_at_check` are in the 7) | — |
| 8 | only `get` decl remains in 237.7b scope | `grep -c "define-dispatch :wat::core::get" wat/core.wat` | 1 |
| 9 | NO HashMap arm in conj | `awk '/fn infer_conj/,/^}/' src/check.rs \| grep -c "wat::core::HashMap"` | 0 |
| 10 | NO List arm sneaked in | `awk '/fn eval_conj/,/^}/' src/runtime.rs \| grep -c "wat__core__List"` | 0 |
| 11 | scope | `git status --short` | src/check.rs + src/runtime.rs + wat/core.wat + the SCORE; NO holon-rs; NO probe edits |

**FM-9:** independently re-run rows 2 + 3, and rows 4/5/6/9/10 (mechanism
actually changed + no scope creep). The probe is the regression guard (green
before AND after); rows 4–6 prove the swap happened; 9 + 10 prove sonnet didn't
sneak in a HashMap or List arm.

## Mode classification
- **Mode A:** all rows green; pattern mirrors `infer_contains` minus a branch +
  return diff; ≤ STOP-2.
- **Mode B:** probe red (especially `conj_*` tests flipping); HashMap arm added
  to conj (scope creep); List arm added; type-preservation broken (conj result
  no longer usable as the collection type — `length (conj …)` would fail);
  registry/other-ops/holon-rs touched. Any → re-brief.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` +
  Mode-B-time.

## On green
Atomic commit: `src/check.rs` + `src/runtime.rs` + `wat/core.wat` +
`SCORE-STONE-237.7b-iii.md`. Advance: 237.7b-iii shipped (`conj` = intrinsic
with custom arm; type-preserving Tier-B confirmed); NEXT = 237.7b-iv (`get` —
the `Option<element>` return variant; 2 collection types Vector<T>+i64 and
HashMap<K,V>+K).
