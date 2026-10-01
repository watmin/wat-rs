# WEIGH — STONE 255.75: every probe runs (E3) — ACCEPTED

**Executor: a Sonnet subagent, commits `5f1bb0c08` (the stone) and `82af2f167` (two corrections).** Weighed by the
orchestrator on 2026-10-01.

## Re-run by the orchestrator at `82af2f167`

| row | result |
|---|---|
| release floor | `.floor/2026-10-01T07-24-44Z`: **6337 passed / 24 skipped**, exit 0; 70 `every_probe_runs::` tests pass |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| the gate picks up a new probe and goes red | a throwaway `wat-scripts/probes/arc-255/zz-weigh-mutation.wat` (raises): `cargo nextest run --release -E 'test(/every_probe_runs/)'` → `71 tests run: 70 passed, 1 failed`, the failure named `probe_wat_scripts_probes_arc_255_zz_weigh_mutation`; file deleted, tree clean |
| probes under the gate | `git ls-files 'wat-scripts/probes/**/*.wat'` → 70 |

## What landed

- **The gate:** `build.rs` generates one test per `wat-scripts/probes/**/*.wat`; each runs the release binary and requires
  exit 0. A probe nothing runs can no longer exist under `probes/`.
- **The twenty:** 7 repaired (the `RecvOutcome`/`SendOutcome.Stopped` class ×5, the `ast-name` class ×3, one overlapping);
  9 negatives moved to `tests/process/fixtures/*.wat` with driven tests naming their errors (plain `.wat`, because they
  fail at run time and `every_wat_bad_fixture_actually_fails` reserves `.wat.bad` for startup failures); 3 retired
  (`m1-cf-norevoke`, `s2s-revoke-probe`, `m1-fix-revoke`: each claim owned deterministically by M1-TEETH); 1 fragment
  (`w3-n-dial-runner`) moved beside a `--check` test.

## Corrections the weigh made (on the record)

- **Cure 4 was rejected.** The agent's first floor went red on `m1_fix_revoke_bounces_dial_2` (`"disconnected"` where the
  test pinned `"recv': peer closed"`); it widened the assertion to accept both, with a mechanism ("the unmatched revoke
  raises under load") the frames do not show. Traced by the orchestrator: the generated feature caller scrubs every
  `RecvOutcome.Lost` cause to `LociDiedError.Disconnected` (`src/runtime.rs` ~3460-3500); a bounce is `drop(stream)` at
  the accept gate (`src/kernel/listener.rs:452`), which the prober reads as a clean EOF (`Closed`) or a reset (`Lost`)
  depending on whether its request bytes arrived first. M1-TEETH already reads `Lost`/`Stopped`/`Closed` all as
  `Outcome.Bounced` (`tests/services/probe_arc170_m1_teeth_revoked.wat:112-117`). The probe was retired.
- **My own error, corrected in the same trace:** I first read `s2s-revoke`'s `"disconnected"` as a failing revoke RPC. It
  was caller2 refused as designed and main seeing `Lost`.
- `generic-shipped` carried two stale `RecvOutcome` errors that its "5 errors" pin counted; repaired, three load-bearing
  errors remain and are asserted by name.

## Verdict

Accepted and pushed. Next by the builder's order: 255.76, assertions for the probes that only print.
