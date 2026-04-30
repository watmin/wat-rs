# Arc 106 — Signal cascade and polling contract through fork

**Status:** in flight (2026-04-29)
**Predecessor:** arc 104d (wat-cli signal forwarding via `kill(2)` to a
single child PID). Arc 106 generalizes that mechanism in two directions
at once — depth (process groups instead of single PID) and contract
(polling stopped? returning 0 instead of default-action 143).

## The findings driving this arc

Three findings landed in one session:

**1. Cascade depth.** Arc 104d's `kill(2)` forwards SIGTERM to ONE child
pid — the cli's direct fork. If that child itself forks via
`:wat::kernel::fork-program`, the grandchildren are not signaled. The
cascade stops at depth one. This is silent — nothing crashes, but a
SIGTERM to the cli leaves grandchildren running until their natural
exit. Wrong.

**2. Broken polling contract through fork.** `:wat::kernel::stopped?`
is the substrate's wat-native signal observation primitive — a flag set
by the wat signal handler when SIGTERM/SIGINT arrives, polled by the
program to decide when to exit cleanly. Pre-arc-104, when the wat
program ran in the cli's process, this worked. Post-arc-104,
`child_branch_from_source` resets all signal handlers to `SIG_DFL`
(at `src/fork.rs:741`) — meaning the child's `KERNEL_STOPPED` flag is
never set by SIGTERM, because there's no handler to set it. The wat
program's `(:wat::kernel::stopped?)` always returns `false` in a
forked child, which renders the contract useless.

**3. Flaky test.** The arc 104d test (`sigterm_to_cli_forwards_to_child`)
asserts the kill-forwarding contract: SIGTERM to cli → cli forwards to
child via kill(2) → child dies with default SIGTERM action → cli's
waitpid returns `WIFSIGNALED + WTERMSIG=15` → cli exits 143. The test
flakes ~30% — sometimes the child exits 0 instead. Root cause is a
signal-delivery / waitpid-return race window. The test is not flaky
because the test is bad; the test is flaky because the contract
underneath it has a race window.

The three findings have ONE cure.

## The cure

**Process groups carry the cascade. Polling carries the contract.
Together they remove the race.**

### Process groups

After fork, the child calls `setpgid(0, 0)` — making itself its own
process group leader. Every subsequent fork inside the wat program
inherits this pgid. Grandchildren, great-grandchildren, recursively —
all members of the group. The kernel does the bookkeeping; the
substrate does not maintain a registry.

The cli's signal handler swaps `kill(child_pid, sig)` for
`killpg(child_pgid, sig)`. Same atomic, same handler, different
syscall. The kernel fans the signal out to every process in the group.

### Polling contract through fork

`child_branch_from_source` at `src/fork.rs:728-743` currently resets
signal handlers to `SIG_DFL`. **It should install the wat handlers
instead.** Then:

- SIGTERM arrives at cli's wat handler → flips `KERNEL_STOPPED` in cli's
  memory + `killpg(CHILD_PGID, SIGTERM)`
- SIGTERM arrives at every process in the group via the kernel's
  delivery
- Each process's wat handler fires → flips its own `KERNEL_STOPPED`
- Wat programs poll `(:wat::kernel::stopped?)` → observe `true` →
  return cleanly
- Each child exits with code 0 (clean exit, not signal-killed)
- The cli's waitpid returns `WIFEXITED` with code 0; cli exits 0

### Why this removes the race

Today's flake is in the post-SIGTERM window: the child is killed by
default action; waitpid races signal delivery; sometimes the kernel's
WIFSIGNALED bit is set, sometimes the process appears as exit 0. With
the polling contract, **the child exits cleanly, not via signal kill.**
There is no signal-delivery race because the program returned `()` —
the kernel sees a normal exit. waitpid returns `WIFEXITED` deterministically.

## Cascade is mandatory; no opt-out

The substrate enforces the cascade. A wat program cannot detach a
forked child from the parent's process group. If a "detached" worker
is needed, it's not a child — it's a separate program the operator
launches separately. The substrate only knows children, and children
inherit the pgid by construction.

This is non-negotiable. Allowing detached children would mean
maintaining per-process child registries, fragmenting the cascade,
losing the kernel's free bookkeeping. The substrate measures
membership through the kernel's process-group abstraction; userland
cannot opt out.

## The slices

