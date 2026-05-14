# Arc 170 FD-multiplex Phase 1C BRIEF — fork-program lifeline + PDEATHSIG retirement

**Phase:** 1C of DESIGN-FD-MULTIPLEX-SHUTDOWN.md. Symmetric to Phase 1B applied to `src/fork.rs::fork_program_from_source` + `child_branch_from_source`.
**Predecessor:** Phase 1B shipped at `8714a6f` (spawn_process.rs PDEATHSIG retired; lifeline mechanism in place via ChildHandleInner.lifeline_w + init_shutdown_signal_with_inputs).
**Goal:** Replace `child_branch_from_source`'s `prctl(PR_SET_PDEATHSIG, SIGTERM)` mechanism with the lifeline pipe. Same shape as Phase 1B. ChildHandleInner already carries `lifeline_w: Option<OwnedFd>` (Phase 1B); this BRIEF wires `Some(lifeline_w)` at the fork-program-from-source site (currently `None`).

**Scope: src/fork.rs ONLY.** Edits are confined to `fork_program_from_source` (line ~829) and `child_branch_from_source` (line ~1015). The legacy `child_branch` path (line ~634) was explicitly out-of-scope in Slice C (no setpgid; arc 106 discipline not applied); its ChildHandleInner site at fork.rs:591 stays `None` per the same scoping.

## Context (read before starting)

