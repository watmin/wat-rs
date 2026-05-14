# Arc 170 — FD-multiplex shutdown (unified): retires Slice C's prctl, subsumes Slice E

**Branch:** arc-170-gap-j-v5-deadlock-state
**Date:** 2026-05-13
**Status:** DESIGN — awaiting orchestrator green light to BRIEF + spawn

## What this slice does

One unified change to the substrate's shutdown machinery:

1. **The shutdown worker's wait grows from one FD to N.** Today: `wat-shutdown-worker` does `read(wake_pipe_read_fd, 1 byte)`. After: `poll(wake_pipe_read_fd, lifeline_pipe_read_fd, ...)` — any FD ready → `trigger_shutdown()`.

2. **Spawn-process plumbs a lifeline pipe.** Parent holds write-end (never writes); child inherits read-end. When parent dies for any reason (`_exit`, panic, SIGKILL, OOM-kill, segfault), kernel closes the parent's FDs as part of process teardown → child's lifeline read-end EOFs → shutdown worker's poll returns → cascade fires.

3. **PR_SET_PDEATHSIG retires.** Both `spawn_process_child_branch` (src/spawn_process.rs:343) and `child_branch_from_source` (src/fork.rs:1066) drop the prctl call. The "early `init_shutdown_signal()` before `install_substrate_signal_handlers`" race-closing edit (Slice C SCORE § "Early init") goes away — there's no PDEATHSIG signal to race against.

4. **Tier-2 PipeFd Receivers gain shutdown-awareness via the same multiplex.** Slice E's original goal (from-pipe Receivers wake on shutdown via `epoll`/`poll(2)` over (pipe_fd, shutdown_eventfd)) is the OUTPUT-side application of the same primitive that the lifeline applies INPUT-side. One implementation; both surfaces gain coverage.

## Why this is one slice, not three

The shutdown worker is a single point in the substrate that owns the shutdown-event-multiplex set. Today it has one input (the wake-pipe written by signal handlers). The forward move grows the input set:

| Input | Source | Today | After |
|---|---|---|---|
| signal handler write byte | `substrate_on_stop_signal` writes `'!'` on SIGTERM/SIGINT | wake-pipe | wake-pipe (unchanged) |
| parent-process death | kernel closes parent's FDs on `_exit`/panic/SIGKILL | (no input — PDEATHSIG path) | **lifeline-pipe EOF** |
| future: signalfd | per arc 197 | n/a | additional pollfd |

The mechanism is uniform: every shutdown-input is a pipe FD; the worker polls them all; any ready → trigger_shutdown. The set grows by adding pollfd entries; the cascade downstream is unchanged.

Slice E's tier-2 Receiver surface ALSO becomes natural: a `Receiver/from-pipe(reader)` is implemented as `poll(reader.fd, shutdown_pipe.fd, ...)`. Same primitive. The tier-2 multiplex IS the substrate's multiplex, exposed at the user surface.

## What gets retired

### Slice C mechanism (preserved as historical record per `feedback_inscription_immutable`)

| Site | Edit |
|---|---|
| `src/spawn_process.rs:343-361` | DELETE the `libc::prctl(PR_SET_PDEATHSIG, SIGTERM, ...)` block + its emit_structured_exit error path |
| `src/spawn_process.rs:369` | DELETE the early `crate::runtime::init_shutdown_signal()` call. The later call in `bootstrap_wat_vm_process` becomes the sole site. The race that motivated the early call (Slice C SCORE § "Early init") no longer exists — no PDEATHSIG signal can fire pre-bootstrap |
| `src/fork.rs:1066-1080` | Symmetric DELETE in `child_branch_from_source` |
| `tests/probe_pdeathsig_kills_orphan_child.rs` | KEEP as historical regression marker. Add header note that PDEATHSIG-based mechanism retired; probe still runs against the new mechanism (lifeline path); should still pass (orphan grandchild dies cleanly via lifeline EOF, not SIGTERM) |

Slice C's SCORE-SLICE-C-PDEATHSIG.md stays. Slice C's commit (`fb9522d`) stays.

### What stays

- `setpgid(0,0)` in both child branches — unchanged. Distinct discipline (signal cascade within process group); see `project_signal_cascade`.
- `install_substrate_signal_handlers` for SIGTERM/SIGINT — unchanged. These handle user-driven signals (Ctrl-C, `kill -TERM <wat-cli-pid>`) which still route via wake-pipe.
- `substrate_on_stop_signal` + wake-pipe write — unchanged. This is one input to the multiplex.
- The entire crossbeam Sender::Drop fanout (Slices A+B) — unchanged. The lifeline triggers the SAME `trigger_shutdown()` function; the downstream cascade is identical.

