# EXPECTATIONS — Stone 237.7b-ii — `contains?` ∀T intrinsic with custom arm. Orchestrator scores on independent re-run.

## Independent runtime prediction

**20–35 min Mode A.** Custom inference arm (mirror `infer_positional_accessor`)
with 3 collection-type branches + arg1 unification + a 3-arm eval. More surface
than 7b-i's length-mirror but the pattern is established (`first`/`get` already
use custom arms). The HashMap-key-vs-element distinction is the trickiest case
(easy to mis-route to V). Wakeup time-box: **2× upper = 70 min.**

## Scorecard (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **probe green (regression guard, LOAD-BEARING)** | `cargo test --release --test probe_arc237_7b_intrinsic_typing 2>&1 \| grep "test result"` | `7 passed; 0 failed` |
| 3 | **green-gate (LOAD-BEARING)** | `./scripts/green-gate.sh 2>&1 \| tail -3` | `green-gate: PASS`, lib `>= 834 passed; 0 failed` |
| 4 | **MECHANISM — decl gone** | `grep -c "define-dispatch :wat::core::contains?" wat/core.wat` | 0 |
| 5 | **MECHANISM — custom arm + eval** | `grep -c '":wat::core::contains?"' src/check.rs src/runtime.rs` | ≥ 2 (custom arm + eval arm) |
| 6 | **MECHANISM — infer_contains helper exists** | `grep -c "fn infer_contains" src/check.rs` | 1 |
| 7 | **MECHANISM — wrong-elem still rejects** | (covered by row 2: `contains_q_wrong_element_rejected_at_check` is one of the 7) | — |
| 8 | other ops intact | `grep -c "define-dispatch :wat::core::get" wat/core.wat` + same for `conj` | 1 each (= 2 total) |
| 9 | NO List arm sneaked in | `awk '/fn eval_contains/,/^}/' src/runtime.rs \| grep -c "wat__core__List"` | 0 |
| 10 | scope | `git status --short` | src/check.rs + src/runtime.rs + wat/core.wat + the SCORE; NO holon-rs; NO probe edits |

**FM-9:** independently re-run rows 2 + 3, and rows 4/5/6 (mechanism actually
changed — not just behavior preserved). The probe is the regression guard
(green before AND after); rows 4–6 prove the swap really happened.

## Mode classification
- **Mode A:** all rows green; pattern mirrors `infer_positional_accessor`
  cleanly; ≤ STOP-3.
- **Mode B:** probe red (especially `contains_q_wrong_element_rejected_at_check`
  flipping to fail = element-typing not enforced); lib/build regressed; decl
  not deleted; HashMap arm wrong (checks V not K); List arm added;
  registry/other-ops/holon-rs touched. Any → re-brief.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` + Mode-B-time.

## On green
Atomic commit: `src/check.rs` + `src/runtime.rs` + `wat/core.wat` + `SCORE-STONE-237.7b-ii.md`.
Advance: 237.7b-ii shipped (`contains?` = intrinsic with custom arm; Tier-B
recipe confirmed); NEXT = 237.7b-iii (`conj`, custom-arm mirror with
type-preservation).
