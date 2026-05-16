# Arc 170 Slice 4a-γ-decorate EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4A-GAMMA-DECORATE-FLAGGED-DEFTESTS.md`
**Task:** #318

## Independent prediction

**Runtime band:** 15–25 minutes Mode A.

Reasoning:
- 4 mechanical decorations (sites 132, 146, 188, 207) — 1-2 lines each
- 1 rearchitecture (site 154) — body reshape ~10 lines + comment refresh; test-first verification cycle
- 2 duplicate-marker comment blocks (sites 188, 207) — 6-line comment each
- Build + test verification: ~5 min
- Sonnet's overhead for cwd verification + reading docs: ~3 min

**Time-box:** ScheduleWakeup at 50 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with explicit grep / build / test evidence per `feedback_four_questions_yes_no` + `feedback_assertion_demands_evidence`:

- **Row A** (5 deftest-hermetic decorations):
  ```bash
  grep -nE "^\(:wat::test::deftest-hermetic\b" /home/watmin/work/holon/wat-rs/wat-tests/test.wat
  ```
  Returns ≥ 5 lines (matched test names: test-assert-stdout-is-matches, test-assert-stderr-matches-pass, test-assert-stderr-matches-fail-reports-pattern, test-run-string-entry-path, test-run-ast-via-program).

- **Row B** (site 154 uses run-hermetic):
  ```bash
  awk '/test-assert-stderr-matches-fail-reports-pattern/,/^\(:wat::test::deftest/' /home/watmin/work/holon/wat-rs/wat-tests/test.wat | grep -c ":wat::test::run-hermetic\b"
  ```
  Returns 2 (outer + inner) ; `grep -c ":wat::test::run-thread\b"` returns 0.

- **Row C** (site 154 inner has eprintln "different content"):
  ```bash
  awk '/test-assert-stderr-matches-fail-reports-pattern/,/^\(:wat::test::deftest/' /home/watmin/work/holon/wat-rs/wat-tests/test.wat | grep -F 'eprintln "different content"'
  ```
  Returns at least one match.

- **Row D** (duplicate markers on 188 and 207):
  ```bash
  grep -B 6 "test-run-string-entry-path\|test-run-ast-via-program" /home/watmin/work/holon/wat-rs/wat-tests/test.wat | grep -i "duplicate"
  ```
  Returns at least 2 matches (one per site).

- **Row E** (build clean):
  ```bash
  cargo build --release --workspace --tests 2>&1 | tail -5
  ```
  Shows `Finished`, zero errors. Warnings unchanged from baseline.

- **Row F** (all 5 tests pass; no regression):
  ```bash
  cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "(test-assert-stdout-is-matches|test-assert-stderr-matches-pass|test-assert-stderr-matches-fail-reports-pattern|test-run-string-entry-path|test-run-ast-via-program)"
  ```
  All 5 in `... ok` set.
  ```bash
  cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "^test result" | awk '{f += $6} END {print f}'
  ```
  Returns ≤ 11 (post-4a-β rotation band).

## Honest deltas to watch for

- **Site 154 rearchitecture surfaces substrate behavior change.** The new body uses real captured stderr content (`"different content"`) — the Failure's `actual` slot will now populate with that line. The existing assertion only checks `expected` slot; the test should still pass. If `expected` slot semantics CHANGED when `actual` is populated (substrate edge case), surface — that's substrate-side information.

- **Comment styling.** The duplicate-marker comments on 188/207 should match the codebase's existing comment conventions (semicolons, indentation, line length). Look at adjacent comments in test.wat to match style.

- **Line-number drift.** Adding 6-line comment blocks to sites 188/207 shifts subsequent line numbers. The 5 deftest-hermetic line numbers in Row A may differ from the BRIEF's stated lines (132, 146, 154, 188, 207) — confirm by test name, not line number.

- **Pressure-flake rotation.** Post-4a-β baseline was 10 failed (variance band 8-11). The 5 modified deftests should land in PASSED. If they appear in FAILED, surface — that's a genuine regression. If pre-existing-rotation tests change set membership but count stays ≤11, that's flake noise (acknowledge but not a regression).

- **Site 154's rearchitecture might reveal redundancy with another test.** If a test elsewhere already exercises real stderr capture + non-match (e.g., a deftest in service-template.wat or holon/), surface in honest deltas. The duplicate-marker pattern from 188/207 might apply.

## Workspace baseline (commit `f2e78ea`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2263 passed / 10 failed (rotation band 8-11; same composition as post-4a-β)

Post-slice-4a-γ-decorate target:
- 2263+ passed (5 modified deftests stay in PASSED set; no new tests added)
- ≤ 11 failed (no new regressions)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–25 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ 11 | TBD |
| Sites decorated | 5 | TBD |
| Site 154 rearchitecture revealed substrate behavior | none expected | TBD |
| Other duplicates surfaced | 0–2 | TBD |
| Mode | A (clean) | TBD |
