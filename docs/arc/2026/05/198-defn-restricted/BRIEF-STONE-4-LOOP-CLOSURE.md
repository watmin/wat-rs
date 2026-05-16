# Arc 198 Slice 2 Stone 4 BRIEF — loop closure: delete Stone B's ad-hoc rule + update tests

**Arc:** 198 slice 2, Stone 4 of 4 — FINAL stone.
**Task:** #328
**Predecessors:**
- Stone 1 (commit `51c69a1`) — inventory wiring
- Stone 2 (commit `6775510`) — proc-macro attribute
- Stone 3 (commit `fe2e0eb`) — attribute applied to `eval_kernel_*_join_result`; both walkers fire

**Successor:** arc 170 Stone C (Client/Server type pairs) — bracket chain resumes after this loop closure.

## Goal

Close the loop: delete arc 170 Stone B's ad-hoc walker rule (now redundant because arc 198's walker fires for the same callees post-Stone-3), then update Stone B's 4 tests to assert on arc 198's error format.

**Two pieces:**

1. **Delete Stone B's ad-hoc walker rule** in `src/check.rs`:
   - The `validate_join_result_user_namespace` fn (~line 3094)
   - The `CheckError::JoinResultUserNamespace` variant (~line 667) + its Display impl + Diagnostic impl
   - The hook into `check_program` (~line 1939)

2. **Update Stone B's 4 tests** in `tests/wat_arc170_stone_b_walker_collapse.rs`:
   - Negative-test assertions currently grep `Thread/join-result` AND `drain-and-join`
   - "Thread/join-result" still appears in arc 198's error (callee name is part of the diagnostic) — keep
   - "drain-and-join" does NOT appear in arc 198's error — replace with something arc 198 actually emits (e.g., `def-restricted` or `DefRestrictedCallerNotAllowed`)
   - Positive-test assertions (substrate-namespace callers ALLOWED) are unaffected — keep

After this stone:
- Only ONE walker enforces *_join-result restriction (arc 198's `walk_for_def_restricted_call`)
- Stone B's tests still exist (history preserved) and still verify the SAME enforcement, just via the new mechanism
- Arc 198 slice 2 is COMPLETE
- Arc 198 itself is ready for INSCRIPTION (a separate slice — possibly bundled with arc 170 closure later)

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — exact line numbers for deletion (BRIEF pointers may have drifted), arc 198's exact error wording for test assertions, whether the deletion needs additional cleanup (orphaned imports, unused error variants in helpers, etc.). Do NOT trust orchestrator claims without grep verification.

## Substrate state pointers (verified at Stone 2 time; may have drifted slightly per Stone 3's discovery)

- `src/check.rs:3094` (approx) — `validate_join_result_user_namespace` (DELETE)
- `src/check.rs:1939` (approx) — Stone B's hook into `check_program` (DELETE the call site)
- `src/check.rs:667+` (approx) — `CheckError::JoinResultUserNamespace` variant (DELETE — variant + Display arm + Diagnostic arm)
- `tests/wat_arc170_stone_b_walker_collapse.rs` — 4 tests; update assertions
- Stone 2's `SCORE-STONE-2-PROC-MACRO-ATTRIBUTE.md` — verify arc 198's error wording
- arc 198 slice 1's `walk_for_def_restricted_call` walker — this is what FIRES post-Stone-4 for `*_join-result`

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read substrate state.** Locate exact line numbers via grep (don't trust orchestrator's BRIEF pointers blindly):
   ```
   grep -n "validate_join_result_user_namespace\|JoinResultUserNamespace" src/check.rs
   ```

2. **Verify arc 198's error wording.** Run a probe test or grep `DefRestrictedCallerNotAllowed` arm in `src/check.rs` Display impl. The wording is what tests need to assert on.

3. **Update Stone B's 4 tests FIRST.** Replace "drain-and-join" assertions with arc 198's actual wording (e.g., `assert!(err.contains("def-restricted"))` or similar — sonnet picks based on the diagnostic).
   - Keep: assertions on callee verb name (`Thread/join-result`, `Process/join-result`) — these appear in BOTH walkers' messages
   - Replace: "drain-and-join" → arc-198-specific wording
   - Positive-tests stay unchanged
   - RUN tests — they will FAIL initially because Stone B's rule still fires (and contributes its "drain-and-join" message to the combined error output, so the OLD assertion still passes; the NEW assertion expects arc-198 wording which is ALSO present). Actually — both walkers fire so combined output has BOTH wordings. The tests might pass even before deletion. Sonnet observes + decides.

4. **Delete Stone B's ad-hoc rule** in `src/check.rs`:
   - `validate_join_result_user_namespace` fn
   - `CheckError::JoinResultUserNamespace` variant + Display arm + Diagnostic arm
   - Hook in `check_program`
   - Any orphaned imports

5. **Build clean.** `cargo build --release --workspace --tests`.

6. **Run Stone B's 4 tests.** Should be green (assertions now match arc 198's error format which is the SOLE walker firing).

