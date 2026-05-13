# Arc 170 — Deadlock detection investigation (Opus mission)

**Opus.** Active failure state in this branch. Mission: find the deadlock, design + implement a detection mechanism, prove it works by causing a PANIC where we previously hung.

## Critical context — this branch is the live failure

You are on branch **`arc-170-gap-j-v5-deadlock-state`**. This branch contains:
- `c3f2bf7` — sonnet's substrate splice fix for `register_types` (correct work; no threading additions)
- `8e07626` — V5 retry deftest macro rewrite (the trigger that exposes a latent deadlock)

Running `cargo test --release --workspace --no-fail-fast` on this branch **deadlocks** — wat-test binaries hang with both threads in `futex_do_wait`, no child process spawned. A live orphan specimen exists (PID 234823 at the time of BRIEF writing, may or may not still be alive when you start).

**The substrate has multiple deadlock-detection arcs already** (arc 117 scope-deadlock compile-time rule; arc 126 channel-pair prevention; arc 131 HandlePool counts as Sender-bearing; arc 132 default 200ms deftest time-limit). The fact that this deadlock SLIPPED PAST all of them means we have a NEW deadlock class we haven't named.

## Your mission

**Find this deadlock. Trace it. Grok the pattern. Design a detection mechanism. Implement it. Verify it by causing a PANIC where the workspace currently hangs.**

User direction 2026-05-14:
> *"we need another [deadlock detection]... when we write enough fix to panic on the deadlock that's present we know we found it"*
>
> *"i think opus should work on the deadlocking state - its in an active failure state - when we write enough fix to panic on the deadlock that's present we know we found it"*

## Operational shape

### Phase 1 — Reproduce + investigate the deadlock (30-60 min)

You are on the branch where it hangs. Reproduce it:

```bash
timeout 60 cargo test --release --workspace --no-fail-fast 2>&1 | tail -30
```

Expect a TIMEOUT (kills the cargo, leaves wat-test orphans). DO NOT wait longer than 60s on any single workspace run. If timeout fires, that's evidence — capture which test(s) hung.

Investigation options (pick what fits):
- **Live process inspection** — if the orphan (PID 234823) or new wat-test orphans are still alive, attach gdb:
  ```bash
  gdb -p <pid> -batch -ex "thread apply all bt" 2>&1 | head -200
  ```
  Both threads in `futex_do_wait`; identify what each is blocked on (mutex address, condvar address, parking_lot signature).
- **`/proc/<pid>/task/*/stack`** — kernel stack traces per thread.
- **`/proc/<pid>/syscall`** — current syscall.
- **Code reading** — trace the substrate code paths for `run-hermetic` → `Process/join-result` → `drain-lines` → `spawn-process` → `eval_kernel_spawn_process` (likely src/runtime.rs + src/spawn.rs + wat/test.wat run-hermetic-driver).
- **Minimal-repro probe** — narrow down which deftest body triggers the deadlock; build a smaller probe that exhibits it deterministically in `<10s`.

### Phase 2 — Identify the pattern (15-30 min)

Once you've located the futex pair, identify the SHAPE of the deadlock. Compare against existing detection arcs:
- **arc 117 (scope-deadlock):** sender alive in same scope as receiver awaiting it → compile-time rule
- **arc 126 (channel-pair):** receiver + sender both used in ways that mutually block
- **arc 131 (HandlePool):** HandlePool counts as Sender-bearing for the scope-deadlock check
- **arc 132 (deftest time-limit):** 200ms default per deftest

Question: is this deadlock a NEW class, or is it an existing class slipping through because some predicate isn't catching it?

Candidate classes (use as hypothesis grid; not exhaustive):
- **Drain-thread + main-thread cycle** — main thread waits on drain thread; drain thread waits on a pipe read that won't get EOF because the substrate doesn't propagate child-exit to drain
- **Spawn-process child startup failure** — child never starts; parent waits for child handshake that won't fire
- **Run-hermetic-driver internal channel cycle** — driver's input/output channels form a deadlock with the body's channels
- **Service-startup channel deadlock** — service driver thread blocks on input channel that the main thread blocks waiting to send to (but Sender/Receiver are in different scopes that don't trigger arc 117's check)

Classify what you find. Write the classification in the findings.

### Phase 3 — Design detection mechanism (15-30 min)

