# Arc 170 FD-multiplex Phase 1D BRIEF — substrate-mechanism probe + leak-zero gate

**Phase:** 1D of DESIGN-FD-MULTIPLEX-SHUTDOWN.md.
**Predecessors:** Phase 1B (`8714a6f`) — spawn_process lifeline; Phase 1C (`daa411a`) — fork-program lifeline. Both PDEATHSIG paths retired; lifeline mechanism wired through ChildHandleInner.
**Goal:** Demonstrate empirically that the lifeline mechanism delivers the orphan-cleanup property Slice C's PDEATHSIG mechanism was supposed to deliver, AND prove the previously-observed 10% race rate is now zero. Two probes ship:

1. **New substrate-mechanism probe** — `tests/probe_lifeline_orphan_clean_via_substrate.rs`. Mirrors `probe_pdeathsig_kills_orphan_child` shape but routes through the new substrate plumbing (lifeline mechanism). Supervisor wat-vm spawns a blocking grandchild via `:wat::kernel::spawn-process`; supervisor `_exit`s without waiting; grandchild dies within 100ms via lifeline EOF + shutdown cascade.

2. **PDEATHSIG diagnostic re-run** — `probe_pdeathsig_diagnostic` (Slice D's existing A/B probe) should now PASS at delay=0 with 0 orphans (50/50). This proves the race that motivated Phase 1D is closed.

## Context (read before starting)

1. This BRIEF.
2. `tests/probe_pdeathsig_kills_orphan_child.rs` — the original PDEATHSIG-era probe. Read the structure; the new probe is structurally the same but with different assertion semantics (lifeline path, not signal path).
3. `tests/probe_pdeathsig_diagnostic.rs` — Slice D's A/B test. Phase 1D's verification re-uses this with `WAT_PROBE_SUPERVISOR_DELAY_MS=0`.
4. `tests/probe_lifeline_pipe_proof.rs` — pure-libc proof. The new probe is the wat-vm-substrate equivalent (lifeline plumbed via `:wat::kernel::spawn-process`).
5. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md` — empirical baseline: 5/50 (10%) at delay=0 with prctl; 0/50 with sleep.
6. `docs/arc/2026/05/170-program-entry-points/DESIGN-FD-MULTIPLEX-SHUTDOWN.md` — design + Phase 1D's probe sketch.
7. `src/spawn_process.rs:140-220` — eval_kernel_spawn_process (lifeline plumbing — Phase 1B).
8. `src/fork.rs:198` — ChildHandleInner (lifeline_w field).

## Edits

### 1. NEW probe file: `tests/probe_lifeline_orphan_clean_via_substrate.rs`

Same structural shape as `tests/probe_pdeathsig_kills_orphan_child.rs` BUT:

- Routes through `:wat::kernel::spawn-process` (which now creates a substrate-owned lifeline pipe per Phase 1B).
- Supervisor wat-vm forks (raw libc::fork from the test) → supervisor calls eval(spawn-process) → grandchild substrate sets up lifeline (Phase 1B/1C path); supervisor `_exit(0)`.
- Test polls `done_pipe` for POLLHUP with 1000ms budget. Grandchild's substrate cascade should fire within MS via lifeline EOF.
- Assertion: grandchild zombie (Z) or gone (`?`) within 1s — same observable contract as the original probe.

Copy the structure from `tests/probe_pdeathsig_kills_orphan_child.rs` line-for-line; only the comments change to reference Phase 1B/1C / lifeline mechanism instead of PDEATHSIG.

Test name: `probe_lifeline_orphan_clean_via_substrate`. Function header docstring names the mechanism + cross-references Phase 1B/1C SCORE docs.

The blocking-child wat source stays IDENTICAL to the original probe:

```scheme
(:wat::core::defn :test::block-until-shutdown
  []
  -> :wat::core::nil
  (:wat::core::let
    [[tx rx] (:wat::kernel::make-unbounded-channel :wat::core::nil)
     _       (:wat::kernel::recv rx)]
    :wat::core::nil))
```

The cascade trigger differs: pre-Phase-1B, SIGTERM → handler → wake-pipe → worker → trigger_shutdown. Post-Phase-1B, lifeline EOF → worker poll(2) returns POLLHUP → trigger_shutdown. Same `RecvOutcome::Shutdown` outcome; same `(:wat::kernel::recv rx)` unblocks; same clean exit. The test cannot tell the difference at the wat surface — only the substrate mechanism changed.

### 2. PDEATHSIG diagnostic re-baseline (optional update)

`tests/probe_pdeathsig_diagnostic.rs` currently runs the original probe pattern (PDEATHSIG-era). After Phase 1B+1C, that path no longer exists — the call goes through the lifeline mechanism regardless of `WAT_PROBE_SUPERVISOR_DELAY_MS`. The probe still validates "grandchild dies within 1s after supervisor exit" — that property NOW depends on the lifeline mechanism (was PDEATHSIG).

UPDATE the probe's header docstring to reflect the post-Phase-1C reality:
- The mechanism under test is now the lifeline pipe, not PR_SET_PDEATHSIG.
- The diagnostic env var still meaningfully ablates the race (lifeline is structurally race-free; expect 50/50 PASS at delay=0).
- Keep the probe; it's now the leak-zero gate.

No code changes to `probe_pdeathsig_diagnostic.rs` body — only the header docstring + comments.

### 3. Preserve `probe_pdeathsig_kills_orphan_child.rs` as historical regression marker

Per `feedback_inscription_immutable` — the original probe stays unchanged. It still validates the same observable contract; the underlying substrate mechanism just changed. If it passes, the orphan-cleanup property holds.

NO edits to this file.

## Scorecard (8 rows)

| Row | What | Evidence |
|-----|------|----------|
| A | NEW probe file `tests/probe_lifeline_orphan_clean_via_substrate.rs` exists | `ls tests/probe_lifeline_orphan_clean_via_substrate.rs` |
| B | New probe routes through `:wat::kernel::spawn-process` (NOT raw libc fork chain) | `grep -n "spawn-process" tests/probe_lifeline_orphan_clean_via_substrate.rs` shows at least one wat-level spawn-process call |
| C | New probe asserts grandchild zombie/gone within 1s after supervisor `_exit` | `grep -n "POLLHUP\|state.*Z\|kill.*ESRCH\|poll_ret" tests/probe_lifeline_orphan_clean_via_substrate.rs` shows the same poll-based rendezvous pattern as original probe |
| D | `cargo build --release --test probe_lifeline_orphan_clean_via_substrate`: clean | build output |
| E | New probe PASSES 1/1 in isolation | `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` shows `1 passed; 0 failed` |
| F | `probe_pdeathsig_kills_orphan_child` STILL PASSES (historical regression marker) — orphan-cleanup property survived mechanism swap | `cargo test --release --test probe_pdeathsig_kills_orphan_child` shows `1 passed; 0 failed` |
| G | `probe_pdeathsig_diagnostic` with `WAT_PROBE_SUPERVISOR_DELAY_MS=0`: 50/50 PASS (lifeline mechanism is structurally race-free; ablation that produced 5/50 leaks in Phase 1B Slice D is now closed) | shell loop of 50 invocations; count failures |
| H | Header docstring on `probe_pdeathsig_diagnostic.rs` updated to reflect post-Phase-1C mechanism (lifeline, not PDEATHSIG) | grep header or read file |

## Verification methodology for Row G (the leak-zero gate)

Same shell-loop pattern as Slice D's A/B test (from `SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md`):

```bash
BIN=$(ls -t target/release/deps/probe_pdeathsig_diagnostic-* | grep -v '\.d$' | head -1)
PASS=0; FAIL=0
for i in $(seq 1 50); do
  if WAT_PROBE_SUPERVISOR_DELAY_MS=0 "$BIN" --quiet --test probe_pdeathsig_diagnostic 2>/dev/null | grep -q "test result: ok"; then
    PASS=$((PASS+1))
  else
    FAIL=$((FAIL+1))
  fi
done
ORPHANS=$(ps faux | grep probe_pdeathsig_diagnostic | grep -v grep | grep -v bash | wc -l)
echo "delay=0:  pass=$PASS  fail=$FAIL  orphans=$ORPHANS"
# REAP before reporting
ps faux | grep probe_pdeathsig_diagnostic | grep -v grep | grep -v bash | awk '{print $2}' | while read p; do kill -9 "$p" 2>/dev/null; done
```

Pass criterion: PASS = 50, FAIL = 0, orphans = 0. The Slice D baseline at delay=0 was PASS = 45 / FAIL = 5 / orphans = 5 (10% race). Phase 1D's pass: zero across 50 trials.

## Constraints

- NO Mutex / RwLock / CondVar in the new probe.
- NO new wall-clock timers; the new probe uses `libc::poll(2)` with explicit budget (mirror the original probe).
- DO NOT modify `tests/probe_pdeathsig_kills_orphan_child.rs` (per `feedback_inscription_immutable`).
- The new probe must be self-contained — no shared fixtures, no helpers in other test files.
- Per `feedback_no_known_defect_left_unfixed`: if Row G's 50-trial sweep produces ANY orphan, STOP and surface. The lifeline mechanism's empirical 100/100 in `probe_lifeline_pipe_proof` says zero. Substrate residue would be a Phase 1B/1C bug.

## STOP-at-first-red

If you hit:
- `cargo build` fails after new probe added → STOP, report.
- New probe panics (Row E) → STOP. Read panic body; root-cause via /proc snapshot of any stuck procs.
- Row G shows orphan leak at delay=0 → STOP. The lifeline mechanism didn't fully replace PDEATHSIG's semantics. Likely a Phase 1B/1C bug we missed.

## On completion

Write `SCORE-FD-MULTIPLEX-PHASE-1D-LIFELINE-PROBE.md`. 8 rows. Include the Row G sweep result table (PASS/FAIL/orphans across 50 trials). Cross-reference Slice D's empirical record so the before/after delta is on disk.

Do NOT commit. Orchestrator commits atomically after independent verification.