### Slice 1 — substrate (the cascade + the polling contract)

**Files:**

- `src/fork.rs::child_branch_from_source` (lines 728-743)
- `crates/wat-cli/src/lib.rs` (CHILD_PID + signal handlers)

**Substrate changes:**

`src/fork.rs::child_branch_from_source`:
1. After `dup2`/`close_inherited_fds_above_stdio`, **before** the
   handler block, call `libc::setpgid(0, 0)`. Failure is non-fatal —
   `_exit(EXIT_STARTUP_ERROR)` if `setpgid` returns -1 (it should not,
   but defensive).
2. **Replace** the SIG_DFL reset block. Instead of resetting handlers
   to default, install the same wat handlers the cli installs at
   startup — the handlers that flip `KERNEL_STOPPED` /
   `KERNEL_SIGUSR1` / etc. The cli's `install_signal_handlers()` (or
   its equivalent in wat-cli's lib.rs) becomes the canonical handler
   set; the fork branch installs the same handlers.

`crates/wat-cli/src/lib.rs`:
1. Rename `static CHILD_PID: AtomicI32` → `static CHILD_PGID: AtomicI32`.
2. After fork: `CHILD_PGID.store(handles.child_handle.pid, Ordering::SeqCst)`
   (the child is its own pgid because of slice 1's setpgid call —
   pgid == pid).
3. Signal handlers: `libc::kill(pid, sig)` → `libc::killpg(pgid, sig)`.
4. Cleanup: same store-back to -1 after waitpid.

**Compiles, passes existing tests** (the kill-default-action contract
still holds for any test that doesn't use stopped? polling — but the
arc 104d test gets rewritten in slice 2).

### Slice 2 — purge the flake (the polling contract test)

**File:** `crates/wat-cli/tests/wat_cli.rs`

Rewrite `sigterm_to_cli_forwards_to_child` to assert the polling
contract:

```rust
#[test]
fn sigterm_to_cli_cascades_via_process_group_polling() {
    wat::fork::run_in_fork(|| {
        let program = r#"
            (:wat::core::define (:demo::loop
                                 (stdout :wat::io::IOWriter)
                                 -> :())
              (:wat::core::if (:wat::kernel::stopped?) -> :()
                ()
                (:demo::loop stdout)))

            (:wat::core::define (:user::main
                                 (stdin  :wat::io::IOReader)
                                 (stdout :wat::io::IOWriter)
                                 (stderr :wat::io::IOWriter)
                                 -> :())
              (:wat::core::let*
                (((_ :()) (:wat::io::IOWriter/println stdout "READY")))
                (:demo::loop stdout)))
        "#;
        // … spawn wat-cli with stdin/stdout piped, read READY, send SIGTERM,
        // wait, assert exit 0 (clean shutdown via polling contract).
    });
}
```

Run 100×. Zero failures acceptable. If even one flake surfaces, the
arc is incomplete — the race is somewhere we missed.

The READY lock-step ensures by-the-time-we-signal:
- cli has fork()ed (CHILD_PGID set)
- cli has installed signal handlers
- child has installed wat signal handlers (slice 1)
- child has called setpgid (slice 1)
- child wat program is running
- child wat program is in the polling loop

After SIGTERM:
- cli's wat handler flips KERNEL_STOPPED (in cli)
- cli's wat handler calls killpg(child_pgid, SIGTERM)
- kernel delivers SIGTERM to child
- child's wat handler flips KERNEL_STOPPED (in child)
- child polls stopped? → true → returns ()
- :user::main returns → child _exits 0
- cli's waitpid returns WIFEXITED with code 0
- cli exits 0

Test asserts `Some(0)`, deterministic.

### Slice 3 — cascade depth test (the real proof)

**File:** `crates/wat-cli/tests/wat_cli.rs` (new test).

A wat program that forks a grandchild via `:wat::kernel::fork-program`,
both poll `stopped?`, both exit cleanly. SIGTERM to cli → killpg →
both processes flip flag → both exit 0 → grandchild reaped by parent
→ parent exits 0 → cli exits 0.