Choose detection TYPE:
- **Compile-time rule (preferred)** — extend an existing walker (probably the scope-deadlock walker) or mint a new one. Catches the pattern at check time before any code runs.
- **Runtime guard (fallback)** — substrate-level check that fires when the pattern is observed (e.g., `Process/join-result` with a built-in timeout that panics with the pattern's name; or a watchdog thread that observes the futex state).
- **Process-level timeout with panic-cascade (last resort)** — every spawn-process gets a max-runtime; on timeout, panic via the cascade machinery with a "deadlock suspected" message naming the spawn site.

Pick the strongest detection the diagnose can justify. The recovery doc + memory `feedback_never_deadlock.md` express substrate doctrine: **deadlocks are compile-time-preventable when the signal is in the type system; runtime detection is the fallback.**

### Phase 4 — Implement detection (30-60 min)

Write the substrate change that adds the detection. The change goes on this branch (`arc-170-gap-j-v5-deadlock-state`).

Design constraints:
- **Don't fix the deadlock by making it not happen** — fix it by DETECTING + PANICKING. The substrate's job is to refuse to silently hang.
- **The V5 retry deftest shape stays as the trigger** — don't revert `wat/test.wat`. The detection should fire on that shape.
- **The minimal-repro probe goes in `tests/probe_deadlock_detection_<name>.rs`** — exhibits the deadlock; under the detection, panics deterministically.
- Detection message should NAME the pattern (e.g., `"deadlock-by-X-pattern: <site>"`).

### Phase 5 — Verify (15-30 min)

```bash
# Minimal-repro probe — should PANIC (with the detection's diagnostic message), not hang
timeout 30 cargo test --release --test probe_deadlock_detection_<name> 2>&1 | tail -20

# Full workspace — the previously-hanging tests should now ALSO panic with the detection message
timeout 120 cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "(deadlock|panicked|FAILED)" | head -30
```

Expected after the detection lands:
- The minimal-repro probe panics with the detection message (test "should panic" or assert on the panic)
- The previously-hung workspace runs now COMPLETE within the timeout, with the previously-hanging tests reporting the detection panic
- Workspace count: probably `2243 + new probes / ≥7 failed` (the same 7 failures, but now they FAIL FAST with the detection message instead of hanging)

If the workspace STILL hangs after detection, the pattern is ambiguous or the detection is incomplete. Surface honestly.

## Required reading IN ORDER

1. `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` § "V5 boss-fight + Gap J diagnosis" — the conversation arc that produced this mission
2. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-E-V4-DEFTEST-REWRITE.md` — V4's original 16-failure analysis
3. `src/spawn.rs` — spawn-process substrate primitive + Process struct
4. `src/runtime.rs` — `eval_kernel_spawn_process`, `eval_kernel_thread_join_result`, channel send/recv primitives
5. `wat/test.wat` lines 460-580 — `run-hermetic-driver` + `run-hermetic` macro
6. `wat/kernel/hermetic.wat` — fork-program-ast wat-side composition
7. Memory `feedback_never_deadlock.md` — the doctrine
8. arc 117 + 126 + 131 + 132 INSCRIPTION/SCORE docs (existing deadlock-detection arcs to compare against)

## Hard constraints

- DO USE timeouts on every cargo test invocation: `timeout 60` on workspace, `timeout 30` on individual probes. Healthy = under budget.
- DO NOT modify the V5 retry deftest macro shape (`wat/test.wat`) — it's the trigger; the detection should fire under it
- DO NOT revert sonnet's `register_types` splice fix (`src/types.rs`) — it's correct foundation work
- DO NOT work around the deadlock by avoiding the path — DETECT it
- DO NOT touch `docs/arc/` except for your findings doc
- DO NOT commit / push / git add — orchestrator atomic-commits after scoring
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- DO USE gdb if useful (it's likely available; check via `which gdb` if you want to verify)
- If after 60 min you cannot identify the futex pair, STOP and report. The orphan + minimal-repro is enough evidence for the orchestrator to decide next steps.

## Deliverable

A SCORE doc at:

`docs/arc/2026/05/170-program-entry-points/SCORE-DEADLOCK-DETECTION-OPUS.md`

Containing:
- **Investigation summary** — futex pair identified (mutex/condvar addresses, parking_lot signatures, code site); minimum-repro probe path; gdb output (key frames) if used
- **Pattern classification** — which class (vs arc 117/126/131/132); is it new or existing-slipping-through
- **Detection design** — what kind of check (compile-time/runtime/timeout); where it lives; what diagnostic message it emits
- **Implementation summary** — files modified + what was added; line counts; behavior change
- **Verification** — minimal-repro panics with detection message (paste output); workspace previously-hanging tests now fail-fast with detection (paste output); 6-row scorecard with PASS/FAIL
- **Honest deltas** — anything unexpected; alternative explanations; certainty rating

Plus the actual code:
- Substrate change(s) implementing the detection (probably `src/runtime.rs` or `src/spawn.rs` or `src/check.rs` depending on compile-time vs runtime)
- Minimal-repro probe in `tests/`
- (Possibly) updates to existing detection arcs' tests if the new rule supersedes/extends them — but probably not in this slice

Then STOP. Report what shipped, where, and which mode (compile-time / runtime / timeout) the detection ended up being. The orchestrator atomic-commits.

## Predicted runtime

**90-180 min Opus.** Deep investigation work; substantive substrate design + implementation; minimal-repro construction; verification cycles.

**Hard cap:** 240 min (4 hours). If you hit the cap, surface findings even if detection isn't fully implemented — the investigation alone is valuable.

## Why this matters

This deadlock slipped past every existing detection mechanism. Naming the rule that catches it accretes into the substrate's deadlock-prevention doctrine. The substrate's *"impeccable foundation"* bar means we don't accept "this hangs sometimes" — we accept "this panics with a diagnostic that points at the cause." The cascade continues until the substrate is hostile to silent deadlock by construction.
