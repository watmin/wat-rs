# Arc 159 — EXPECTATIONS (slice 1)

**Drafted 2026-05-07 by orchestrator before sonnet spawn.**

## Independent prediction

**Predicted runtime:** 25-40 min Mode A. **Time-box:** 60 min wall-
clock.

**Why this estimate:**
- `process_let_binding` extension: ~10-20 LOC (one new branch)
- `parse_let_binding` extension: ~10 LOC (one new branch)
- `step_let` extension: ~10-30 LOC (verify + extend if needed)
- `LegacyTypedLetBinding` variant + Display + diagnostic: ~30 LOC
  (mirror of `BareLegacyLetStar`)
- `walk_for_legacy_typed_let_binding` walker: ~40-60 LOC
  (mirror of `validate_legacy_let_star`)
- 10-13 tests: ~250-400 LOC

Comparable to arc 154 slice 1a (60 min predicted, 28 min actual)
and arc 158a slice 1 (~14 min actual). Arc 159's slice 1 has
more pieces but each is a mirror of well-understood precedent.

**Mode classification:**
- **Mode A**: 10-13 of 10-13 new tests pass; workspace failures
  are exclusively `LegacyTypedLetBinding` on legacy sites
  (expected); no destructure tests broken.
- **Mode B**: 1-2 surprise reds; sonnet patches (e.g., step_let
  needed an unanticipated extension).
- **Mode C**: substrate gap; orchestrator decides.

## Expected scorecard rows

| Row | Expectation | Verification |
|---|---|---|
| **End-to-end new shape (test 1)** | `(let ((x 2)) (+ x 1))` evaluates to 3:i64 | Test runtime check |
| **Tests 2-5 (multi/closure/type/sequential)** | All pass | Test diagnostics |
| **Walker on legacy (tests 6-8)** | `LegacyTypedLetBinding` fires per legacy binding | Test diagnostics |
| **Destructure preservation (tests 9-10) — CRITICAL** | 2-elem and 3-elem destructure work; v1 bug doesn't recur | Test runtime check |
| **Regression (test 11)** | `ScopeDeadlock` fires on new-shape Channel binding | Test diagnostic |
| **Pre-existing destructure tests** | `let_star_destructures_a_pair`, `let_destructure_requires_tuple` STILL PASS | `cargo test --lib destructure` |
| **Workspace failures** | Many `LegacyTypedLetBinding` (expected); zero unexpected | Failure shape sane |
| **3-file constraint** | Edits in `src/check.rs`, `src/runtime.rs`, NEW test file ONLY | `git diff --stat` |
| **NO src/ embedded-wat edits** | Sonnet does NOT touch unit-test embedded wat strings in src/check.rs / src/runtime.rs (v1's scope creep) | `git diff src/check.rs src/runtime.rs` reviewed |
| **Uncommitted state** | Sonnet does NOT commit | `git log --oneline -3` |

## Honest delta candidates

- **`step_let` extension scope.** Sonnet 1a v1 extended `step_let`
  for new shape; arc 159 sonnet should verify by reading the
  current code and extending only if needed. Honest delta if the
  step path needed more work than anticipated.
- **Walker pattern matching depth.** The walker walks INTO let
  bindings (arc 154's walker matched outer keyword; this one
  matches binding shape). Sonnet may surface that the walker
  needs to recurse deeper into nested lets. Mirror of arc 154's
  recipe should handle it.
- **`process_let_binding` interaction with the existing
  `is_typed_single` and destructure branches.** New Symbol-at-[0]
  branch BEFORE the binder-as-List logic — must not affect the
  later branches.
- **Sonnet-scope-creep prevention.** The BRIEF explicitly forbids
  src/ embedded-wat edits. Verify by reading the diff post-spawn.
  If sonnet swept embedded wat, kill + reland.

## SCORE methodology

After slice 1 returns, score per row. Then:
- Mode A clean → proceed to slice 2 (wat-rs consumer sweep)
- Mode B → patch and proceed
- Mode C → orchestrator decides

## Pre-flight checklist

- [x] DESIGN.md current
- [x] BRIEF-SLICE-1.md drafted
- [x] EXPECTATIONS-SLICE-1.md drafted (this commit)
- [x] Workspace baseline 2036 / 0 / 0 verified
- [ ] Commit BRIEF + EXPECTATIONS
- [ ] `model: "sonnet"` set on Agent call (FM 12)
- [ ] `run_in_background: true` set on Agent call
- [ ] ScheduleWakeup at 60 min (3600s) post-spawn
- [ ] Brief explicitly prohibits src/ embedded-wat edits

## Why this slice now

Arc 158a settled the walker dependency. Arc 159 substrate is the
direct path to the user-visible end-state. With v1's lessons
explicitly addressed (destructure preservation tests; sonnet-scope-
creep prohibition), this should ship clean.

The proactive stepping-stones discipline (recovery doc § 5 +
memory `feedback_stepping_stones_proactive.md`) is being applied
correctly: arc 158a was the explicit dependency-fix arc; arc 159
is the user-feature arc atop the settled foundation.
