# Arc 170 FD-multiplex Phase 1B BRIEF — spawn-process lifeline pipe + PDEATHSIG retirement

**Phase:** 1B of DESIGN-FD-MULTIPLEX-SHUTDOWN.md.
**Predecessor:** Phase 1A shipped at `61217c7` (shutdown worker polls N FDs; `init_shutdown_signal_with_inputs(extra: &[i32])` exists, additive).
**Goal:** Replace `spawn_process_child_branch`'s `prctl(PR_SET_PDEATHSIG, SIGTERM)` mechanism with a lifeline pipe. Parent holds write-end (never writes); child registers read-end with the shutdown worker via `init_shutdown_signal_with_inputs`. When parent dies for any reason, kernel closes parent's FDs → child's lifeline read-end POLLHUP → worker wakes → cascade fires.

**Scope: spawn_process.rs ONLY.** Phase 1C closes the symmetric retirement in `fork.rs::child_branch_from_source`. Phase 1D adds the probe.

## Context (read before starting)

1. This BRIEF.
2. `docs/arc/2026/05/170-program-entry-points/DESIGN-FD-MULTIPLEX-SHUTDOWN.md` — full design + Phase composition.
3. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-D-LEAK-ZERO-VERIFICATION.md` — empirical record of the PDEATHSIG race (10% rate) and the lifeline proof (100/100).
4. `tests/probe_lifeline_pipe_proof.rs` — 100-trial pure-libc demo of the mechanism. Reference for the FD topology and POLLHUP semantics.
5. `src/runtime.rs:201` — `init_shutdown_signal` + `init_shutdown_signal_with_inputs` (Phase 1A). The worker spawned by these polls all input FDs; this BRIEF wires the lifeline read-fd in as one of those inputs.
6. `src/spawn_process.rs` — the file you're editing. Read `eval_kernel_spawn_process` (~line 75) and `spawn_process_child_branch` (~line 273) end-to-end before drafting edits.
7. `src/fork.rs:198` — `ChildHandleInner`. You'll add an `Option<OwnedFd>` field.
8. INTERSTITIAL § "Slice D surfaced Slice C as the deviation" — the rationale; read for voice. The lifeline isn't a clever new mechanism — it's restoring the substrate's existing pattern (piggyback on documented kernel invariant, same as crossbeam Sender::Drop). Slice C's PDEATHSIG was the deviation.

## Substrate edits

### 1. `src/fork.rs` — `ChildHandleInner` gains a lifeline write-end

Today:

```rust
pub struct ChildHandleInner {
    pub pid: libc::pid_t,
    pub reaped: AtomicBool,
    pub cached_exit: OnceLock<i64>,
}
```

Add a single field for the lifeline write-end:

```rust
pub struct ChildHandleInner {
    pub pid: libc::pid_t,
    pub reaped: AtomicBool,
    pub cached_exit: OnceLock<i64>,
    /// Arc 170 FD-multiplex — substrate-owned lifeline write-end.
    /// Parent holds this; never writes. When the parent process dies for
    /// any reason (clean exit / panic / SIGKILL / OOM-kill / segfault),
    /// the kernel closes all the parent's FDs as part of process
    /// teardown — including this one. The child's poll(2) over its
    /// lifeline read-end fires POLLHUP and the substrate shutdown
    /// cascade triggers.
    ///
    /// Wrapped in Option because tier-1 callers (Forked in-process by
    /// the legacy fork-program path before Phase 1C) may not yet plumb
    /// a lifeline. Once Phase 1C ships, fork-program-ast also wires
    /// one and this is always Some for forked children.
    pub lifeline_w: Option<std::os::fd::OwnedFd>,
}
```

Update `ChildHandleInner::new` to take a `lifeline_w: Option<OwnedFd>` parameter:

```rust
impl ChildHandleInner {
    pub fn new(pid: libc::pid_t, lifeline_w: Option<std::os::fd::OwnedFd>) -> Self {
        Self {
            pid,
            reaped: AtomicBool::new(false),
            cached_exit: OnceLock::new(),
            lifeline_w,
        }
    }
    // wait_or_cached unchanged
}
```

`Drop` is UNCHANGED — the OwnedFd's own Drop runs automatically after the explicit SIGKILL+waitpid steps complete (Rust drops fields in declaration order after the explicit `drop` body, which is empty here for the new field). Order: SIGKILL → waitpid → field drops → lifeline_w closes. Child is already dead by the time the lifeline closes; closing it is harmless cleanup.

Find ALL existing callers of `ChildHandleInner::new` and pass `None` for now (they get filled in for fork.rs in Phase 1C):

```
grep -nE "ChildHandleInner::new\b" src/ crates/
```

Expected sites:
- `src/spawn_process.rs:188` — currently `Arc::new(ChildHandleInner::new(pid))`. This becomes `Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)))` — see edit #3 below.
- Any others (likely in `src/fork.rs`): pass `None` for this BRIEF; Phase 1C wires them.

### 2. `src/spawn_process.rs` — create lifeline pipe before fork

In `eval_kernel_spawn_process`, FIND the existing pipe-creation block (around line 144–155):

```rust
let (input_r, input_w) = make_pipe(":wat::kernel::spawn-process")?;
let (output_r, output_w) = make_pipe(":wat::kernel::spawn-process")?;
let (stderr_r, stderr_w) = make_pipe(":wat::kernel::spawn-process")?;