## What gets added

### Substrate-owned lifeline pipe per spawn-process / fork-program

`spawn_process.rs::eval_kernel_spawn_process`:

```rust
// BEFORE the libc::fork() call:
let (lifeline_r, lifeline_w) = make_pipe(":wat::kernel::spawn-process::lifeline")?;
let lifeline_r_raw = lifeline_r.as_raw_fd();

// existing fork; child branch:
//   - drop parent-side ends (lifeline_w)
//   - dup the lifeline_r into bootstrap's known location OR pass into
//     bootstrap_wat_vm_process via BootstrapArgs.lifeline_fd
//   - bootstrap registers it as the second pollfd in shutdown worker
// parent branch:
//   - drop child-side end (lifeline_r)
//   - hold lifeline_w in ChildHandleInner OR drop-binding tied to ProgramHandle
//     so parent's process death (or explicit handle drop) closes it
```

The parent does NOT need to be wat-vm-aware. The lifeline write-end is just an `OwnedFd` held inside the substrate's parent-side handle structure. Process death closes it; explicit `drop` on the handle closes it. Both routes are the substrate's existing FD-ownership pattern.

### Shutdown worker's poll grows

`runtime.rs::init_shutdown_signal()`:

```rust
// today:
let _ = unsafe { libc::read(read_fd, buf.as_mut_ptr() as *mut _, 1) };

// after: poll on N FDs; on any ready → trigger_shutdown
let mut fds: Vec<libc::pollfd> = vec![
    libc::pollfd { fd: wake_pipe_read_fd, events: libc::POLLIN, revents: 0 },
    // lifeline_pipe_read_fd added when bootstrap calls register_shutdown_input(lifeline_fd)
];
loop {
    let n = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as _, -1) };
    if n > 0 { break; }  // any FD ready (POLLIN or POLLHUP) → shutdown
    // EINTR / spurious wakeups: retry
}
trigger_shutdown();
```

The worker only fires once (existing behaviour). The poll wait blocks on multiple FDs; the first to fire wins. POLLHUP on a pipe FD (when all writers close) is sufficient — no payload needs to arrive.

### Bootstrap API gains an input registration

`bootstrap_wat_vm_process` accepts an optional `lifeline_fd: Option<i32>` in `BootstrapArgs`. When present, it's appended to the shutdown worker's pollfd set before the worker spawns. The substrate's existing `init_shutdown_signal()` becomes idempotent w.r.t. the input set rather than the worker spawn alone.

For wat-cli's main wat-vm process: no parent in the substrate sense; no lifeline registered; worker has one input (the wake-pipe).

For spawn-process children: bootstrap receives the lifeline_fd from spawn_process_child_branch; worker has two inputs.

For fork-program-ast children (per arc 104): symmetric registration.

## Mechanism check against the four substrate rules

| Rule | Check |
|---|---|
| **ZERO-MUTEX** | No new Mutex. The pollfd set is owned by the worker thread (single owner). Inputs are added BEFORE the worker spawns (during bootstrap); no concurrent mutation |
| **Lock-step (no wall-clock)** | `poll(fds, -1)` — infinite timeout; OS-level event wait. No `nanosleep`, no `recv_timeout`. Kernel signals readiness via OS-native event mechanism |
| **Structural-enforcement-over-runtime** | The lifeline is invisible to user wat code. It's a substrate-owned fd pair; the parent's handle holds the write-end via Rust ownership (Drop closes it); the kernel closes it on process death. User cannot accidentally bypass parent-death detection because there's no API surface to bypass |
| **Substrate-imposed-not-followed** | Every spawn-process child gets the lifeline automatically. The shutdown worker polls it automatically. No user opt-in. Same shape as `typed_recv`'s shadow-channel: no API to call recv without observing shutdown |

## Probe + regression coverage

Existing probes that exercise this path:

- `probe_pdeathsig_kills_orphan_child` (Slice C's probe) — STILL VALID. The grandchild's parent dies; grandchild detects via lifeline EOF (instead of SIGTERM); cascade fires; grandchild exits cleanly. Probe passes regardless of mechanism.
- `probe_shutdown_cascade_crossbeam` (Slice B's probe) — UNCHANGED. Tests SIGTERM → cascade. The signal-handler-write-to-wake-pipe path stays.
- `probe_pdeathsig_diagnostic` (Slice D's diagnostic) — KEEP as regression marker; with the lifeline mechanism in place, delay=0 should produce 50/50 PASS (no orphans). Becomes the leak-zero gate.
- `probe_lifeline_pipe_proof` (Slice D's proof) — KEEP as substrate-mechanism documentation. 100/100 stays the standard.

New probe to add in this slice:

- `probe_lifeline_orphan_clean_via_substrate` — same shape as `probe_pdeathsig_kills_orphan_child` but routed through the new substrate plumbing (instead of bare libc fork). Verifies the substrate-level integration: supervisor wat-vm calls `:wat::kernel::spawn-process`; grandchild blocks on recv; supervisor `_exit`s without waiting; grandchild dies within 100ms via lifeline EOF cascade.

## Honest open edges

### The `wat-cli` parent process

When user runs `wat my-program.wat`, wat-cli is the wat-vm. It has NO parent in the substrate sense (its parent is the shell). The wat-cli wat-vm process has no lifeline input registered. Shell death does not propagate via this mechanism — but the shell's SIGHUP / closed-TTY mechanism already routes through `install_substrate_signal_handlers` and the existing wake-pipe path. Unchanged.

### Children of children (recursive spawn-process)

The fractal architecture (per INTERSTITIAL § "Spawn-process composes recursively"): L1 spawns L2; L2 spawns L3. Each spawn-process creates a fresh lifeline pipe pair. L2's parent (L1) holds L2's lifeline-write. L3's parent (L2) holds L3's lifeline-write. When L1 dies, L2's lifeline EOFs → L2 starts shutting down → L2's drop discipline closes L3's lifeline-write (via dropping L3's ProgramHandle) → L3's lifeline EOFs. Cascade rides the tree naturally.

### The `child_branch` (forms-based fork-program-ast) path

Per Slice C SCORE: `child_branch` did NOT receive PDEATHSIG (it has no setpgid; arc 106 discipline was not applied). Same scoping for this slice — lifeline plumbed in `child_branch_from_source` (the modern path) only. If `child_branch` survives until arc 170 close, it stays as legacy without lifeline coverage. Per arc 109 retirement plan, `child_branch` retires entirely.

## Why this isn't a deferral

Per `feedback_pivot_not_defer` + `feedback_deferral_bias_is_signal`: when reaching for "defer X," the bias IS evidence X is load-bearing. Slice C's mechanism is empirically race-prone (10% rate at 50 trials); the substrate has a known better mechanism (lifeline, 100/100); the path forward is to ship the better mechanism, not to delay.

Per `feedback_no_known_defect_left_unfixed`: known defect with known fix → ship now.

## Slice composition

| Phase | What | Size | Verify |
|---|---|---|---|
| 1 | Substrate plumbing: lifeline pipe pair in spawn_process + fork-program-ast; thread through bootstrap | S-M | cargo build --release --workspace |
| 2 | Shutdown worker: replace `read` with `poll(N fds, -1)`; register lifeline_fd via bootstrap | S | unit test on poll exit |
| 3 | Retirement: delete prctl + early init_shutdown_signal from both child branches | S | cargo build clean |
| 4 | Probe: new `probe_lifeline_orphan_clean_via_substrate`; verify Slice D diagnostic now 50/50 PASS at delay=0 | S | leak-zero on 50 trials |
| 5 | Tier-2 PipeFd Receivers: from-pipe recv selects on (pipe_fd, lifeline-or-wake-fd) for shutdown awareness | M | new probe per Slice E's original brief |
| 6 | SCORE doc + retire Slice C from active backlog (preserve INSCRIPTION) | S | grep `prctl|PDEATHSIG` src/ → only comments/docs remain |

Phases 1-4 retire Slice C cleanly. Phase 5 closes Slice E's original scope. Phase 6 is paperwork.

Each phase ships and verifies independently. Stepping stones per `feedback_iterative_complexity`.

## Cross-references

- `SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md` — empirical record this slice forwards from
- `SHUTDOWN-AWARE-CHANNELS-BACKLOG.md` — Slice E (now subsumed); Slice C (now retired by this slice)
- `SCORE-SLICE-C-PDEATHSIG.md` — historical record per `feedback_inscription_immutable`
- INTERSTITIAL § "Slice D surfaced Slice C as the deviation" — recursive substrate-teaching context
- INTERSTITIAL § "How the shadow channel fans out" — the architectural pattern this slice extends
- INTERSTITIAL § "Wat disciplines its own designers" session-catch #3 — the banked pushback this slice closes
- `feedback_inscription_immutable` — why Slice C's INSCRIPTION stays
- `tests/probe_lifeline_pipe_proof.rs` — 100/100 mechanism proof