7. **Run ALL predecessor tests.** Verify 17/17 across Stones A + B + arc 198 slice 1 + slice 2 stones 1-3.

8. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (3 stable + flake variance).

9. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd.
- DO NOT modify arc 198 slice 1's `walk_for_def_restricted_call` or `DefRestrictedCallerNotAllowed` — they stay.
- DO NOT modify Stone 1's `RestrictionEntry` or `src/freeze.rs` iteration.
- DO NOT modify Stone 2's `#[restricted_to(...)]` proc-macro.
- DO NOT modify Stone 3's attribute applications on `eval_kernel_*_join_result`.
- DO NOT touch arc 198 slice 1's wat-side `def-restricted` / `defn-restricted` forms.
- DO NOT touch Stone A's `drain-and-join` helpers.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs/EXPECTATIONS/SCOREs / this BRIEF / this EXPECTATIONS / superseded slice-2 monolithic BRIEF + EXPECTATIONS.
- DO NOT update USER-GUIDE / docs — that's a separate concern.
- DO NOT delete arc 170 Stone B's ~40 caller migrations from user-namespace `*_join-result` to `*_drain-and-join` — those migrations are independent of this walker change.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.

## Scorecard (6 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `validate_join_result_user_namespace` fn DELETED | `grep -n "validate_join_result_user_namespace" src/check.rs` returns empty |
| B | `CheckError::JoinResultUserNamespace` variant DELETED (variant + Display + Diagnostic) | `grep -n "JoinResultUserNamespace" src/check.rs` returns empty |
| C | Stone B walker hook in `check_program` REMOVED | `grep -nA 3 "validate_join_result\|JoinResultUserNamespace" src/check.rs` returns empty |
| D | Stone B's 4 tests pass with arc 198's error format assertions | `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse` → 4/4 green |
| E | All other predecessor tests still green (Stone A, arc 198 slice 1, Stone 1, Stone 2, Stone 3) | targeted runs all green; 19/19 across stones (Stone B 4/4 + Stone A 4/4 + slice 1 5/5 + Stone 1 1/1 + Stone 2 3/3 + Stone 3 2/2) |
| F | Workspace test failure count ≤ baseline (3 stable + flake variance) | full workspace cargo test failures ≤ baseline |

## STOP triggers

- Arc 198's error wording doesn't contain ANY substring suitable for the negative-test assertions → STOP and surface (may need to extend arc 198's error message)
- Deleting Stone B's rule causes orphaned imports / unused warnings that break build → fix locally; if more than 3 orphans, STOP
- Stone B's tests fail in ways that suggest arc 198's walker isn't firing → STOP and root-cause
- Predecessor test regresses → STOP
- > 3 unexpected substrate-finding surfaces → STOP

## Workspace baseline (commit `fe2e0eb`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures (t6, totally_bogus, startup_error) + lifeline flake variance

Post-Stone-4 target:
- = baseline passed count + 0 (Stone B's 4 tests stay green via updated assertions; no new tests)
- ≤ baseline failures (no regressions)

## Time-box

45 min predicted. Hard stop 90 min. If approaching stop, write partial SCORE.

## On completion

Write `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-4-LOOP-CLOSURE.md`:
- 6 rows YES/NO with grep-able evidence
- Honest deltas: exact arc 198 error wording used in test assertions, any orphaned-import cleanup, line-number drift from BRIEF estimates
- Calibration record

Return final summary: rows passed/failed + arc 198 error wording in tests + workspace delta + path to SCORE.

You are launching now. T-minus 0.
