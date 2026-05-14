# Arc 170 Slice 4a-β EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4A-BETA-SWEEP-LEGACY-CALLERS.md`
**Task:** #313

## Independent prediction

**Runtime band:** 25–45 minutes Mode A.

Reasoning:
- 32 active call sites distributed across wat-tests/ + tests/ + crates/ + examples/
- Most follow P1 / P2a / P3 — mechanical: replace the call form with `(run-thread <inlined body>)` or `(run-hermetic <inlined body>)`
- Per-pattern batch verification (5 P1, then 18 P2a, then 9 P3) keeps build cycles fast — ~3 incremental builds total
- P2b sites (computed forms) require case-by-case judgment — expected 0-5; mid-sweep STOP threshold
- Each migration is ~5-line edit at the call site; 32 of them is the bulk
- Workspace test rerun adds ~3 min
- Per-file consolidation (one pass per file) keeps the diff coherent

**Time-box:** ScheduleWakeup at 90 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with explicit grep evidence per `feedback_four_questions_yes_no` + `feedback_assertion_demands_evidence`:

- **Row A** (zero `:wat::test::run` callers):
  ```bash
  grep -rE ":wat::test::run[^-A-Za-z]" wat-tests/ tests/ crates/ examples/ \
    | grep -v "^Binary" | grep -v "\.md:" | grep -v "^\s*;;" | wc -l
  ```
  Returns 0 active hits (excluding `run-thread` / `run-hermetic` which start with `run-` and are matched out by `[^-A-Za-z]`).

- **Row B** (zero `:wat::test::run-ast` callers):
  ```bash
  grep -rE ":wat::test::run-ast\b" wat-tests/ tests/ crates/ examples/ \
    | grep -v "^Binary" | grep -v "\.md:" | wc -l
  ```
  Returns 0 active hits.

- **Row C** (zero `:wat::test::run-hermetic-ast` callers):
  ```bash
  grep -rE ":wat::test::run-hermetic-ast\b" wat-tests/ tests/ crates/ examples/ \
    | grep -v "^Binary" | grep -v "\.md:" | wc -l
  ```
  Returns 0 active hits. Note `:wat::test::run-hermetic` (no `-ast` suffix) is allowed; only `-ast` retires.

- **Row D** (new thread-based callers use `run-thread`):
  ```bash
  grep -rEn ":wat::test::run-thread\b" wat-tests/ tests/ crates/ examples/ \
    | grep -v "^Binary" | grep -v "\.md:" | wc -l
  ```
  Returns ~24 hits (23 migrated callers + 1 from the standalone test at `wat-tests/run-thread.wat`). Delta from baseline (which had ~1) ≈ +23.

- **Row E** (new hermetic callers use `run-hermetic`):
  ```bash
  grep -rEn ":wat::test::run-hermetic\b" wat-tests/ tests/ crates/ examples/ \
    | grep -v "^Binary" | grep -v "\.md:" | wc -l
  ```
  Delta from baseline ≈ +9 (the migrated `run-hermetic-ast` callers).

- **Row F** (build clean):
  ```bash
  cargo build --release --workspace --tests 2>&1 | tail -5
  ```
  Shows `Finished`, zero errors.

- **Row G** (workspace tests; no regression):
  ```bash
  cargo test --release --workspace --no-fail-fast 2>&1 \
    | grep -E "^test result" \
    | awk '{f += $6} END {print f}'
  ```
  Failures summed ≤ 9 (post-4a-α baseline). Ideally MORE tests in the PASSED set than pre-slice — the legacy callers gain canonical-macro coverage.

- **Row H** (P2b / Layer-2 escalations surfaced):
  SCORE doc has a "P2b sites" section if any. If none surface: SCORE explicitly states "no P2b sites encountered; 32/32 sites migrated mechanically; all P1/P2a/P3."

## Honest deltas to watch for

- **Source-string parametric tests.** Some legacy `:wat::test::run` callers build source strings via `string::format` or similar. Can't trivially become body forms. Most likely 0-3 such sites; flag for case-by-case migration.

- **stdin-driven tests.** If a legacy test ACTUALLY uses the `stdin :Vector<String>` parameter to drive `readln` in the child body, it needs Layer 2 (`run-hermetic-with-io`) — NOT `run-thread` (which can't drive typed input in the body-only Layer 1 shape). Surface; expected 0-2 sites at most.

- **Scope parameter dropouts.** All variants take an optional scope. Modern macros DROP scope; if any test passes `(:wat::core::Some "scope-name")` and DEPENDS on scope semantics (rather than just defaulting), surface — but `wat/kernel/hermetic.wat:106-117` confirms scope was never functional, so this should be a no-op drop.

- **Helper-function wrapping.** Per `feedback_test_file_composition`, some test files may wrap the legacy macro in a helper (`run-test`, `run-and-assert`, etc.). Migrate the helper once; the helper's callers don't need changes. Surface "N call sites covered by 1 helper migration" in SCORE for honesty.

- **Existing `run-hermetic` callers (not `-ast`).** A few sites may ALREADY use `:wat::test::run-hermetic` (Layer 1 macro from arc 170 slice 3 phase C). Don't touch those — they're already canonical. Row D/E counts exclude them from the +N delta.

- **Workspace pressure flake.** Post-4a-α baseline is 9 failures (pressure-flake rotation set; composition varies). The slice's gate is ≤ 9, not exact set. If the failure SET rotates but the count holds, that's clean; if count jumps to 10+, surface the regression class.

- **Per-file site distribution.** Honest reporting in SCORE: which files had how many sites; whether any file had > 5 sites (indicating it's a "test of test infrastructure" file that may need different treatment).

## Workspace baseline (commit `ddb3cad` — yesterday's tip)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2264 passed / 9 failed (Phase-3 pressure-flake rotation set; ≤ 9 is the gate)

Post-slice-4a-β target:
- 2264+ passed (former legacy callers now run via canonical macros — count may be same if it's the same tests via different macros; may be higher if new tests get registered as macro expansion changes shape)
- ≤ 9 failed (no new regressions)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 25–45 min | TBD |
| Scorecard rows | 8/8 PASS | TBD |
| Workspace fail count | ≤ 9 | TBD |
| P2b sites surfaced | 0–5 | TBD |
| Layer-2 escalations | 0–2 | TBD |
| Helper-function consolidations | TBD | TBD |
| Stdin-string parametric sites | 0–3 | TBD |
| Mode | A (clean) | TBD |
