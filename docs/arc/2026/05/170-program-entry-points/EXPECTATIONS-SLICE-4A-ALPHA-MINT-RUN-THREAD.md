# Arc 170 Slice 4a-α EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4A-ALPHA-MINT-RUN-THREAD.md`
**Task:** #308

## Independent prediction

**Runtime band:** 15–30 minutes Mode A.

Reasoning:
- Three small define forms in wat/test.wat (~30 lines total combined)
- One standalone test file (~50 lines for both Ok-path and Err-path deftests)
- Test-first build cycle: ~5 incremental builds
- No sweep work, no existing test migrations, no consumer churn
- Workspace test rerun adds ~3 min
- Padding for: filename convention survey, FQDN spelling lookup, possible Thread<I,O> type-signature parse fix

**Time-box:** ScheduleWakeup at 60 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with explicit grep / build / test evidence per `feedback_four_questions_yes_no` + `feedback_assertion_demands_evidence`:

- **Row A** (`failure-from-thread-died` defined):
  ```bash
  grep -nE "^\(:wat::core::define" wat/test.wat | grep "failure-from-thread-died"
  ```
  Returns at least one line. Visual inspection confirms signature is `(chain :wat::core::Vector<wat::kernel::ThreadDiedError>) -> :wat::kernel::Failure`.

- **Row B** (`run-thread-driver` defined with `Thread<nil,nil> -> RunResult`):
  ```bash
  grep -nA 4 "run-thread-driver" wat/test.wat | grep "Thread<wat::core::nil"
  ```
  Returns at least one match.

- **Row C** (`run-thread` defmacro):
  ```bash
  grep -nE "^\(:wat::core::defmacro" wat/test.wat | grep "run-thread\b"
  ```
  Returns at least one line. Visual confirms the body is the spawn-thread expansion.

- **Row D** (standalone deftests both paths):
  ```bash
  find wat-tests/ tests/ -name "*run_thread*" -o -name "*run-thread*"
  grep -nE "(run-thread-ok-path|run-thread-err-path)" <found_file>
  ```
  Returns two test-name matches. Visual confirms Err-path asserts `:Some` failure (not `:None`).

- **Row E** (build clean):
  ```bash
  cargo build --release --workspace --tests 2>&1 | tail -5
  ```
  Shows `Finished`, zero errors, zero warnings related to the new code.

- **Row F** (workspace tests; new deftests pass; no regressions):
  ```bash
  cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "^test result"
  ```
  Summed failures ≤ 11 (post-Phase-3 baseline).
  ```bash
  cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "(slice-4a-alpha::run-thread-ok-path|slice-4a-alpha::run-thread-err-path)"
  ```
  Both test names appear in the PASSED set, not FAILED.

## Honest deltas to watch for

- **`failure-from-thread-died` placement.** I'm proposing wat/test.wat next to run-thread-driver. If sonnet finds a stronger placement argument (e.g., a thread-kernel file like `wat/kernel/thread.wat` exists or is conventionally expected for symmetry with hermetic.wat), surface in SCORE and propose. Don't force the test.wat placement if there's a more honest home.

- **`Thread<wat::core::nil,wat::core::nil>` FQDN spelling.** The exact angle-bracket FQDN form may need adjustment per the codebase's existing Thread<I,O> consumers. `grep -rE "Thread<" src/ wat/` first to confirm convention. If the spelling differs from the BRIEF's example, note the working form in SCORE.

- **Test file naming convention.** Survey `find wat-tests/ tests/ -name "*hermetic*"` to locate run-hermetic's Layer 1 verification test. Match that pattern. If run-hermetic has no dedicated test file, decide between adding a new file vs appending to an existing related test file — note the choice in SCORE delta.

- **Err-path assertion shape.** The BRIEF sketches `(:wat::core::match ...)` destructuring for `:Option<Failure>`. If a cleaner accessor pattern exists (`RunResult/failure` already exists; perhaps `Option/is-some?` or similar), use it. Note the chosen idiom in SCORE.

- **Workspace pressure flake.** Per FD-multiplex Phase 3 notes, the workspace has ~11 stable failures from substrate contention under fanout (composition rotates). New deftests may flake under workspace pressure even if passing in isolation. If 11 → 11 with the new tests in passed set, that's clean. If failures jump to 12+, surface the regression class.

- **`assert-eq` against `:wat::core::None`.** The Ok-path test asserts `(:wat::test::assert-eq :wat::core::None (:wat::kernel::RunResult/failure result))`. Verify assert-eq can compare Option values directly, or use match/destructure if it can't. Note in SCORE.

## Workspace baseline (commit 5a7441c)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2260 passed / 11 failed (composition rotates; ≤ 11 is the gate, not exact set)

Post-slice-4a-α target:
- 2262+ passed (two new deftests join the PASSED set; existing tests unchanged)
- ≤ 11 failed (no new regressions)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 15–30 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ 11 | TBD |
| New deftests passed | 2 (Ok + Err) | TBD |
| `failure-from-thread-died` placement | wat/test.wat | TBD |
| New test file path | TBD | TBD |
| FQDN spelling adjustments | none expected | TBD |
| Mode | A (clean) | TBD |