```rust
#[test]
fn sigterm_to_cli_cascades_two_levels_deep() {
    wat::fork::run_in_fork(|| {
        // Parent program: forks a grandchild that runs the same
        // poll-stopped loop. Parent waits for grandchild, exits 0.
        // Grandchild polls stopped?, exits 0. Both flip on cascade.
        let program = r#"
            ;; Grandchild source — text, will be forked from parent.
            ;; (:wat::kernel::fork-program-source ...) inherits the
            ;; parent's pgid by construction.
            ...
        "#;
        // Spawn cli with this program. Read "PARENT READY" + "GRANDCHILD
        // READY" markers from stdout (lock-step on both processes
        // having entered their polling loops). Send SIGTERM. Both
        // exit 0. waitpid in parent reaps grandchild. cli exits 0.
    });
}
```

This is the load-bearing proof of the cascade. Without it, "process
groups cascade" is a claim, not verified property.

### Slice 4 — INSCRIPTION + USER-GUIDE + 058 row + memory entry

- `INSCRIPTION.md` here, recording slice contents + commit refs.
- USER-GUIDE.md update on the polling contract (now works through
  fork) and the cascade contract (signals propagate to every
  descendant).
- 058 FOUNDATION-CHANGELOG row in the lab repo.
- Memory entry: *tests are not flaky; flakes are bugs to be fixed at
  root.* Captures the discipline that this arc embodies — never
  `--ignore`, never "retry once," fix the underlying race.

## What this kills along the way

- Arc 104d's "child dies with default action" contract — replaced by
  polling.
- The 100ms sleep race in the test — gone, READY is the wire.
- The exit-143 expectation — replaced by exit-0 (clean shutdown).
- Any future SIG_DFL-based child contract — substrate now treats wat
  handlers as universal across process boundaries.
- The "what about detached workers?" question — substrate-illegal, no
  opt-out.

## Open questions

None blocking. A few worth recording:

**Q1.** TTY foreground process group interaction — when wat-cli runs
attached to a terminal and the user hits Ctrl-C, the TTY sends SIGINT
to the foreground pgid (the cli's group, by default). Both the cli AND
its wat children receive SIGINT directly. Each wat handler flips its
own KERNEL_STOPPED; each program polls and exits cleanly. The cli's
killpg becomes redundant in this case (the kernel already broadcast).
Either way: clean exit. Behavior: unchanged from user perspective.

**Q2.** SIGUSR1/SIGUSR2/SIGHUP — same cascade applies. The cli's wat
handler flips its own flag + killpg. Each child's wat handler flips
its flag. Wat programs poll the corresponding `sigusr1?` / `sigusr2?`
/ `sighup?` primitives. No new substrate; same shape as SIGTERM.

**Q3.** Reset semantics — `KERNEL_STOPPED` is set-once per process
(the substrate convention). User-signal flags (`KERNEL_SIGUSR1`, etc.)
are reset by `:wat::kernel::reset-sigusr1!` etc. After cascade, each
process maintains its own flag state independently — the cli flipping
its KERNEL_STOPPED doesn't affect the children's flag state.

## Memory entries to write afterward

1. **Tests are not flaky** — capture the discipline. Flakes are bugs
   the test surfaced; fix at root, never retry/ignore.
2. **Process groups carry cascade** — when wat code forks, the child
   inherits the pgid; the cli's killpg reaches every descendant; no
   per-process registry needed; the kernel tracks membership.
3. **Wat handlers are universal across fork** — installed in cli at
   startup, re-installed in every fork child by `child_branch_from_source`.
   `KERNEL_STOPPED` and friends work identically in every wat process.

## What does NOT ship

- Per-program child registry. The kernel's pgid tracking is sufficient.
- A "detach this worker from the cascade" primitive. Substrate-illegal.
- A new `:wat::kernel::*` form. Existing primitives (`stopped?`,
  `sigusr1?`, etc.) work transparently in fork children once the
  handlers are installed.

## Cross-references

- `arc/2026/04/104-wat-cli-fork-isolation/` — the predecessor; this
  arc generalizes the kill-forwarding mechanism.
- `src/fork.rs:728-743` — the SIG_DFL reset block this arc replaces.
- `src/runtime.rs:51-119` — the KERNEL_STOPPED + KERNEL_SIGUSR1/2/HUP
  flag definitions.
- `crates/wat-cli/src/lib.rs:CHILD_PID` — the atomic this arc renames
  to CHILD_PGID.
- `docs/CIRCUIT.md` — the wiring diagram philosophy this arc upholds
  (signals are part of the wiring; cascade is part of the topology).
