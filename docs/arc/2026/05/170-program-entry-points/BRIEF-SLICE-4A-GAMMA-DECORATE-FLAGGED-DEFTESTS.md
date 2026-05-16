# Arc 170 Slice 4a-γ-decorate BRIEF — decorate 5 flagged deftests + rearchitect site 154 + mark 188/207 as duplicates

**Task:** #318
**Phase:** Slice 4a-γ second sub-stone (audit ✓ → decorate → flip).
**Predecessors:** Audit shipped at `f2e78ea`. Worklist: 5 sites in `wat-tests/test.wat` (lines 132, 146, 154, 188, 207). All flagged for hermetic semantics. The audit + the user's architectural assessment together determine the per-site action below.

## Goal

Apply the audit's worklist with the orchestrator's architectural refinement: decorate all 5; rearchitect site 154 to honestly exercise stderr-matching machinery; mark sites 188/207 as duplicates of site 132. After this slice, the deftest macro flip (#314) is safe — every hermetic-required deftest is `:wat::test::deftest-hermetic`; every remaining `:wat::test::deftest` site is safe for the post-flip thread default.

## Per-site actions

| Site | Action | Detail |
|---|---|---|
| `wat-tests/test.wat:132` (`test-assert-stdout-is-matches`) | Decorate | `:wat::test::deftest` → `:wat::test::deftest-hermetic`; no other change |
| `wat-tests/test.wat:146` (`test-assert-stderr-matches-pass`) | Decorate | same shape |
| `wat-tests/test.wat:154` (`test-assert-stderr-matches-fail-reports-pattern`) | Decorate + rearchitect | See § "Site 154 rearchitecture" below |
| `wat-tests/test.wat:188` (`test-run-string-entry-path`) | Decorate + add duplicate-marker comment | See § "Site 188/207 duplicate markers" below |
| `wat-tests/test.wat:207` (`test-run-ast-via-program`) | Decorate + add duplicate-marker comment | same shape |

### Site 154 rearchitecture (architectural correctness)

**Current shape (lines 154-167):**

```scheme
(:wat::test::deftest :wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern
  ()
  ;; rune:complectens(embedded-program) — outer let has 2 bindings (r, fail); bulk is a TWO-level nested embedded-program AST literal (fixture)
  ;; ... legacy commentary ...
  (:wat::core::let
    [r
      (:wat::test::run-thread
        (:wat::core::let
          [silent
            (:wat::test::run-thread
              ())]
          (:wat::test::assert-stderr-matches silent "my-pattern")))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail
      ((:wat::core::Some f)
        (:wat::core::let
          [expected (:wat::kernel::Failure/expected f)]
          (:wat::core::match expected
            ((:wat::core::Some e) (:wat::test::assert-eq e "my-pattern"))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "expected slot empty" :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None"
               :wat::core::None :wat::core::None)))))
```

**Problem:** the test claims to verify `assert-stderr-matches`'s failure-reporting shape on no-match, but exercises the EMPTY-INPUT EDGE CASE (silent's stderr is empty). The pattern-matching loop never runs against actual content; the assertion fires because empty Vec has zero candidates. Strict semantically the test passes for the wrong reason.

**Rearchitected shape:**

