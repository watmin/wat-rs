# Arc 106 — INSCRIPTION

**Status:** shipped 2026-04-29.

The wat substrate's signal cascade now runs at any depth via POSIX
process groups; the wat-native polling contract
(`:wat::kernel::stopped?` etc.) works through fork. The flaky
sigterm test is purged at root, not papered over.

## What shipped

### Slice 1 — substrate (commit `b526c4b`)

`src/fork.rs::child_branch_from_source` (the cli's direct-child code
path):

- **Replaced** the SIG_DFL reset block (arc 104d's contract: child
  dies by default action; cli forwards via `kill(2)`) with two
  moves:
  1. `libc::setpgid(0, 0)` — child becomes its own process group
     leader; pgid == its pid. POSIX-mandated inheritance: any
     subsequent fork from this child or any descendant keeps the
     same pgid unless setpgid is called again.
  2. `install_substrate_signal_handlers()` — wat handlers for
     SIGINT / SIGTERM / SIGUSR1 / SIGUSR2 / SIGHUP. Each handler
     flips the matching `KERNEL_*` atomic; that atomic is what
     `:wat::kernel::stopped?` and friends read.

- New `pub fn install_substrate_signal_handlers()` and four
  `extern "C" fn substrate_on_*_signal` handlers. Async-signal-safe
  (one atomic store per handler invocation; nothing else). Distinct
  from the cli's handlers in `crates/wat-cli/src/lib.rs` — the
  cli's ALSO call `killpg(CHILD_PGID, sig)` to broadcast; the
  substrate's handlers in fork children just flip flags. The kernel
  delivers signals to every group member via the cli's killpg; per-
  process forwarding logic lives only at the cli (the entry point).

- `child_branch` (the wat-program-callable fork via
  `:wat::kernel::fork-program-ast`) is **unchanged**. No setpgid;
  no explicit handler install. The grandchild inherits the parent's
  pgid via POSIX default (no setpgid in this code path), and
  inherits the parent's signal handlers via fork-inheritance
  (parent's handlers were installed by `child_branch_from_source`
  one level up). Same outcome, less code.

`crates/wat-cli/src/lib.rs`:

- Renamed `static CHILD_PID: AtomicI32` → `static CHILD_PGID:
  AtomicI32`. Semantic: this atomic now tracks the child's process
  group, not just the single PID. The store after fork sets it to
  `child_handle.pid` because slice 1's setpgid made pgid == pid.
- Renamed comment block + cleanup line.
- `forward_signal`: `libc::kill(pid, sig)` → `libc::killpg(pgid, sig)`.
  One syscall, kernel-driven fanout to every process in the group.

### Slice 2 — purge the flake (commit `b526c4b`)

`crates/wat-cli/tests/wat_cli.rs`:

- Renamed `sigterm_to_cli_forwards_to_child` →
  `sigterm_to_cli_cascades_via_polling_contract`. New contract:

  ```
  cli ← SIGTERM → handler → flips KERNEL_STOPPED + killpg(CHILD_PGID, SIGTERM)
                  ↓
  child ← SIGTERM (kernel via group) → handler → flips KERNEL_STOPPED
                                       ↓
  wat program polls (:wat::kernel::stopped?) → observes true → returns
                                                                ↓
  :user::main returns → child _exits 0 → cli waitpid → WIFEXITED 0 → cli exits 0
  ```

- Lock-step via stdout marker. Wat program prints "READY" right
  before entering the polling loop. Test reads stdout until "READY",
  THEN sends SIGTERM. By the moment we read READY: cli forked +
  CHILD_PGID set, child setpgid'd + installed handlers + loaded
  program. **No 100ms sleep; the wire is the synchronization.**

- Test body wrapped in `wat::fork::run_in_fork` for hermetic
  isolation (chapter 29 / arc 024 — fresh signal-handler state, no
  SIGCHLD residue from earlier tests in the same binary).

- Asserts `code == Some(0)` — clean shutdown via observed stop flag.
  NOT 143 (pre-arc-106 default-SIGTERM contract). NOT None.

**100 consecutive runs: 0 failures.** Flake purged at root.

### Slice 3 — cascade-depth proof (commit `e72331e`)

`crates/wat-cli/tests/wat_cli.rs`:

- New test: `sigterm_cascades_two_levels_via_process_group`.

- The wat program (parent) forks a grandchild via
  `:wat::kernel::fork-program-ast`. Both processes poll
  `(:wat::kernel::stopped?)`. Parent prints "PARENT READY" → forks
  → enters a forward-loop reading the grandchild's stdout into its
  own stdout. Grandchild prints "GRANDCHILD READY" → enters poll
  loop. Test reads two lines (deterministic order: parent line first,
  then grandchild's via parent's forward).

- Test sends SIGTERM to cli's pid. cli handler flips +
  killpg(CHILD_PGID). **Kernel delivers SIGTERM to BOTH parent AND
  grandchild via process-group membership.** Each handler flips its
  own KERNEL_STOPPED. Each polling loop observes stopped → returns.
  Grandchild _exits 0 → stdout closes → parent's forward-loop sees
  :None → returns. Parent wait-childs the grandchild (reaps,
  observes exit 0) → exits 0. cli waitpid → 0. cli exits 0.

- Asserts Some(0). The clean exit IS the cascade proof.

**Counterfactual:** If grandchild had its own pgid (setpgid in
`child_branch` we explicitly did not add), cli's killpg would only
signal parent; grandchild would keep running; parent's forward-loop
would never return; test would hang. Test passing → cascade depth
verified.

