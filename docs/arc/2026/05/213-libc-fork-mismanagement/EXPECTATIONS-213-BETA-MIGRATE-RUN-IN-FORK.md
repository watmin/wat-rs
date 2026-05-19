# Arc 213 stone β — EXPECTATIONS

## Independent prediction

- **Runtime band:** 20-40 min Mode A. Single-function refactor using α's just-shipped primitive. Caller verification across 5 test binaries is the bulk of the work.
- **LOC changed:** ~30-40 (run_in_fork body shrinks; calling spawn_lifelined + Pidfd::wait_status is more concise than bare fork+waitpid)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** LOW — α's smoke probe proved spawn_lifelined works end-to-end (normal exit + signal exit); β is mechanical migration. Possible plumbing surprise: a caller asserts on the EXACT panic message string from the old "waitpid failed" / "forked child exited with failure" assertions; β's wait_status path may emit different prose. If any test asserts panic-message-string, that's an honest delta worth surfacing.

## Honest-delta watch

LOW-risk migration. Three risk surfaces:

1. **Panic message string changes.** Old: `"forked child exited with failure (status={:#x})"`. New: `"forked child exited with failure: {:?}"` (Debug-format ExitStatus). If any test asserts on the EXACT string via `#[should_panic(expected = "...")]` or substring match, it'll fail post-migration. Cheap to fix (update assertion to new prose); should be flagged in SCORE.

2. **Lifeline FD count.** Each `run_in_fork` call now opens 2 FDs (the lifeline pipe pair). Tests in tight loops or tests that count open FDs might surface this. Should not affect any of the 5 known caller binaries (none of them stress-test FD counts).

3. **Process group change.** spawn_lifelined calls `setpgid(0, 0)` in child — child becomes its own process group leader. Old behavior: child inherits parent's process group. If any test sends signals via `kill(0, sig)` (kill the whole process group), the child no longer receives them. Highly unlikely any of the 5 known callers do this, but worth noting.

If any of these surface in test failures, document in SCORE rather than masking with a workaround.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `run_in_fork` body migrated to spawn_lifelined + Pidfd::wait_status | YES |
| 2 | Public signature unchanged (`F: FnOnce() + UnwindSafe`) | YES |
| 3 | `libc::fork()` call removed from run_in_fork | YES |
| 4 | `libc::waitpid()` call removed from run_in_fork | YES |
| 5 | Manual WIFEXITED/WEXITSTATUS decoding removed from run_in_fork | YES |
| 6 | cargo build --release clean (5 pre-existing warnings unchanged) | YES |
| 7 | Smoke probe `probe_pidfd_primitive` still passes 2/2 (α regression check) | YES |
| 8 | `wat_harness_deps` 3 run_in_fork sites still pass | YES |
| 9 | `probe_shutdown_cascade_crossbeam` still passes | YES |
| 10 | `probe_shutdown_cascade_pipefd` still passes | YES |
| 11 | `wat-cli wat_cli` test using run_in_fork still passes | YES |
| 12 | 5 runtime.rs lib tests using run_in_fork still pass | YES |
| 13 | Zero modifications to ANY caller site | YES |
| 14 | Zero changes outside src/fork.rs | YES |
| 15 | SCORE inscribes any panic-message-string deltas + caller verification | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; substrate fork-path inconsistency closed; β complete
- **Mode B (acceptable):** a test fails because it asserted on the exact panic message string OR a stress-test surfaces FD-count interaction; sonnet describes the delta + REVERTS + returns. Orchestrator decides whether to: (i) update the test assertion to new prose (small adjacent commit), or (ii) keep old panic format in run_in_fork's wait_status assertion (preserve exact string)
- **Mode C:** STOP rule broken (touched γ sites, changed signature, modified caller sites, added lifeline_r polling)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (α's primitive is verified; migration is mechanical; signature stays). MEDIUM-HIGH on first-attempt Mode A (the panic-string risk is real-but-bounded; if any test asserts on it, that surfaces honestly and is one-line-fix territory).
- **Risk factors:**
  - Panic-message-string assertion in ANY of the 11 caller tests (mitigation: SCORE-described + caller-update small commit)
  - One of the 5 runtime.rs lib tests has subtle process-group dependence (mitigation: setpgid(0,0) is honest substrate behavior; if a test depends on inheriting parent's pgroup, that test was relying on substrate-inconsistent behavior and the migration surfaces it)
- **Why this matters:** validates α's primitive in real production usage beyond smoke probes. Closes the substrate's "every fork has a lifeline" promise. The 5 stones that follow (γ migrate 3 libc::fork sites; δ migrate waitpid/kill callers; ε migrate /proc reads; ζ L2 module-privacy enforcement; η INSCRIPTION) build on the per-stone trust gate β proves.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

After α shipped, the next-move tiebreaker on β vs arc 212 ζ-1 came out NEUTRAL on direct precedent-laying (neither lays foundation the other needs). Secondary tiebreakers selected β:
- β is bounded + commits clean standalone (single concern, ~30 min)
- ζ-1 starts a multi-hour campaign that doesn't commit clean until ζ-7 (atomic-commit pair per `feedback_no_broken_commits`)
- β validates α's primitive in production usage (close the loop on α before starting wide campaign)
- β closes a known production-relevant gap (substrate's "every fork has a lifeline" inconsistency)
- Foundation-impeccable discipline: close known defects before starting new campaigns

Per-stone trust gate: β SCORE returns → orchestrator verifies → atomic commit → re-run tiebreaker on γ vs ζ-1 with new data.

## Cross-references

- Arc 213 DESIGN — the full stone chain α/β/γ/δ/ε/ζ/η
- Arc 213 α SCORE (`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-ALPHA-MINT-PIDFD-PRIMITIVE.md`) — the primitive β rebases on
- `src/fork.rs:149-185` — the migration target (run_in_fork)
- `src/runtime.rs:23502-23568` — 5 run_in_fork lib-test sites
- `crates/wat-cli/tests/wat_cli.rs:465` — wat-cli test caller
- `tests/probe_shutdown_cascade_crossbeam.rs:55` — shutdown cascade probe
- `tests/probe_shutdown_cascade_pipefd.rs:59` — pipefd shutdown probe
- `tests/wat_harness_deps.rs:53/83/105` — harness deps test sites
- `feedback_tractability_tiebreaker` — the sequencing discipline that selected β over ζ
- `feedback_substrate_owns_not_callers_match` — the doctrine β extends (substrate fork-path consistency)
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — the architectural commitment β operationalizes
