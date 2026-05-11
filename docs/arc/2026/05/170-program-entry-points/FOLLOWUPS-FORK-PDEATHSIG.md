# Follow-up: fork process-leak fix (PR_SET_PDEATHSIG)

**Status:** investigated 2026-05-10 during slice 1f-ζ verification runs; queued for separate slice.

## Reproduction (specific tests, not all forks)

The leak is NOT a substrate-wide fork misuse. It's specific to **hermetic tests with body-level deadlocks** (e.g., `stdin-test::spawn-shape`, `stdout-test::spawn-shape`, `stderr-test::spawn-shape` from slice 1f-β-i/ii/iii — known structural scope-deadlocks).

Verified reproduction: `cargo test --release --workspace stdin_test_spawn_shape` leaves **9 orphans** that persist after cargo test exits.

## Root cause — time-limit doesn't kill the inner thread

`crates/wat-macros/src/lib.rs:707-715` documents the leak explicitly:

```rust
Err(::std::sync::mpsc::RecvTimeoutError::Timeout) => {
    // Real timeout: inner thread is still running.
    // We can't safely kill a Rust thread from
    // outside; the runaway worker leaks until
    // process exit. Synthesized message preserves
    // arc-123's existing UX.
    panic!(#timeout_msg);
}
```

The flow:
1. `deftest-hermetic` macro spawns inner thread to run the test body
2. Test body runs `run-sandboxed-hermetic-ast` → `fork-program-ast` → child process
3. Test body deadlocks (scope-deadlock in stdin/stdout/stderr spawn-shape tests)
4. 200ms time-limit fires (arc 132 default)
5. OUTER thread panics with timeout message
6. INNER thread (holding `Arc<ChildHandleInner>` for the forked child) **keeps running**
7. Arc doesn't drop; `ChildHandleInner::drop` never fires; child stays alive
8. Cargo test process eventually exits without running Rust Drops on the leaked inner thread → child reparents to init

**This is not a fork.rs lacuna. It's an upstream design decision in `wat-macros`.**

## Symptom

`ps faux | grep wat-rs` shows N orphan child processes parented to `?` (init) in state `Sl` (sleeping) during long-running test workloads. User killed manually multiple times. Example observed during slice 1f-ζ:

```
watmin 1612965 ... 1907096 ... ?  Sl  19:20  test-73742482e4c4dc8d
watmin 1612967 ... 1907096 ... ?  Sl  19:20  test-73742482e4c4dc8d
... (15+ instances) ...
```

All orphaned (parent `?`), all sleeping. Sticking around because:
1. Their original parent (cargo test runner / wat-rs test binary) died
2. They're blocked on stdin read / channel recv waiting for input that will never come

## Root cause

`src/fork.rs` (multiple sites) and `src/spawn_process.rs:275+` call `libc::fork()` then `libc::setpgid(0, 0)` in the child branch. The substrate has TWO cleanup paths:

1. **`ChildHandleInner::drop`** at `src/fork.rs:219-232` — runs `kill(SIGKILL) + waitpid` when the Arc reaches refcount 0. Used in clean cases.
2. **Process-group cascade** via `setpgid(0, 0)` + `killpg` in CLI signal handlers — used for SIGINT/SIGTERM forwarding.

**Neither survives:**

- Parent `SIGKILL` from outside (user kills cargo test, OOM killer, external `kill -9`)
- Parent `std::process::abort` (panic = "abort" profile, or `assert!` macros under abort)
- Cargo test runner timeout reaping its workers

When parent dies without Drop running, the kernel reparents the child to init. Child keeps running indefinitely (or until manually killed).

**Grep confirms:** `grep -rn PR_SET_PDEATHSIG src/` returns **zero matches**. The Linux kernel-level death-signal mechanism is not used anywhere in the substrate.

## The fix

