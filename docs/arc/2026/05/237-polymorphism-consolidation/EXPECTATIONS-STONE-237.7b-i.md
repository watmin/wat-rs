# EXPECTATIONS — Stone 237.7b-i — `empty?` ∀T intrinsic. Orchestrator scores on independent re-run.

## Independent runtime prediction

**10–20 min Mode A.** Exact `length` mirror (one scheme + one 3-arm eval handler +
one decl delete + dispatch-arm wire); the recipe is proven (237.7a) and the typing
is Tier A (concrete bool return, no custom inference). Wakeup time-box: **2× upper = 40 min.**

## Scorecard (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **probe green (regression guard, LOAD-BEARING)** | `cargo test --release --test probe_arc237_7b_intrinsic_typing 2>&1 \| grep "test result"` | `5 passed; 0 failed` |
| 3 | **green-gate (LOAD-BEARING)** | `./scripts/green-gate.sh 2>&1 \| tail -1` + lib line | `green-gate: PASS`; lib `>= 834 passed; 0 failed` |
| 4 | **MECHANISM — decl gone** | `grep -c "define-dispatch :wat::core::empty?" wat/core.wat` | 0 |
| 5 | **MECHANISM — builtin present** | `grep -c '":wat::core::empty?"' src/check.rs src/runtime.rs` | ≥ 2 (scheme + eval arm) |
| 6 | other ops intact | `grep -c "define-dispatch :wat::core::\(contains?\|get\|conj\)" wat/core.wat` | 3 (untouched) |
| 7 | NO List arm added | `grep -c "wat__core__List" src/runtime.rs` (eval_empty region) | 0 new in eval_empty |
| 8 | scope | `git status --short` | src/check.rs + src/runtime.rs + wat/core.wat + the SCORE; NO holon-rs; NO probe edits |

**FM-9:** independently re-run rows 2 + 3, and rows 4/5 (mechanism actually changed).
The probe is the regression guard (green before AND after); rows 4–6 prove the swap
happened + is isolated to `empty?`.

## Mode classification
- **Mode A:** all rows green; cascade mechanical; ≤ STOP-2.
- **Mode B:** probe red, lib/build regressed, decl not deleted, List arm added,
  registry/other-ops/holon-rs touched. Any → re-brief.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` + Mode-B-time.

## On green
Atomic commit: `src/check.rs` + `src/runtime.rs` + `wat/core.wat` + `SCORE-STONE-237.7b-i.md`.
Advance: 237.7b-i shipped (empty? = intrinsic; bool-return recipe proven); NEXT =
237.7b-ii (contains? Tier A + conj type-preserving).