```scheme
(:wat::test::deftest-hermetic :wat-tests::std::test::test-assert-stderr-matches-fail-reports-pattern
  ()
  ;; Verifies assert-stderr-matches's failure-reporting shape on REAL non-matching stderr.
  ;; Inner produces actual stderr content that doesn't match the pattern; the matcher
  ;; loop runs against that content and fires; the failure carries `expected = "my-pattern"`
  ;; and `actual = (Vec ... captured stderr lines ...)`.
  ;;
  ;; Architectural change (arc 170 slice 4a-γ-decorate): inner spawn was previously
  ;; :wat::test::run-thread with empty body — the test passed via empty-input edge case
  ;; without exercising the pattern-matching machinery. The rearchitecture uses
  ;; :wat::test::run-hermetic with a non-matching stderr line so the matcher loop
  ;; actually runs.
  (:wat::core::let
    [r
      (:wat::test::run-hermetic
        (:wat::core::let
          [silent
            (:wat::test::run-hermetic
              (:wat::kernel::eprintln "different content"))]
          (:wat::test::assert-stderr-matches silent "my-pattern")))
     fail (:wat::kernel::RunResult/failure r)]
    (:wat::core::match fail -> :wat::core::nil
      ((:wat::core::Some f)
        (:wat::core::let
          [expected (:wat::kernel::Failure/expected f)]
          (:wat::core::match expected -> :wat::core::nil
            ((:wat::core::Some e) (:wat::test::assert-eq e "my-pattern"))
            (:wat::core::None (:wat::kernel::assertion-failed!
                     "expected slot empty" :wat::core::None :wat::core::None)))))
      (:wat::core::None (:wat::kernel::assertion-failed!
               "expected Failure, got :None"
               :wat::core::None :wat::core::None)))))
```

**Changes:**
1. `:wat::test::deftest` → `:wat::test::deftest-hermetic`
2. Outer `(:wat::test::run-thread ...)` → `(:wat::test::run-hermetic ...)`
3. Inner `(:wat::test::run-thread ())` → `(:wat::test::run-hermetic (:wat::kernel::eprintln "different content"))`
4. Replace the legacy/migration commentary (`rune:complectens(embedded-program)` lines + nested-sandbox prose) with a clean rationale comment naming the rearchitecture.

The match arms + assertions in the outer scope are UNCHANGED — they still verify `failure.expected = "my-pattern"`. The test still PASSES (the assertion still fires on no-match; the failure-shape carries the pattern as before). The test is now HONEST about its mechanism.

If the rearchitecture surfaces a regression (e.g., the failure shape changes when actual content is captured vs empty), STOP and surface — that's a substrate finding, not a slice failure.

### Site 188/207 duplicate markers (preserve per accumulate-tests policy)

Sites 188 and 207 are post-4a-β-migration duplicates of site 132 (same pattern: `run-hermetic` with `println` body; `assert-stdout-is` against expected Vec). The user's policy: accumulate tests; defer cleanup to post-109 (when coverage tooling lands).

For each, prepend a comment block above the `(:wat::test::deftest ...)` form noting the duplication:

```scheme
;; Duplicate of :wat-tests::std::test::test-assert-stdout-is-matches at line 132 —
;; same hermetic-print-and-capture pattern with different fixture string. Preserved
;; per accumulate-tests-defer-cleanup policy (test cleanup is post-109; coverage
;; tooling needed to verify safe deletion). Original test purpose
;; ("test the legacy <STRING-entry / AST-via-program> path") retired during
;; arc 170 slice 4a-β when the legacy :wat::test::run / run-ast paths
;; were swept to canonical macros.
(:wat::test::deftest-hermetic :wat-tests::std::test::test-run-string-entry-path
  ...)
```

Adapt the comment text per site (188 says "STRING-entry"; 207 says "AST-via-program"). The deftest itself decorates to `deftest-hermetic` (mechanical rename).

## Substrate edits — NONE

No changes to `src/`, `wat/test.wat` macros, `wat/kernel/`, or any non-flagged test file. Pure call-site changes within the 5 flagged sites in `wat-tests/test.wat`.

## Constraints (HARD)

- **Operate ONLY in `/home/watmin/work/holon/wat-rs/`** per `feedback_no_worktrees` + FM 7-bis. `pwd` first; reject `.claude/worktrees/` paths; use `git -C /home/watmin/work/holon/wat-rs` and absolute paths.
- DO NOT commit. Orchestrator commits atomically after independent verification.
- DO NOT touch any test file other than `wat-tests/test.wat`.
- DO NOT modify the deftest / deftest-hermetic macro definitions in `wat/test.wat` (lines 294, 326). Those flip in #314.
- DO NOT modify the run-thread / run-hermetic / run-thread-driver / failure-from-thread-died / run-hermetic-driver families in `wat/test.wat`. They're substrate; this slice changes consumers only.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / the recovery doc / INTERSTITIAL / this BRIEF / EXPECTATIONS / the audit doc.
- Per `feedback_inscription_immutable`: the audit doc at `f2e78ea` STAYS as-is. The orchestrator's architectural assessment (this BRIEF's rationale) is the working artifact for #318; the audit doc is the inscribed snapshot of "what the lexical scan found."