let input_r_raw = input_r.as_raw_fd();
let output_w_raw = output_w.as_raw_fd();
let stderr_w_raw = stderr_w.as_raw_fd();
```

ADD a fourth pipe immediately AFTER stderr_w_raw and capture the read-end's raw fd:

```rust
// Arc 170 FD-multiplex — lifeline pipe.
// Parent holds lifeline_w; never writes. Child polls lifeline_r_raw
// via the shutdown worker (registered in spawn_process_child_branch
// below). When parent dies for any reason, kernel closes lifeline_w
// → child's poll fires POLLHUP → shutdown cascade.
let (lifeline_r, lifeline_w) = make_pipe(":wat::kernel::spawn-process")?;
let lifeline_r_raw = lifeline_r.as_raw_fd();
```

### 3. `src/spawn_process.rs` — fork branches handle the new pipe

In the CHILD branch (after `pid == 0` check), pass `lifeline_r_raw` into `spawn_process_child_branch`. In the PARENT branch (after fork returns), build the ChildHandleInner with `Some(lifeline_w)` and drop the parent's copy of `lifeline_r` (child holds it now).

Current child-branch call (around line 169–180):

```rust
if pid == 0 {
    spawn_process_child_branch(
        package,
        input_r_raw,
        output_w_raw,
        stderr_w_raw,
        (input_r, input_w),
        (output_r, output_w),
        (stderr_r, stderr_w),
    );
}
```

Becomes:

```rust
if pid == 0 {
    spawn_process_child_branch(
        package,
        input_r_raw,
        output_w_raw,
        stderr_w_raw,
        lifeline_r_raw,
        (input_r, input_w),
        (output_r, output_w),
        (stderr_r, stderr_w),
        lifeline_r,
    );
}
```

Current parent branch (around line 182–211):

```rust
// ── PARENT BRANCH ────────────────────────────────────────────
drop(input_r);
drop(output_w);
drop(stderr_w);

let handle = Arc::new(ChildHandleInner::new(pid));
// ...
```

Becomes:

```rust
// ── PARENT BRANCH ────────────────────────────────────────────
drop(input_r);
drop(output_w);
drop(stderr_w);
// Drop the parent's copy of lifeline_r — only the child holds the
// read-end now. The parent retains lifeline_w (held in ChildHandleInner
// below) until parent process death closes it.
drop(lifeline_r);