Add `prctl(PR_SET_PDEATHSIG, SIGKILL)` in the child branch IMMEDIATELY after fork in every fork site. This Linux mechanism: child registers "kill me when my parent dies (any reason)" and the kernel handles it autonomously — no parent participation needed.

**Why this is right:**

- Belt-and-suspenders. Existing `Drop`-based reap still runs in clean cases; PDEATHSIG catches catastrophic cases.
- No code removed, just one additional `prctl` call per child branch.
- Linux-only — acceptable per `feedback_no_windows` (wat-rs is Linux-first).
- Race window between `fork()` and `prctl()` is microseconds; parent would have to die in that exact window to leave an orphan.

## Fork sites needing the call

```bash
grep -n "if pid == 0\|pid == 0 {" src/fork.rs src/spawn_process.rs
```

Approximate sites (verify line numbers at slice time — may have shifted post-Console retirement):

- `src/fork.rs:~140` (`run_in_fork` — generic forker)
- `src/fork.rs:~530` (main `fork-program-ast` path)
- `src/fork.rs:~801` (sibling fork site)
- `src/fork.rs:~947` (another fork site)
- `src/fork.rs:~1008` (another fork site)
- `src/spawn_process.rs:~275` (`spawn_process_child_branch`)

## The edit (each site)

Immediately after the `if pid == 0` block enters (before any other child setup like `setpgid`):

```rust
unsafe {
    // PR_SET_PDEATHSIG — kernel sends SIGKILL to this child if
    // its parent dies for any reason (clean exit, SIGKILL, abort,
    // panic-abort). Belt-and-suspenders alongside
    // ChildHandleInner::drop's explicit reap path — catches the
    // catastrophic-parent-death cases where Drop never runs.
    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong);
}
```

`libc::prctl`, `libc::PR_SET_PDEATHSIG`, and `libc::SIGKILL` are all already available via the `libc` crate; no new dependencies.

## Verification

```bash
# Spawn a test workload that forks
cargo test --release --workspace --no-fail-fast &
WORKLOAD_PID=$!

# Wait briefly
sleep 5

# Kill the parent harshly
kill -9 $WORKLOAD_PID

# Wait a beat
sleep 2

# Should be NO orphan wat-rs processes
ps faux | grep wat-rs | grep -v grep
```

Without the fix: orphans accumulate (current behavior).
With the fix: orphans auto-die within ~1 sec of parent death (SIGKILL is unignorable).

## Predicted slice scope

~30 min sonnet. ~6 fork sites × 3-line edit each. Mechanical. Single-source-of-truth pattern.

## Two complementary fixes

Both are worth shipping; they address different layers:

**Layer 1 (substrate) — PDEATHSIG** (this slice's primary target):
- Catches catastrophic-parent-death cases (SIGKILL, abort, panic-abort).
- Fixes the symptom for ALL fork callers.

**Layer 2 (wat-macros) — kill inner-thread's children on timeout**:
- Track `Arc<ChildHandleInner>` (or equivalent registry) per test thread.
- On timeout: walk the registry, kill children, then panic the outer thread.
- OR: run inner test in a process (not thread) so the timeout can SIGKILL the whole process group.
- This is the upstream fix that prevents the leak even when the test PROCESS keeps running.

For arc 170's immediate purposes — Layer 1 alone is sufficient. The leak is bounded by the test-process lifetime; PDEATHSIG handles the final cleanup. Layer 2 is its own foundation work (call it FOLLOWUPS-TIMELIMIT-LEAK.md when authored).

## Cross-references

- `src/fork.rs:219-232` — `ChildHandleInner::drop` (existing cleanup; complementary to PDEATHSIG)
- `src/fork.rs:74-76` — process-group + killpg cascade comments (existing signal-forwarding doctrine)
- `docs/ZERO-MUTEX.md` — substrate doctrine (PDEATHSIG doesn't introduce Mutex; permitted)
- This file is the standing artifact for the fix; a future slice's BRIEF references it
