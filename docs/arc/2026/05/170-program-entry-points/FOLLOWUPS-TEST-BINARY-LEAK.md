# Follow-up: deadlocking-test-binary leak (corrected diagnosis)

**Status:** investigated 2026-05-10 across multiple passes during arc 170 1f-δ/ε/ζ/η. The root cause was misdiagnosed twice before the correct framing emerged from forensic inspection of currently-alive orphans. **This is the corrected note.**

## Symptom

`ps faux | grep wat-rs` shows N orphan processes parented to init (PID 1), state `S/Sl`, blocked in `futex_do_wait` (per `/proc/$pid/wchan`).

Example observed 2026-05-10 20:51:32 → still alive at 20:58 (7 minutes later, unresponsive):

```
PID 2026042  PPID 1  S  futex_do_wait
  cmdline: target/release/deps/test-32fbbe6ae6c5f433 --test-threads=1 --exact deftest_stderr_test_add_and_write
  fds: 0,1,2 = pipes ; 3,4,5 = dup'd copies (synthesize_real_fd_stdio from slice 1f-γ)
```

All 9 orphans observed had identical structure: same binary, same dup'd-fd pattern, same futex wait, all known-deadlocking hermetic tests.

## Root cause (CORRECTED)

**The orphans are NOT forked children of test binaries.** They ARE the test binaries themselves, stuck in `futex_do_wait` because:

1. The hermetic test runs the slice 1f-γ orchestrator (`invoke_user_main_orchestrated`)
2. The orchestrator spawns three substrate services + bridge threads
3. The test body has a deadlock — a thread blocks on a channel recv that never completes (e.g., `spawn-shape` tests' scope-deadlock pattern; `add-and-write` test body deadlock)
4. The main test thread enters futex wait, never returns
5. `cargo test` / `timeout 10 ...` sends SIGTERM
6. SIGTERM panics the test runner's outer thread, but the inner futex-blocked thread keeps the process alive (Rust can't safely kill a thread from outside)
7. The parent (cargo / timeout / walk script) exits → test binary reparents to init → orphan

**This is NOT a fork.rs lacuna. It's a deadlocking-test-binary SIGTERM-unresponsiveness issue.**

## What's actually happening (worked example)

The 9 orphans observed during a per-test walk used `timeout 10 $BINARY --exact <test_name>`:

```bash
timeout 10 target/release/deps/test-32fbbe6ae6c5f433 --test-threads=1 --exact deftest_stderr_test_add_and_write
```

When the test deadlocks:
- `timeout` waits 10s, sends SIGTERM (GNU coreutils default)
- $BINARY's main thread is in `futex_do_wait` — SIGTERM panics one Rust thread but the binary's main loop keeps the process alive
- `timeout` exits (its job is done after SIGTERM; no `--kill-after` was set)
- $BINARY becomes orphan

**`timeout --kill-after=2 10` would have escalated to SIGKILL after 2 more seconds — `kill -9` is unignorable. The walk script lacked this flag.**

## The actual issues to fix

### Tier 1 — walk script hygiene (trivial, immediate)

Whenever shelling out to a test binary with a timeout, always use:
```bash
timeout --kill-after=2 10 $BINARY ...   # SIGKILL escalation
```

This is a shell-script discipline; not a substrate change.

### Tier 2 — test binary unresponsive-to-SIGTERM (medium)

The test binary should be capable of dying cleanly under SIGTERM even when test threads are deadlocked. Options:
- Install a SIGTERM handler at test-binary startup that does `libc::_exit(N)` after a grace period (e.g., 1 sec)
- This bypasses Rust Drop entirely (which is OK — the test process is dying anyway)
- Trade-off: any orphaned-child reap relying on Drop is also bypassed; need to combine with Tier 3

### Tier 3 — substrate PR_SET_PDEATHSIG for forked children (small)

For ACTUAL forked children (different from this leak): add `prctl(PR_SET_PDEATHSIG, SIGKILL)` in fork-child branches. This is the original substrate-fix idea — still valid as defense-in-depth for cases where a fork-child's parent dies and Drop doesn't run.

Sites: `src/fork.rs` (5 fork sites) + `src/spawn_process.rs:275+`. ~30 min sonnet, mechanical.

### Tier 4 — fix the deadlocks themselves (large; not this slice)

The hermetic tests with deadlocks (`spawn-shape`, `add-and-write`, `multi-thread-routing`, `remove-drops-entry`) have scope-deadlock or channel-recv-forever bugs in their bodies. Fixing those eliminates the source. These are test-body bugs (introduced when the tests were authored in slices 1f-β-i/ii/iii); a triage slice would close them.

## Reproduction

```bash
# Set up the leak
ps faux | grep wat-rs   # should be empty
timeout 10 target/release/deps/test-32fbbe6ae6c5f433 --test-threads=1 --exact deftest_stderr_test_add_and_write
ps faux | grep wat-rs   # one orphan, futex_do_wait
```

The leak is 100% reproducible with this single-test invocation against a deadlocking hermetic test.

## Cross-references

- `src/fork.rs:219-232` — `ChildHandleInner::drop` (existing fork-child cleanup; complementary to PDEATHSIG)
- `crates/wat-macros/src/lib.rs:707-715` — time-limit's "we can't kill a Rust thread" admission (related; addresses the inner-thread case but not the SIGTERM case)
- Slices 1f-β-i/ii/iii — origin of the hermetic tests with body deadlocks
- This investigation's prior drafts (now corrected) — premature framings of fork.rs as the source; user's discipline question ("what is your assertion of accuracy") forced the correction
