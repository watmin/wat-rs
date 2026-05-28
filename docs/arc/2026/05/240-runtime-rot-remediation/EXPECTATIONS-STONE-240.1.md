# EXPECTATIONS — Stone 240.1 — first/rest List arm + Bundle alias-unfold. Orchestrator scores on independent re-run.

## Independent runtime prediction

**10–20 min Mode A.** Two surgical check.rs edits, both mirroring existing code
(B mirrors the Vec arm; C mirrors infer_positional_accessor's `reduce` usage).
No new types, no runtime work. Wakeup time-box: **2× upper = 40 min.**

## Scorecard (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **B: first-on-List (LOAD-BEARING)** | `cargo test --release --test wat_arc220_list 2>&1 \| grep "test result"` | `23 passed; 0 failed` |
| 3 | **C: Bundle-Holons (LOAD-BEARING)** | `cargo test --release --test wat_bundle_capacity 2>&1 \| grep "test result"` | `7 passed; 0 failed` |
| 4 | **lib baseline (LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 5 | MECHANISM — List arm present | `grep -c "wat::core::List" src/check.rs` (in infer_positional_accessor region) | ≥ 1 new arm |
| 6 | MECHANISM — reduce in bundle | `grep -n "reduce(&t, subst" src/check.rs` (infer_holon_bundle other-branch) | present |
| 7 | scope | `git status --short` | only `src/check.rs` + the SCORE; NO runtime.rs; NO holon-rs |

**FM-9:** independently re-run rows 2 + 3 + 4. wat_arc220_list goes 21→23 passed
(the 2 first/conj failures clear); wat_bundle_capacity goes 6→7.

## Mode classification
- **Mode A:** all rows green; both fixes purely additive; ≤ STOP-2.
- **Mode B:** any guard still red, lib regressed, runtime.rs touched, holon-rs
  touched, or a path-name special-case used for the alias. Any → re-brief.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` + Mode-B-time.

## On green
Commit `src/check.rs` + `SCORE-STONE-240.1.md` as one commit. Advance: 240.1 shipped
(B + C); NEXT = 240.2 (stone5c stale-test) then 240.3 (consumer .wat sweep).
