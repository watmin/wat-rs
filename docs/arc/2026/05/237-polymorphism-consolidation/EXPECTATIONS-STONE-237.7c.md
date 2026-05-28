# EXPECTATIONS — Stone 237.7c — `:wat::core::assoc` polymorphic HashMap+Record. Orchestrator scores on independent re-run.

## Independent runtime prediction

**15–25 min Mode A.** Recipe is thrice-proven (7b-ii/iii/iv); ONE new shape
(Path-with-free-T arg2 for Record umbrella); runtime workhorses already exist
(no inner helper minting). The Record arm is simpler than the HashMap arm
(no element-typing on arg2). Probe edits are mechanical (delete 2 `#[ignore]`
lines).

7b-iv landed in 9.5min (Mode-A, well under the band). 7c has the same skeleton
plus one cleaner arm and a mechanical probe un-ignore — expect comparable.

Wakeup time-box: **2× upper = 50 min**.

## Scorecard (independent re-run — RAW commands, no wrapper script)

| # | Row | Verify | Mode-A pass |
|---|-----|--------|-------------|
| 1 | compile clean | `cargo build --release -p wat 2>&1 \| grep -c "^error"` | 0 |
| 2 | **probe green (post-un-ignore, LOAD-BEARING)** | `cargo test --release --test probe_arc237_7c_assoc_polymorphic 2>&1 \| grep "test result"` | `6 passed; 0 failed; 0 ignored` |
| 3a | **test-build (gate part 1, LOAD-BEARING)** | `cargo build --release --tests --workspace 2>&1 \| grep -c "^error"` | 0 |
| 3b | **lib baseline (gate part 2, LOAD-BEARING)** | `cargo test --release --lib -p wat 2>&1 \| grep "test result"` | `>= 834 passed; 0 failed` |
| 4 | **MECHANISM — alias gone** | `grep -c "define-alias :wat::core::assoc" wat/core.wat` | 0 |
| 5 | **MECHANISM — tombstone in place** | `grep -c "237.7c" wat/core.wat` | ≥ 1 |
| 6 | **MECHANISM — infer_assoc helper** | `grep -c "fn infer_assoc" src/check.rs` | 1 |
| 7 | **MECHANISM — custom arm + eval arm wired** | `grep -c '":wat::core::assoc"' src/check.rs src/runtime.rs` | ≥ 2 |
| 8 | **TRAP — HashMap arm uses V (not K) for arg2** | (covered by probe row `assoc_hashmap_wrong_value_type_rejected_at_check` — if HashMap arm unified arg2 with K, the wrong-value test would silently pass when it shouldn't or fail when it should pass) | — |
| 9 | **TRAP — Record arm uses :keyword for arg1** | (covered by probe: passing `:value` keyword to base/holonic Record works; non-keyword would fail) | — |
| 10 | **TRAP — Record arm does NOT unify arg2** | `awk '/fn infer_assoc/,/^}/' src/check.rs \| grep -c "wat::Record"` | ≥ 1 (the Record arm exists); inspect the body — arg2 must NOT have a unify-call within the Record arm |
| 11 | **PARITY — holonic flavor preserved** | (covered by probe row `assoc_holonic_record_returns_holonic_record_parity_preserved` — passing through `eval_record_assoc` preserves the holonic variant; if routing accidentally degrades to base, accessor would still read the value, but downstream holon-ops would fail. The probe's load-bearing check is the i64 round-trip.) | — |
| 12 | NO touch of per-Type leaves | `git diff --stat HEAD \| grep -E "(eval_record_assoc\|hashmap_assoc_inner)"` | 0 lines changed inside those fn bodies (allow line-number shifts; semantic edits inside their bodies are STOP-triggered) |
| 13 | scope | `git status --short` | `src/check.rs` + `src/runtime.rs` + `wat/core.wat` + `tests/probe_arc237_7c_assoc_polymorphic.rs` + the SCORE; NO holon-rs; NO additional probe edits |

**FM-9:** independently re-run rows 2 + 3a + 3b (load-bearing greens), and rows
4/5/6/7/10/13 (mechanism actually changed + traps avoided + no scope creep).
The probe is the regression guard — especially the two un-ignored rows
(`assoc_base_record_returns_base_record_struct_only` +
`assoc_holonic_record_returns_holonic_record_parity_preserved`).

## Mode classification

- **Mode A:** all rows green; ≤ STOP-2; pattern is the fourth Tier-B mirror.
- **Mode B:**
  - probe red (the un-ignored Record rows fail) → routing through
    `eval_record_assoc` broken, or `infer_assoc`'s Record arm misshaped
  - HashMap-arm K-vs-V swap (regression on row `assoc_hashmap_wrong_value_type_rejected_at_check`)
  - Record-arm arg2 unification added (would over-constrain free ∀T)
  - flavor lost (holonic → base post-assoc; row 5 fails silently or downstream)
  - scope creep: registry/other-ops/holon-rs/per-Type-leaf-body touched
  - wrapper script invoked
  - probe `#[ignore]` re-added to "make build green"
  Any → re-brief.
- **Time-violation:** wakeup fires with sonnet running → `TaskStop` + Mode-B-time.

## On green

Atomic commit: `src/check.rs` + `src/runtime.rs` + `wat/core.wat` +
`tests/probe_arc237_7c_assoc_polymorphic.rs` + `SCORE-STONE-237.7c.md`.

Mirror the 7b-iv commit message shape (`git show fad1c1c6`).

Advance: **237.7c shipped — `:wat::core::assoc` is now a polymorphic ∀T
intrinsic spanning HashMap + Record (the records-doctrine slice). All four
collection ops (length, empty?, contains?, conj, get, assoc) and the
records-pair-mutation verb are now intrinsics; the surface-level assoc alias
is HARD CUT.**

Remaining in arc 237: 237.8 arithmetic (concrete-per-type defclauses + DELETE
widest-contagion + HARD CUT arc-146 `DispatchRegistry`); 237.9 INSCRIPTION
(folds arc 146 + arc 148 closures). Or pause at the seam (decision-point on
return).
