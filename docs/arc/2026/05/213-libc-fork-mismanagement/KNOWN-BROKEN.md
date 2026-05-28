# KNOWN-BROKEN test (surfaced by arc 240, 2026-05-27)

Arc 239 made the workspace test-build compile; the full `cargo test --workspace`
then surfaced a flaky failure in this arc's **lifeline** primitive (the
`spawn_lifelined` / Pidfd + lifeline-pipe PDEATHSIG-replacement, `src/fork.rs:154`
"Arc 213 β"). Per user direction 2026-05-27 it is **process-management-grade** and
belongs to this open arc, not arc 240 (which only does consumer-`.wat` drift +
clean substrate gaps).

**Red test:**
- `probe_lifeline_pipe_proof::lifeline_pipe_zero_orphans_across_100_trials`
  — `tests/probe_lifeline_pipe_proof.rs:214`

**Observed:** failed **1/100 trials** ("lifeline pipe failed in 1/100 trials").
The probe asserts the STRONG claim: 100/100 forks produce zero orphans regardless
of supervisor exit timing — "no signal, no timer, no race" (the lock-step
alternative to `PR_SET_PDEATHSIG`, which had a demonstrated ~10% orphan race in
the fork→prctl window).

**What to investigate when arc 213's pidfd consumer-migration cascade (#373) lands:**
is the 1/100 a *residual race* in the lifeline mechanism (the "cannot race" claim
is then too strong and needs qualification), or an *environmental flake* under the
100-fork storm (resource-limit / scheduler artifact unrelated to the lifeline FD
logic)? Re-run `cargo test --release --test probe_lifeline_pipe_proof` in isolation
several times to characterize before concluding. Do NOT fold into routine gates —
it spawns processes (the very leak class held off the routine path until arc 170/213
settle, per `feedback_green_gate_lib_and_build`).

Cross-ref: `docs/arc/2026/05/240-runtime-rot-remediation/DESIGN.md` (root cause F);
FD-multiplex Phase 1B (#301, "spawn-process lifeline, retire PDEATHSIG").
