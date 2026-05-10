# Arc 170 slice 3 — EXPECTATIONS

> Atomic-commit pair: phase A (opus, testing-lib platform) +
> phase B (sonnet, mechanical sweep). Bundled commit when
> workspace = 0-failed.

## Independent prediction

**Phase A predicted: 90-180 min opus.**
**Phase B predicted: 60-180 min sonnet.**
**Total: 150-360 min between two agents + orchestrator
atomic-commit ceremony.**

Phase A reasoning:
- run-hermetic + run-hermetic-with-io macros: NEW (medium)
- deftest macro rebuild: judgment-heavy (depends on shape choice)
- hermetic.wat rebuild or retirement: medium
- sandbox.wat migration: medium
- Comparable to slice 1c's substrate work (~90 min)

Phase B reasoning:
- ~277 test fixture migrations
- Mechanical patterns (3-arg main → 4-arg, fork-program → spawn-process,
  spawn-program → spawn-process|spawn-thread)
- Sonnet velocity on mechanical sweep is high; arc 168 slice 2
  was 951 sites in ~90 min sonnet → ~277 sites is ~30-90 min
  optimistic, ~180 min if ambiguities surface

**Hard cap: 360 min phase A + 360 min phase B = 720 min total.**
If either phase overruns, in-flight check.

## Scorecard

All rows must pass. Phase A and B feed into bundled commit.

### Phase A rows

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| A.1 — `run-hermetic` macro minted | wat/test.wat (or wat/test/run-hermetic.wat) defines `:wat::test::run-hermetic` macro that wraps body in fn + spawns hermetically + returns RunResult | ✓ |
| A.2 — `run-hermetic-with-io` macro minted | Layer 2 macro introduces rx + tx as bindings in body scope; spawns; drains; returns parsed Vec\<O\> + failure | ✓ |
| A.3 — deftest rebuild | `wat/test.wat`'s deftest macro emits new contract shape (option (a) or (b)); deftest-hermetic + make-deftest + make-deftest-hermetic same treatment; choice + reasoning surfaced | ✓ |
| A.4 — hermetic.wat rebuilt or retired | `wat/std/hermetic.wat` either rebuilt on typed-channel API OR retired entirely (subsumed by run-hermetic macros); choice + reasoning surfaced | ✓ |
| A.5 — sandbox.wat migrated | `wat/std/sandbox.wat` no longer uses spawn-program* internally; migrated to spawn-process(fn) or spawn-thread(fn); choice + reasoning surfaced | ✓ |
| A.6 — Phase A workspace state | post-phase-A: 268 deftest_* failures GREEN (deftest emission matches new contract); 277 other failures still RED; verify via cargo-test-summary.sh | ✓ |
| A.7 — Phase A no commit | dirty tree; phase A's work uncommitted; ready for phase B against dirty tree | ✓ |

### Phase B rows

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| B.1 — 3-arg main migrations | all test fixtures with legacy 3-arg `:user::main` migrated to 4-arg + ExitCode shape | ✓ |
| B.2 — fork-program* migrations | all user-source fork-program/fork-program-ast callsites migrated to spawn-process(fn) | ✓ |
| B.3 — spawn-program* migrations | all user-source spawn-program/spawn-program-ast callsites migrated per two-mode taxonomy (spawn-process or spawn-thread; reasoning surfaced per ambiguous case) | ✓ |
| B.4 — Lib test + arc112 probes | `runtime::tests::assert_eq_failure_renders_actual_and_expected` migrated; 2 arc112 probes migrated or removed | ✓ |
| B.5 — Phase B workspace state | post-phase-B: 2122 passed 0 failed (1594 base + 277 swept + 268 deftest_* from phase A + 15 arc170 contract tests; verify exact count) | ✓ |
| B.6 — Phase B no commit | dirty tree; phase B's work uncommitted; ready for orchestrator atomic commit | ✓ |

### Atomic commit + bundled rows

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| C.1 — Workspace 0-failed | `./scripts/cargo-test-summary.sh` shows passed: N failed: 0 | ✓ |
| C.2 — Single atomic commit | orchestrator commits both phases as ONE commit; commit message names both phases + their work | ✓ |
| C.3 — Branch on remote | `arc-170-program-entry-points` carries the atomic commit; main untouched | ✓ |
| C.4 — Zero Mutex usage | no Mutex/RwLock/CondVar introduced (zero-mutex doctrine) | ✓ |
| C.5 — SCOREs 1, 1B, 1C, 2 untouched | immutable per `feedback_inscription_immutable.md`; verified | ✓ |
| C.6 — Slice 1b/1c/2 APIs unchanged | extract_closure + ClosurePackage shape; typed_channel module; spawn-process verb; :user::main signature; walkers — all stable | ✓ |
| C.7 — Process additive bandaid still in place | slice 4 retires; verify no premature legacy-field removal in slice 3 | ✓ |
| C.8 — DESIGN-intent alignment | testing-lib reaches polished form per TIERS.md three-layer API; Layer 1 macro hides ALL fn ceremony; Layer 2 macro introduces typed-channel bindings; Layer 3 is substrate primitive (no testing wrapper) | ✓ |

## Honest-delta categories (across both phases)

- **Phase A: deftest expansion shape (a) vs (b)** — agent picks
- **Phase A: hermetic.wat rebuild vs retirement** — agent picks
- **Phase A: sandbox.wat migration shape** — agent picks
- **Phase B: ambiguous migration shapes** — surface; orchestrator
  decides
- **Slice 1b honest delta B (match-arm pattern bindings)** — if
  rebuilding deftest body trips this, surface
- **deftest's prelude/load-mechanism integration** with run-hermetic
- **FM 5 trap** — TODOs verboten

## Calibration row

Phase A actual: ___ minutes (Mode A/B/C).
Phase B actual: ___ minutes (Mode A/B/C).
Total: ___ minutes (predicted 150-360).

deftest expansion choice: (a) routes through run-hermetic / (b) emits 4-arg main directly.
hermetic.wat disposition: rebuilt / retired.
sandbox.wat migration: spawn-process / spawn-thread / hybrid.

Workspace post-phase-A: ___ passed ___ failed.
Workspace post-phase-B: ___ passed ___ failed.

Honest deltas surfaced: ___ phase A + ___ phase B.

## What's next (orchestrator-side, post-slice-3)

When slice 3 atomic-commits to green:
- SCORE-SLICE-3.md authored + committed (documents both phases +
  the atomic-commit pattern + calibration data)
- Slice 4 BRIEF + EXPECTATIONS authored — bandaid retirement
  pair: opus destructively retires Process<I,O> legacy 3 fields
  + walker bodies + legacy dispatch arms; sonnet sweeps any
  residual; orchestrator atomically commits.
- Slice 5 paperwork authored after slice 4 lands.

## SCORE artifact

After both phases atomic-commit to green, orchestrator writes
SCORE-SLICE-3.md as one document covering both phases. Slice 3
agents (phase A opus + phase B sonnet) report individually to
chat; orchestrator synthesizes both into the SCORE.