let handle = Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)));
// ...
```

### 4. `src/spawn_process.rs` — `spawn_process_child_branch` gains lifeline_r_raw

Signature change (around line 273):

```rust
fn spawn_process_child_branch(
    package: ClosurePackage,
    input_r_raw: i32,
    output_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    input_pair: (OwnedFd, OwnedFd),
    output_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> !
```

In the function body, drop `lifeline_r` is not needed — the OwnedFd's drop runs at end of scope automatically AFTER dup2 / shutdown-worker-registration are done.

Wait — actually the OwnedFd MUST stay alive past the call to `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` because the worker thread holds a raw int and reads from it. If `lifeline_r` is dropped, the read-fd is closed. **Solution:** call `std::mem::forget(lifeline_r)` after we register the raw fd with the worker — the OS now owns the FD via the worker thread. (Mirror of how stdio dup2 transfers ownership to the OS.)

Place inside the function body, AFTER the existing `install_silent_panic_hook()` call (around line 317) and AFTER `setpgid` (around line 321) and BEFORE the existing prctl block:

```rust
// Arc 170 FD-multiplex Phase 1B — register the lifeline read-end with
// the shutdown worker. The worker's poll(2) set grows by one FD; when
// the parent process dies for any reason, kernel closes the parent's
// lifeline write-end → this read-fd EOFs (POLLHUP) → worker wakes →
// trigger_shutdown → cascade unblocks all parked recvs.
//
// init_shutdown_signal_with_inputs is idempotent (OnceLock guard). The
// later call inside bootstrap_wat_vm_process becomes a no-op; the worker
// the bootstrap call would otherwise spawn is replaced by THIS one with
// the lifeline FD registered. Order matters: this MUST run before any
// later init_shutdown_signal() call.
crate::runtime::init_shutdown_signal_with_inputs(&[lifeline_r_raw]);

// Transfer FD ownership to the worker thread — the substrate now owns
// the lifeline read-fd. Dropping OwnedFd here would close the FD and
// the worker would immediately POLLHUP (false-positive shutdown).
std::mem::forget(lifeline_r);
```

DELETE the existing line at the OLD early-init position (the line that just calls `crate::runtime::init_shutdown_signal();` — around line 369). The new `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` above replaces it.

### 5. `src/spawn_process.rs` — REMOVE the prctl block entirely

Delete lines 332–361 (the entire `// Arc 170 Slice C — PR_SET_PDEATHSIG.` comment block + the `if unsafe { libc::prctl(...) }` block + its error path).

Keep `setpgid` (line 321 — unchanged).

After Phase 1B the child branch's order is:

```
dup2 stdio
install_silent_panic_hook
setpgid(0, 0)
init_shutdown_signal_with_inputs(&[lifeline_r_raw])  ← NEW (replaces both the old prctl block AND the old early init_shutdown_signal call)
mem::forget(lifeline_r)
install_substrate_signal_handlers
startup_from_forms
bootstrap_wat_vm_process (later init_shutdown_signal call is no-op via OnceLock guard)
eval main fn
```

### 6. `src/spawn_process.rs` — clean up the Slice C "early init" comment block

The block at ~line 363-368 that explains "Arc 170 Slice C — initialize the shutdown infrastructure BEFORE installing signal handlers" — this comment now describes the PDEATHSIG-race-closing motivation that no longer exists. Replace with a brief note pointing to the lifeline:

```rust
// Arc 170 FD-multiplex Phase 1B — shutdown infrastructure initialized
// above with the lifeline FD; signal handlers installed after so any
// SIGTERM/SIGINT route to the existing wake-pipe path.
```

## Scorecard (10 rows — sonnet provides evidence; orchestrator verifies)

| Row | What | Evidence |
|-----|------|----------|
| A | `ChildHandleInner` gains `lifeline_w: Option<OwnedFd>` field | `grep -n "lifeline_w" src/fork.rs` |
| B | `ChildHandleInner::new` signature updated; all callers pass `Some(lifeline_w)` (spawn_process) or `None` (others) | `grep -nE "ChildHandleInner::new\b" src/ crates/` shows all sites updated |
| C | `eval_kernel_spawn_process` creates a fourth `make_pipe` for the lifeline before fork | `grep -n "lifeline" src/spawn_process.rs` shows pipe creation |
| D | Parent branch drops `lifeline_r`, builds handle with `Some(lifeline_w)` | grep shows the parent branch passing Some(lifeline_w) to ChildHandleInner::new |
| E | `spawn_process_child_branch` signature includes `lifeline_r_raw: i32` + `lifeline_r: OwnedFd` parameters | function signature in src/spawn_process.rs |
| F | `spawn_process_child_branch` calls `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` (replacing the old bare `init_shutdown_signal()` call) | `grep -n "init_shutdown_signal_with_inputs\|init_shutdown_signal(" src/spawn_process.rs` |
| G | `mem::forget(lifeline_r)` after the registration call | grep shows the forget |
| H | The `prctl(PR_SET_PDEATHSIG, ...)` block + error path are GONE from `spawn_process.rs` | `grep -n "PR_SET_PDEATHSIG\|prctl" src/spawn_process.rs` shows NO matches in code (comments referencing the retirement are fine) |
| I | `cargo build --release --workspace --tests` passes clean | build output |
| J | `cargo test --release --test probe_shutdown_cascade_crossbeam` PASSES (Slice B cascade unchanged) AND `cargo test --release --test probe_lifeline_pipe_proof` PASSES 100/100 in isolation (mechanism proof unchanged) | test output |

## Verification — NOT in scope

- Running `probe_pdeathsig_kills_orphan_child` to verify the orphan-cleanup property survives the mechanism swap — that's Phase 1D's job (probe will need updates because it currently asserts via PDEATHSIG-cascade semantics, not lifeline semantics).
- Symmetric retirement in `fork.rs::child_branch_from_source` — Phase 1C.
- Stability-100 sweep — after Phase 1D's probe lands.

## Constraints

- NO Mutex / RwLock / CondVar additions.
- NO new wall-clock timers; no `recv_timeout`; no `nanosleep`.
- NO changes outside `src/spawn_process.rs` and `src/fork.rs` (only `ChildHandleInner` in fork.rs).
- DO NOT touch `fork.rs::child_branch_from_source` (separate phase).
- DO NOT touch `wat-cli` callers of `init_shutdown_signal` — they're cli-main; no lifeline applicable.
- DO NOT delete `tests/probe_pdeathsig_kills_orphan_child.rs` — it stays as historical regression marker; Phase 1D adjusts it if needed.
- Per `feedback_inscription_immutable`: do NOT edit Slice C's INSCRIPTION / SCORE doc / BRIEF.

## STOP-at-first-red

If you hit:
- `cargo build` fails after edits → STOP, report. Do NOT bandage; the substrate gap or BRIEF gap is the real signal.
- `probe_shutdown_cascade_crossbeam` fails → STOP. Slice B cascade is load-bearing; if the lifeline plumbing broke it, the BRIEF or implementation is wrong.
- ChildHandleInner callers turn out to be 3+ sites in unexpected places → STOP, surface the list. We may need to split this BRIEF.

## On completion

Write `SCORE-FD-MULTIPLEX-PHASE-1B-SPAWN-PROCESS-LIFELINE.md` as a sibling. 10 rows scored against the table above. Note any honest deltas (substrate surfaces discovered that the BRIEF missed; alternate edits made; what surprised you). Do NOT commit — orchestrator commits atomically after independent verification.