## Scorecard (6 rows, YES/NO with grep evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | Sites 132, 146, 154, 188, 207 in `wat-tests/test.wat` use `:wat::test::deftest-hermetic` | `grep -nE "^\(:wat::test::deftest-hermetic\b" wat-tests/test.wat` shows 5 hits at the migrated lines (line numbers may shift slightly due to comment additions) |
| B | Site 154's body uses `:wat::test::run-hermetic` (twice — outer + inner) | `grep -nA 5 "test-assert-stderr-matches-fail-reports-pattern" wat-tests/test.wat` confirms two `run-hermetic` calls in the body; zero `run-thread` |
| C | Site 154's inner body produces `(:wat::kernel::eprintln "different content")` | grep confirms the eprintln presence inside the inner run-hermetic |
| D | Sites 188 and 207 carry duplicate-marker comments | `grep -B 6 "test-run-string-entry-path\|test-run-ast-via-program" wat-tests/test.wat` shows the prepended comment block referencing line 132 |
| E | `cargo build --release --workspace --tests` clean | build output shows `Finished`, no errors |
| F | All 5 affected tests PASS; workspace failure count ≤ post-4a-β baseline (10 failures, with 8-11 rotation band) | `cargo test --release --workspace --no-fail-fast` shows the 5 deftests in PASSED set; total failed ≤ 11 |

## STOP-at-first-red

- `cargo build` fails after any per-site edit → STOP at the breaking site; surface.
- Site 154's rearchitecture causes the test to FAIL (failure shape changed under hermetic actual-content) → STOP; surface the actual vs expected. May indicate a substrate finding worth investigating before completing the slice.
- Workspace test failure count REGRESSES (>11) → STOP; surface regression class.
- Encounter any unexpected structural pattern in `wat-tests/test.wat` not covered by the 5 sites → STOP; surface; do not improvise.

## Implementation protocol (test-first)

The decoration is mechanical; the rearchitecture has a test-first dimension:

1. **Verify cwd** + branch + tip.
2. **Decorate sites 132, 146, 188, 207 first** (mechanical 4 renames + duplicate-marker comments on 188/207). Build + test after each batch.
3. **Site 154 rearchitecture:** apply the body change BEFORE decorating the deftest. Build → test the rearchitected body still passes (failure-shape assertion still holds against actual non-matching content). Then decorate the deftest.
4. **Final verification:** full workspace build + test pass; all 5 decorated tests in PASSED set.

If site 154's rearchitecture surfaces a substrate-side surprise (e.g., the Failure struct's `actual` slot now populates and changes behavior), STOP — that's information for the orchestrator, not a slice failure.

## On completion

Write `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-4A-GAMMA-DECORATE-FLAGGED-DEFTESTS.md`. 6 rows YES/NO with grep evidence. Honest deltas — especially:

- Whether site 154's rearchitecture revealed any substrate-side findings
- Whether the duplicate-marker comments revealed any other duplicates worth flagging
- Whether the workspace failure-set rotated (no new entries) or stable
- Calibration record per EXPECTATIONS template

Do NOT commit.

## Followup notes (for the orchestrator's record, not this slice's scope)

- After #318 ships, #314 (4a-γ-flip — change deftest macro body to run-thread) is unblocked. The audit + decoration together prove every hermetic-required deftest is decorated; the flip is mechanically safe.
- Post-109 cleanup pass (when coverage tooling exists) should review the duplicate-marker comments for safe deletion candidates. Sites 188/207 are the obvious starters; the comments cite site 132 as canonical.
- Site 154's rearchitecture also documented for the post-109 review — if `actual` slot inspection becomes useful in future test patterns, this site is the canonical example.
