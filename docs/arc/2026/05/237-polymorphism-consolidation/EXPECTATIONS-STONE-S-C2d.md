# EXPECTATIONS — Stone S-C.2d — `:wat::Record/same-data?`

Paired with `BRIEF-STONE-S-C2d.md`. Orchestrator scores against an INDEPENDENT local re-run.

## Independent runtime prediction

**25–45 min Mode A.** One substrate primitive mirroring `eval_record_assoc` (dispatch + eval fn +
scheme) + a small refactor factoring `record_field_map` out of `eval_record_to_map`. The impl is
the composition proven GREEN in the probe's `comp_*` group. Wakeup time-box: **2× upper = 90 min**.

## Scorecard verification (independent re-run)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep "^error"` | 0 |
| 2 | **probe 6/6 (LOAD-BEARING)** | `cargo test --release --test probe_arc237_sC2d_same_data 2>&1 \| grep "test result"` | `6 passed; 0 failed` (comp_* stay green; samedata_* flipped RED→GREEN) |
| 3 | **lib baseline (LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 4 | `=` unaffected | `--test probe_arc238_eq_completeness` | 8/8 |
| 5 | record->map / defrecord regression | defrecord surface probe | green (eval_record_to_map refactor didn't change behavior) |
| 6 | scope | `git status --short` | only `src/runtime.rs` + the probe + SCORE |

**FM-9 on the claim:** re-run rows 2 + 3 independently. Confirm the probe MEASURES type-blindness —
specifically `samedata_cross_type_equal` (`Pt[0,0]` vs `Coord[0,0]` → true) is the load-bearing
contract that proves it ignores class; and `samedata_diff_value` → false proves it still compares
data. Spot-read the eval fn to confirm it routes through `record->map` + `values_equal` (name-keyed),
not a positional struct compare.

## Mode classification

- **Mode A:** all rows green; impl is the composition; ≤ STOP-3.
- **Mode B:** type-aware impl (forbidden); positional compare (forbidden); `eval_record_to_map`
  behavior changed (its probe red); baseline dropped. Any → re-brief.

## Trap-doors (mirror BRIEF)

1. type-aware → REJECT (must be type-blind). 2. positional → REJECT (name-keyed). 3. re-impl map
equality → REJECT (reuse `values_equal`). 4. break `eval_record_to_map` → REJECT. 5. holon-rs /
existing `values_equal` arms touched → REJECT.

## On green

Atomic commit: `src/runtime.rs` + `tests/probe_arc237_sC2d_same_data.rs` + `SCORE-STONE-S-C2d.md`.
Then USER-GUIDE line (`same-data?` = type-blind record data equality, contrast with type-strict `=`)
+ tracker advance: S-C.2d ✓; NEXT = S-C.3 (macro split) → S-D.