50 consecutive runs of the cascade test: 0 failures.
30 consecutive runs of the full wat-cli suite: 0 failures.

### Slice 4 — record (this commit)

This INSCRIPTION + USER-GUIDE update + 058 row + memory entries.

## Sub-fog resolutions

**1a — canonical wat-handler installer.** Resolved by adding
`install_substrate_signal_handlers()` to `src/fork.rs` rather than
hoisting from cli. The cli's handlers do MORE (killpg-cascade); the
substrate's handlers do LESS (just flag-flip). They're sibling
implementations of the same contract at different responsibility
layers. Cli for entry-point cascade; substrate for process-internal
flag observation.

**1b — KERNEL_STOPPED state at fork time.** Resolved as documented
in DESIGN: COW inheritance is the right semantic. If the parent
already had `true` (cli is shutting down when a wat program forks a
grandchild), the grandchild inherits `true` and immediately polls
out of any loop. That IS the cascade we want.

**1c — setpgid failure mode.** Resolved by writing the error to fd
2 directly (async-signal-safe; bypasses the substrate's IOWriter
stack which holds Mutexes inherited from parent) then `_exit(EXIT_STARTUP_ERROR)`.
Defensive — should not happen in practice (child is not a session
leader) but cascade contract is non-recoverable on failure.

**2a — READY lock-step protocol.** Confirmed sufficient. Every
cascade prerequisite is settled by the time READY appears in stdout.

**2b — test runner discipline.** `wat::fork::run_in_fork` wrap.

**2c — verifying zero flake.** 100 consecutive runs, slice 2 alone.
0 failures. The flake reported pre-arc-106 (~30%) is purged at root.

**3a — how does the parent fork the grandchild?**
`:wat::kernel::fork-program-ast` with `:wat::test::program` for the
forms. Returns a `:wat::kernel::ForkedChild` struct.

**3b — what does the parent do on stopped?** Forwards grandchild's
stdout until grandchild's stdout closes (its own observation of
stopped → returns → exits 0 → fd closes), then `wait-child` to reap.
The forward-loop is the parent's "polling" — read-line returns
:None when the grandchild closes its stdout, which is the cascade
signal at the parent layer.

**3c — lock-step shape for two processes.** Two READY markers, read
in order from the cli's piped stdout. Test reads exactly two lines.

**3d — pgid inheritance verification.** The cascade-test's clean
exit IS the verification. Without inheritance, the test would hang
(grandchild never receives SIGTERM, never returns, parent's
forward-loop never sees :None).

**4a/b/c/d — slice 4 contents.** This document + USER-GUIDE update
+ 058 row + memory entries.

## What does NOT ship (per design)

- Per-program child registry. The kernel's pgid tracking is sufficient.
- A "detach this worker from the cascade" primitive. Substrate-illegal.
- A new `:wat::kernel::*` form. Existing primitives (`stopped?`,
  `sigusr1?`, etc.) work transparently in fork children once the
  handlers are installed (which they always are post-arc-106).
- A `wat-cli` flag to disable cascade. Substrate-illegal — the
  contract is mandatory.

## What this kills

- Arc 104d's "child dies with default action" contract. Replaced.
- The 100ms sleep race in `sigterm_to_cli_forwards_to_child`. Gone.
- The exit-143 expectation. Gone (replaced by exit-0 clean shutdown).
- Any future SIG_DFL-based child contract. Substrate now treats wat
  handlers as universal across process boundaries.
- The "what about detached workers?" question. Substrate-illegal,
  no opt-out.

## Lessons captured

1. **Tests are not flaky.** A flaky test is a bug the test surfaced,
   not a condition to tolerate. Find the root race, fix it at the
   substrate, rewrite the test to be deterministic. Lock-step pipes,
   polling contracts, kill the race window. Never `--ignore`, never
   `for i in 1..3 retry`.
2. **Process groups carry cascade.** When wat code forks, the child
   inherits the pgid; the cli's killpg reaches every descendant; no
   per-process registry needed; the kernel tracks membership.
3. **Wat handlers are universal across fork.** Installed in cli at
   startup; re-installed in every fork child via
   `child_branch_from_source` (or inherited via fork-inheritance for
   wat-program-forked grandchildren). `KERNEL_STOPPED` and friends
   work identically in every wat process.
4. **The wire is the synchronization.** Lock-step via stdout markers
   replaces sleeps. By the time you read READY, every prerequisite
   the program had to satisfy to print READY is done.
5. **Counterfactual proofs are the strongest cascade proofs.** The
   cascade test's clean exit IS the proof — if cascade depth broke,
   the test would hang. Negative-space evidence.

## Cross-references

- `arc/2026/04/104-wat-cli-fork-isolation/INSCRIPTION.md` — predecessor.
- `src/fork.rs:728-743` (now `:728-808`) — the SIG_DFL reset block
  this arc replaced.
- `src/runtime.rs:51-119` — KERNEL_STOPPED, KERNEL_SIGUSR1/2/HUP.
- `crates/wat-cli/src/lib.rs:104-117` — CHILD_PGID atomic.
- `crates/wat-cli/src/lib.rs:432-449` — forward_signal with killpg.
- `docs/CIRCUIT.md` — the wiring philosophy this arc upholds.

## Test summary

- Substrate: 95+ test binaries green; 0 failures.
- Polling-contract test (slice 2): 100/100 green.
- Cascade-depth test (slice 3): 50/50 green.
- Full wat-cli suite (10 tests + 2 new): 30/30 green.