1. This BRIEF.
2. `docs/arc/2026/05/170-program-entry-points/SCORE-FD-MULTIPLEX-PHASE-1B-SPAWN-PROCESS-LIFELINE.md` — what shipped in 1B. The pattern you're mirroring.
3. `docs/arc/2026/05/170-program-entry-points/DESIGN-FD-MULTIPLEX-SHUTDOWN.md` — full design.
4. `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-C-PDEATHSIG.md` — explicitly scoped out `child_branch` (legacy forms-based path); same scoping holds here.
5. `src/runtime.rs:201` — `init_shutdown_signal_with_inputs(extra: &[i32])` shipped in Phase 1A.
6. `src/fork.rs:198` — `ChildHandleInner` struct (Phase 1B added `lifeline_w: Option<OwnedFd>` field; constructor takes it).
7. `src/spawn_process.rs:140-220` — the analogous edit pattern (Phase 1B's eval_kernel_spawn_process + parent-branch logic). Read for mechanical reference.
8. `src/spawn_process.rs:280-370` — Phase 1B's `spawn_process_child_branch`. Mirror its lifeline-registration + `mem::forget` pattern in `child_branch_from_source`.

## Substrate edits

### 1. `src/fork.rs::fork_program_from_source` — create lifeline pipe before fork

Today (around lines 838–855):

```rust
let (stdin_r, stdin_w) = make_pipe(OP)?;
let (stdout_r, stdout_w) = make_pipe(OP)?;
let (stderr_r, stderr_w) = make_pipe(OP)?;

let stdin_r_raw = stdin_r.as_raw_fd();
let stdout_w_raw = stdout_w.as_raw_fd();
let stderr_w_raw = stderr_w.as_raw_fd();

let owned_source = source.to_string();
let owned_canonical = canonical.map(|s| s.to_string());

let pid = unsafe { libc::fork() };
```

ADD a fourth pipe AFTER stderr_w_raw and capture lifeline_r_raw:

```rust
// Arc 170 FD-multiplex Phase 1C — lifeline pipe.
// Parent holds lifeline_w; never writes. Child polls lifeline_r_raw via
// the shutdown worker (registered in child_branch_from_source below).
// When parent dies for any reason, kernel closes lifeline_w → child's
// poll fires POLLHUP → shutdown cascade. Same pattern as spawn-process
// (Phase 1B; see src/spawn_process.rs).
let (lifeline_r, lifeline_w) = make_pipe(OP)?;
let lifeline_r_raw = lifeline_r.as_raw_fd();
```

### 2. `src/fork.rs::fork_program_from_source` — child branch call gains lifeline args

Today (lines 866–880):

```rust
if pid == 0 {
    child_branch_from_source(
        owned_source,
        owned_canonical,
        loader,
        argv,
        stdin_r_raw,
        stdout_w_raw,
        stderr_w_raw,
        (stdin_r, stdin_w),
        (stdout_r, stdout_w),
        (stderr_r, stderr_w),
    );
}
```

Becomes:

```rust
if pid == 0 {
    child_branch_from_source(
        owned_source,
        owned_canonical,
        loader,
        argv,
        stdin_r_raw,
        stdout_w_raw,
        stderr_w_raw,
        lifeline_r_raw,
        (stdin_r, stdin_w),
        (stdout_r, stdout_w),
        (stderr_r, stderr_w),
        lifeline_r,
    );
}
```

### 3. `src/fork.rs::fork_program_from_source` — parent branch drops lifeline_r, builds handle with Some(lifeline_w)

Today (lines 882–893):

```rust
// ── PARENT BRANCH ────────────────────────────────────────────
drop(stdin_r);
drop(stdout_w);
drop(stderr_w);

Ok(ForkedProgramHandles {
    child_handle: Arc::new(ChildHandleInner::new(pid, None)),
    stdin_w,
    stdout_r,
    stderr_r,
})
```

Becomes:

```rust
// ── PARENT BRANCH ────────────────────────────────────────────
drop(stdin_r);
drop(stdout_w);
drop(stderr_w);
// Drop the parent's copy of lifeline_r — only the child holds the
// read-end now. The parent retains lifeline_w (held in ChildHandleInner
// below) until parent process death closes it.
drop(lifeline_r);

Ok(ForkedProgramHandles {
    child_handle: Arc::new(ChildHandleInner::new(pid, Some(lifeline_w))),
    stdin_w,
    stdout_r,
    stderr_r,
})
```

### 4. `src/fork.rs::child_branch_from_source` — signature + body

Signature change (line ~1015):

```rust
fn child_branch_from_source(
    source: String,
    canonical: Option<String>,
    loader: Arc<dyn SourceLoader>,
    argv: Vec<String>,
    stdin_r_raw: i32,
    stdout_w_raw: i32,
    stderr_w_raw: i32,
    lifeline_r_raw: i32,
    stdin_pair: (OwnedFd, OwnedFd),
    stdout_pair: (OwnedFd, OwnedFd),
    stderr_pair: (OwnedFd, OwnedFd),
    lifeline_r: OwnedFd,
) -> !
```

In the body, at the EXISTING `init_shutdown_signal()` call site (line ~1105), REPLACE the bare call with the with-inputs variant + `mem::forget`:

```rust
// Arc 170 FD-multiplex Phase 1C — register the lifeline read-end with
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

Update the comment immediately above (the "Arc 170 Slice C — initialize the shutdown infrastructure BEFORE..." block at lines 1099–1104) — replace with the Phase 1B-style framing already used in spawn_process.rs:

```rust
// Arc 170 FD-multiplex Phase 1C — shutdown infrastructure initialized
// above with the lifeline FD; signal handlers installed after so any
// SIGTERM/SIGINT route to the existing wake-pipe path.
```

### 5. `src/fork.rs::child_branch_from_source` — DELETE the prctl block

Lines 1074–1097 (the entire `// Arc 170 Slice C — PR_SET_PDEATHSIG.` comment + `if unsafe { libc::prctl(...) }` block + its error path).

Keep `setpgid` (line 1063 — unchanged).

After Phase 1C the child branch's order is:

```
drop parent-side pipes
dup2 stdio
install_silent_panic_hook
setpgid(0, 0)
init_shutdown_signal_with_inputs(&[lifeline_r_raw])  ← NEW (replaces prctl + bare init)
mem::forget(lifeline_r)
install_substrate_signal_handlers
close_inherited_fds_above_stdio  ← UNCHANGED, post-handler-install
startup_from_source
... rest of child runtime ...
```

## Scorecard (10 rows — sonnet provides evidence; orchestrator verifies)

| Row | What | Evidence |
|-----|------|----------|
| A | `fork_program_from_source` creates fourth `make_pipe` for lifeline before fork | `grep -n "lifeline" src/fork.rs` shows pipe creation + `lifeline_r_raw` |
| B | `fork_program_from_source` parent branch drops `lifeline_r` and builds handle with `Some(lifeline_w)` | grep shows parent branch passing Some(lifeline_w); `drop(lifeline_r)` immediately before |
| C | `fork_program_from_source` child branch call passes `lifeline_r_raw` + `lifeline_r` to `child_branch_from_source` | grep shows call with new args |
| D | `child_branch_from_source` signature gains `lifeline_r_raw: i32` + `lifeline_r: OwnedFd` parameters | `awk '/^fn child_branch_from_source/,/^) -> !/' src/fork.rs` shows new params |
| E | `child_branch_from_source` calls `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` (replacing the old bare `init_shutdown_signal()` call) | `grep -n "init_shutdown_signal" src/fork.rs` — the only matches in child_branch_from_source are the with_inputs variant + comments; zero bare `init_shutdown_signal()` calls |
| F | `mem::forget(lifeline_r)` after the registration call | `grep -n "mem::forget" src/fork.rs` shows the forget |
| G | The `prctl(PR_SET_PDEATHSIG, ...)` block + error path are GONE from `child_branch_from_source` | `grep -nE "PR_SET_PDEATHSIG\|prctl" src/fork.rs` shows NO matches in code (comments referencing the retirement are allowed) |
| H | `child_branch` (legacy forms-based path) and its `ChildHandleInner::new(pid, None)` site UNCHANGED | `grep -n "ChildHandleInner::new" src/fork.rs` shows the line 591 site STILL with `None`; only line 889 (the fork-program-from-source site) changes to `Some(lifeline_w)` |
| I | `cargo build --release --workspace --tests` passes clean | build output |
| J | `probe_shutdown_cascade_crossbeam` PASSES AND `probe_lifeline_pipe_proof` PASSES 100/100 in isolation | both test invocations show `1 passed; 0 failed` |

## Verification — NOT in scope

- Probe that exercises the new fork-program-from-source lifeline mechanism end-to-end — that's Phase 1D's job (combined with the spawn-process verification probe).
- The legacy `child_branch` path retirement / lifeline plumbing — explicitly out of scope per Slice C scoping.

## Constraints

- NO Mutex / RwLock / CondVar.
- NO new wall-clock timers; no recv_timeout; no nanosleep.
- NO changes outside `src/fork.rs`.
- DO NOT touch the legacy `child_branch` path (lines 634–~800).
- DO NOT touch the `ChildHandleInner::new(pid, None)` site at line 591 — that's the legacy path.
- DO NOT delete `tests/probe_pdeathsig_kills_orphan_child.rs`.
- Per `feedback_inscription_immutable`: do NOT edit Slice C's INSCRIPTION / SCORE doc / BRIEF.

## STOP-at-first-red

If you hit:
- `cargo build` fails after edits → STOP, report.
- `probe_shutdown_cascade_crossbeam` fails → STOP. Slice B cascade is load-bearing.
- Discovering that the legacy `child_branch` path SHOULD ALSO get the lifeline (e.g., it has setpgid we missed) → STOP, report. Don't expand scope mid-BRIEF.

## On completion

Write `SCORE-FD-MULTIPLEX-PHASE-1C-FORK-PROGRAM-LIFELINE.md` as a sibling. 10 rows scored. Note any honest deltas. Do NOT commit — orchestrator commits atomically after independent verification.
