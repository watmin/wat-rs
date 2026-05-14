> **⚠ SUPERSEDED 2026-05-14** — Companion to the wrong-direction BRIEF at `BRIEF-SLICE-4A-LEGACY-TEST-RUN-SWEEP.md` (SUPERSEDED, same date). See that file's prologue + `INTERSTITIAL-REALIZATIONS.md` § 2026-05-14 for the rescope rationale. Preserved as failure-engineering artifact. The current slice plan lives in `BRIEF-SLICE-4A-ALPHA-MINT-RUN-THREAD.md` + chain successors.

---

# Arc 170 Slice 4a EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-4A-LEGACY-TEST-RUN-SWEEP.md`

## Independent prediction

**Runtime band:** 25–45 minutes Mode A.

Reasoning:
- 32 active call sites distributed across wat-tests/ + tests/ + crates/ + examples/.
- Most should follow P1 / P2a (literal source-string or literal forms-vector) which is mechanical: replace the call form with `(:wat::test::run-hermetic <inlined body>)`.
- P2b (computed-forms callers) may need case-by-case judgment — expected 0-5 sites; mid-sweep STOP threshold.
- Each migration is small (~5-line edit at the call site); 32 of them is the bulk.
- Workspace test rerun adds ~3 min.

**Time-box:** ScheduleWakeup at 90 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with evidence per `feedback_four_questions_yes_no`:

- **Row A** (no `:wat::test::run` callers): `grep -rE ":wat::test::run[^-A-Za-z]" wat-tests/ tests/ crates/ examples/ | grep -v "^Binary" | grep -v "//" | grep -v "\.md:" | wc -l` returns 0.
- **Row B** (no `run-ast`): similar grep with `run-ast` returns 0.
- **Row C** (no `run-hermetic-ast`): similar grep with `run-hermetic-ast` returns 0.
- **Row D** (new sites use Layer 1 macro): grep shows the migrated sites contain `:wat::test::run-hermetic` in their body — note `run-hermetic` is also valid in deftest expansions, so this row's check is about INCREASE (delta from baseline, e.g., +20-30 hits for `run-hermetic` across the migrated files).
- **Row E** (build clean): `cargo build --release --workspace --tests 2>&1 | tail -5` shows Finished, zero errors.
- **Row F** (failure count): workspace test failure set ≤ 11 (post-Phase-3 baseline). Count via `cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "^test result" | awk '{ f += $6 } END { print f }'`.
- **Row G** (honest deltas surfaced): SCORE doc has a "P2b sites" or "Layer 2 promotions" or "tests deleted" section if any. If none surface, row says "no non-trivial migrations; 32/32 mechanical."
- **Row H** (legacy wrappers untouched): `grep -nE "wat::test::(run|run-ast|run-hermetic-ast)\b" wat/test.wat` shows the legacy DEFINES still present at the expected line numbers.

## Honest deltas to watch for

- **Source-string parametric tests.** Some legacy tests build source strings via `string::format` or similar. These can't trivially become body forms. STOP and surface — likely 0-3 such sites.
- **stdin-driven tests.** If a legacy test actually uses the `stdin :Vector<String>` parameter to drive readln in the child, it needs Layer 2 (`run-hermetic-with-io`) migration. Surface these for case-by-case judgment.
- **scope :Option<String> parameter.** All variants take an optional scope. The modern macro DROPS scope entirely (per DESIGN.md: "leaked substrate plumbing; today's hermetic.wat errors on `:Some`; not functional anyway"). If a test passes `(:wat::core::Some "scope-name")` — DROP it; no replacement needed.
- **Existing run-hermetic callers.** A few sites may ALREADY use `:wat::test::run-hermetic` (Layer 1 macro). Don't touch those — they're already canonical.

## Workspace baseline (post-Phase-3 + amendment, commit bed1a71)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2260 passed / 11 failed (composition varies under workspace pressure; 9 pre-existing + 2 pressure-rotation; see Phase 3 SCORE for breakdown)

Post-slice-4a target:
- 2260+ passed (former legacy callers now run via run-hermetic; may be same count if tests just got migrated, may be higher if more get registered)
- ≤ 11 failed (no regressions from migration)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 25–45 min | TBD |
| Scorecard rows | 8/8 PASS | TBD |
| Workspace fail count | ≤ 11 | TBD |
| P2b sites surfaced | 0–5 | TBD |
| Layer 2 promotions | 0–3 | TBD |
| Mode | A (clean) | TBD |
