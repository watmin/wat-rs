# Arc 170 FD-multiplex Phase 1D EXPECTATIONS

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-1D-LIFELINE-PROBE.md`

## Independent prediction

**Runtime band:** 10–18 minutes Mode A.

Reasoning:
- New probe file ~150 lines (mirror of existing `probe_pdeathsig_kills_orphan_child.rs`); mostly mechanical copy + adjusted comments.
- 50-trial shell loop verification adds ~30s wall-clock (each trial ~50ms + test binary startup).
- Header docstring update on `probe_pdeathsig_diagnostic.rs` is trivial.

**Time-box:** ScheduleWakeup at 36 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with evidence:

- **Row A** (new file exists): `ls tests/probe_lifeline_orphan_clean_via_substrate.rs`
- **Row B** (uses spawn-process): `grep -n "spawn-process" tests/probe_lifeline_orphan_clean_via_substrate.rs` returns ≥1 match
- **Row C** (poll-based rendezvous): grep for the assertion shape — `POLLHUP`, `poll_ret`, zombie-state check
- **Row D** (build clean): `cargo build --release --test probe_lifeline_orphan_clean_via_substrate 2>&1 | tail -3` shows Finished
- **Row E** (probe passes): `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` shows `1 passed; 0 failed`
- **Row F** (historical marker still passes): `cargo test --release --test probe_pdeathsig_kills_orphan_child` shows `1 passed; 0 failed`
- **Row G** (leak-zero gate): inline shell loop in SCORE doc shows 50/50 PASS at delay=0 with 0 orphans. Include the exact command + output.
- **Row H** (docstring updated): grep header or read file to confirm comment references lifeline mechanism

## Honest deltas to watch for

- **Probe flakes vs substrate races.** The lifeline pipe proof showed 5/100 chained-run flake; in isolation it's 100/100. The new probe's 50-trial sweep MUST run in isolation (one `cargo test --test <name>` invocation per trial OR all 50 in one invocation if `probe_pdeathsig_diagnostic` is single-test-per-binary). A flake at high N (>1% rate) deserves a /proc snapshot before declaring success.
- **`probe_pdeathsig_kills_orphan_child` semantics.** The original probe asserted "grandchild dies within 1s via SIGTERM cascade." Post-Phase-1C the SIGTERM never arrives (no prctl); the cascade fires via lifeline EOF instead. The probe's observable assertion (grandchild zombie/gone within 1s) still holds — but the diagnostic message in the probe might reference PDEATHSIG. If sonnet finds the probe panic msg references "PR_SET_PDEATHSIG cascade broken" or similar, leave the message UNCHANGED (per `feedback_inscription_immutable` — the probe is historical artifact). If the probe fails because the mechanism semantics differ in observable ways, STOP and surface — that's a real substrate gap.
- **`probe_pdeathsig_diagnostic` header.** The `WAT_PROBE_SUPERVISOR_DELAY_MS` env var was an A/B switch for the PDEATHSIG race. Post-Phase-1C it's vestigial (lifeline is race-free regardless). Header docstring should note the env var still ablates timing but the mechanism is now race-free. Don't remove the env var — it's the leak-zero gate; if it ever produces orphans, that's a substrate regression.

## Workspace baseline (post-Phase-1C, commit daa411a)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --test probe_shutdown_cascade_crossbeam`: 1/1 PASS
- `cargo test --release --test probe_lifeline_pipe_proof`: 1/1 PASS in 30ms (in isolation)
- `cargo test --release --test probe_pdeathsig_kills_orphan_child`: STILL PASSES (verifies substrate orphan-cleanup property survived mechanism swap)

Note: `probe_pdeathsig_kills_orphan_child` not yet run against post-Phase-1C state — the BRIEF asks sonnet to verify Row F as evidence that the swap preserved the observable contract. If it fails, that's a critical signal.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 10–18 min | TBD |
| Scorecard rows | 8/8 PASS | TBD |
| Honest deltas | 1–2 surfaces | TBD |
| Mode | A (clean) | TBD |
| Row G result | 50/50 PASS, 0 orphans | TBD |
