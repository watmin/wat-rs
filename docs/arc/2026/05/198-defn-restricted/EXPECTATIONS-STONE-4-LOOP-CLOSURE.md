# Arc 198 Slice 2 Stone 4 EXPECTATIONS

**BRIEF:** `BRIEF-STONE-4-LOOP-CLOSURE.md`

## Independent prediction

**Runtime band:** 45 minutes sonnet.

Reasoning:
- Delete `validate_join_result_user_namespace` fn: ~30-50 LOC removed
- Delete `CheckError::JoinResultUserNamespace` variant + Display + Diagnostic: ~30-50 LOC removed
- Remove hook in `check_program`: 1-3 LOC removed
- Update Stone B's 4 tests: ~10-20 LOC changed (replace "drain-and-join" with arc 198's wording)
- Verify with cargo test
- Possibly minor orphaned-import cleanup

**Time-box:** 90 min hard stop.

## SCORE methodology

6 rows YES/NO per BRIEF:

- **Row A** (fn deleted): grep returns empty
- **Row B** (variant deleted): grep returns empty
- **Row C** (hook removed): grep returns empty
- **Row D** (Stone B tests pass): targeted cargo test green
- **Row E** (predecessors green): targeted runs all green
- **Row F** (workspace baseline): cargo test summed failed ≤ baseline + flake variance

## Honest deltas to watch for

- **Arc 198's exact error wording.** Per the Display impl seen earlier: contains "def-restricted binding", "allowed-caller whitelist", "namespace prefix", "exact-FQDN match", AND the callee name. Stone B's tests grep for "drain-and-join" — replace with "def-restricted" or "allowed-caller" or similar. The callee name assertions (Thread/join-result, Process/join-result) stay.

- **Line-number drift.** Stone 3's outcome noted line numbers had drifted from BRIEF estimates (16722/16340 → 17041/16531 for the eval_kernel fns). Sonnet should grep current line numbers, not trust BRIEF.

- **Combined-walker behavior during transition.** Both walkers fire in Stone 3's state. After Stone 4 deletion, only arc 198 fires. The output goes from "Stone B error + arc 198 error" to just "arc 198 error." Stone B's old assertion `err.contains("drain-and-join")` would FAIL because that substring only came from Stone B's rule. Hence the test update.

- **Orphaned imports.** Deleting `validate_join_result_user_namespace` may leave unused imports in `src/check.rs`. Build will warn or fail; sonnet cleans up.

- **Unused helper fns.** If `validate_join_result_user_namespace` used helpers that have no other callers post-deletion, sonnet may need to delete those too (or leave them with `#[allow(dead_code)]` if substrate convention requires retention). Surface in SCORE.

- **Sibling tests for Stone B's positive cases.** The 2 positive tests (substrate-namespace callers ALLOWED) shouldn't need changes — they assert no compile error fires. Arc 198's walker doesn't fire on substrate-namespace callers either. So positive tests stay.

## Workspace baseline (commit `fe2e0eb`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures + lifeline flake variance

Post-Stone-4 target:
- = baseline passed count + 0 (no new tests; Stone B's 4 tests stay green via updated assertions)
- ≤ baseline failures (no regressions)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 45 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ baseline | TBD |
| New test count | 0 (Stone B's tests UPDATED, not new) | TBD |
| LOC deleted | ~60-100 | TBD |
| Arc 198 wording in tests | "def-restricted" OR "allowed-caller" OR similar | TBD |
| Substrate-discovery surprises | 0-2 | TBD |
| Mode | Subtractive (delete redundant rule + update test assertions) | TBD |
