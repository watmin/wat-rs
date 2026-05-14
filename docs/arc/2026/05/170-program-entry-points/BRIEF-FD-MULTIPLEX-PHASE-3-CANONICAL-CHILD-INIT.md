# Arc 170 FD-multiplex Phase 3 BRIEF — canonical child_post_fork_init helper

**Phase:** 3 of DESIGN-FD-MULTIPLEX-SHUTDOWN.md (substrate-imposed-not-followed applied to the substrate's own fork paths).
**Predecessors:** Phases 1A–1E + 2 shipped at `61217c7..6062cfc`. Workspace test shows two new failures (`stream_test_with_state_buffer_all_at_eos`, `lru_test_local_cache_put_then_get`) — pass in isolation, fail under workspace pressure. Root cause: FD pressure from new pipes (broadcast + lifeline) accumulating because **`spawn_process_child_branch` does NOT call `close_inherited_fds_above_stdio`** — symmetric defect to Phase 1E's fork.rs work, scoped out then as "separate hygiene concern."

**Goal:** Eliminate the symptom (FD leak in spawn-process children) AND eliminate the discipline trap (two fork paths can drift on the post-fork sequence). Extract the canonical 5-step post-fork sequence into ONE substrate-owned helper that both `spawn_process_child_branch` and `child_branch_from_source` call. The duplicated sequence collapses to one function call per fork path. Forgetting becomes structurally impossible.

## The discipline this enforces

Same pattern as the rest of arc 170's substrate-imposed-not-followed work:

| Discipline applied to | API surface | Way to deviate |
|---|---|---|
| shutdown observation | `typed_recv` only | no `recv-without-shutdown` exists |
| parent-death detection | lifeline EOF only | no way to "skip the lifeline" |
| panic propagation | Result + expect only | no silent recv allowed |
| **post-fork init (this phase)** | `child_post_fork_init(lifeline_r_raw)` only | no way to "do steps differently" |

After this phase, the post-fork canonical sequence lives in ONE function. Both modern fork paths (spawn-process + fork-program-from-source) call it. Future fork paths (arc 191 `exec-program`, future remote-spawn) call it. The discipline IS the function call.

## The canonical sequence (5 steps)

| # | Step | Why |
|---|------|-----|
| 1 | `install_silent_panic_hook()` | Suppress Rust panic stderr; substrate emits structured ProcessPanics |
| 2 | `setpgid(0, 0)` w/ structured-exit on failure | Make child its own pgrp leader (arc 106 signal cascade) |
| 3 | `close_inherited_fds_above_stdio(&[lifeline_r_raw])` | FD hygiene: close inherited parent FDs; skip lifeline read-end |
| 4 | `init_shutdown_signal_with_inputs(&[lifeline_r_raw])` | Substrate's shutdown infrastructure; lifeline + broadcast pipes |
| 5 | `install_substrate_signal_handlers()` | SIGTERM/SIGINT/SIGUSR1/2/SIGHUP wired to substrate handlers |

Both fork paths currently do steps 1, 2, 4, 5 identically. Only step 3 diverges (fork-program calls it; spawn-process doesn't). After Phase 3, both paths call ONE helper that does all 5 steps in order; step 3 is no longer optional.

## Substrate edits

### 1. NEW `pub(crate) fn child_post_fork_init(lifeline_r_raw: i32)` in `src/fork.rs`

Lives in `src/fork.rs` because fork.rs is the canonical home for child-fork mechanics. Both child branches already live there or adjacent (spawn_process.rs imports from fork.rs already — `install_substrate_signal_handlers`, `make_pipe`, `ChildHandleInner`). Make it `pub(crate)` so spawn_process.rs can call it.

```rust
/// Canonical post-fork initialization for substrate-spawned wat-vm
/// children. Both fork paths (`fork_program_from_source` ::
/// `child_branch_from_source` and `spawn_process` ::
/// `spawn_process_child_branch`) call this immediately after
/// finishing their pipe-specific dup2 + drop work and before any
/// substrate startup/eval.
///
/// The 5-step canonical sequence:
///
/// 1. Install the silent panic hook (substrate's structured-stderr
///    emit owns panic propagation; Rust's default panic output is
///    suppressed).
/// 2. Make the child its own process-group leader (arc 106 signal
///    cascade discipline). Structured-stderr + `_exit` on failure.
/// 3. Close inherited FDs above stdio (FD hygiene). The substrate-
///    owned lifeline read-end is in the skip-list so it survives
///    for the shutdown worker.
/// 4. Initialize the shutdown infrastructure with the lifeline FD
///    registered. Opens wake-pipe + broadcast pipe; spawns worker
///    polling (wake_pipe_read, lifeline_r_raw) and holding
///    broadcast_w.
/// 5. Install substrate signal handlers (SIGTERM/SIGINT/SIGUSR1/2/
///    SIGHUP) wired through the wake-pipe to the shutdown cascade.
///
/// On any failure inside, emits structured ProcessPanics on fd 2 and
/// `_exit(EXIT_STARTUP_ERROR)`. Never returns to caller; either
/// completes all 5 steps or terminates the child.
///
/// `mem::forget(lifeline_r)` stays in the CALLER's scope (transfer of
/// OwnedFd ownership to the substrate worker via the raw fd; the
/// OwnedFd value's drop must not run, but the function takes only
/// the raw fd, so the caller is the one with the OwnedFd in scope).
pub(crate) fn child_post_fork_init(lifeline_r_raw: i32) {
    // Step 1
    install_silent_panic_hook();

    // Step 2
    if unsafe { libc::setpgid(0, 0) } < 0 {
        let err = std::io::Error::last_os_error();
        emit_structured_exit(
            None,
            crate::runtime::process_died_error_startup_value(
                format!("setpgid(0, 0) failed: {}", err),
            ),
        );
        unsafe { libc::_exit(EXIT_STARTUP_ERROR) };
    }

    // Step 3
    close_inherited_fds_above_stdio(&[lifeline_r_raw]);

    // Step 4
    crate::runtime::init_shutdown_signal_with_inputs(&[lifeline_r_raw]);

    // Step 5
    install_substrate_signal_handlers();
}
```

Place near `install_silent_panic_hook` definition (~line 444 in src/fork.rs) so reads logically group.

### 2. `src/fork.rs::child_branch_from_source` collapses 5 steps to 1 call

Replace the inline sequence (current lines ~1049-1120 approximately — install_silent_panic_hook through install_substrate_signal_handlers, including the close_inherited_fds + init_shutdown_signal_with_inputs + mem::forget):

```rust
// (after the pipe-specific drop sequence ending around line 1046)
child_post_fork_init(lifeline_r_raw);
std::mem::forget(lifeline_r);

// (continue with startup_from_source...)
```

The `mem::forget` stays in the caller — it's the OwnedFd's ownership transfer, scope-bound.

### 3. `src/spawn_process.rs::spawn_process_child_branch` collapses 4 steps to 1 call + gains step 3

Replace the inline sequence (current lines ~317-374 — install_silent_panic_hook + setpgid + init_shutdown_signal + install_substrate_signal_handlers; MISSING close_inherited_fds today):

```rust
// (after the pipe-specific drop sequence ending around line 312, after drop(lifeline_w))
crate::fork::child_post_fork_init(lifeline_r_raw);
std::mem::forget(lifeline_r);

// (continue with startup_from_forms...)
```

The Phase 1D `drop(lifeline_w)` stays in spawn_process — it's spawn-process-specific (fork-program's lifeline_w is held in the parent's ChildHandleInner only and is correctly inherited+closed by the close-sweep in step 3 of the helper).

### 4. Verify legacy `child_branch` (forms-based, line ~634 in fork.rs) is NOT a caller

`child_branch` is the legacy fork-program-ast path that doesn't have setpgid and doesn't have lifeline. Per Slice C scoping it stays as-is. Confirm it does NOT call `child_post_fork_init`. Its existing close_inherited_fds_above_stdio(&[]) call stays unchanged.

## Scorecard (10 rows)

| Row | What | Evidence |
|-----|------|----------|
| A | `pub(crate) fn child_post_fork_init(lifeline_r_raw: i32)` exists in src/fork.rs | `grep -n "fn child_post_fork_init" src/fork.rs` |
| B | child_post_fork_init body contains all 5 canonical steps in order | read the function body; confirm install_silent_panic_hook → setpgid → close_inherited_fds → init_shutdown_signal_with_inputs → install_substrate_signal_handlers |
| C | `child_branch_from_source` calls `child_post_fork_init(lifeline_r_raw)` instead of inline sequence | `grep -nE "child_post_fork_init\|install_silent_panic_hook\|close_inherited_fds_above_stdio" src/fork.rs` — install_silent_panic_hook + close_inherited_fds calls appear ONLY inside child_post_fork_init body (and child_post_fork_init's call site at child_branch_from_source); not inline at the child_branch_from_source body anymore |
| D | `spawn_process_child_branch` calls `crate::fork::child_post_fork_init(lifeline_r_raw)` instead of inline sequence | `grep -nE "child_post_fork_init\|install_silent_panic_hook\|setpgid" src/spawn_process.rs` — install_silent_panic_hook + setpgid no longer appear inline in spawn_process_child_branch |
| E | `mem::forget(lifeline_r)` remains in both callers' scope (NOT inside child_post_fork_init) | `grep -n "mem::forget" src/spawn_process.rs src/fork.rs` shows the forget at caller scope |
| F | Legacy `child_branch` (fork.rs:~634) is UNCHANGED — still calls `close_inherited_fds_above_stdio(&[])` and does not call child_post_fork_init | grep + read |
| G | `cargo build --release --workspace --tests` clean | build output |
| H | All 6 probes PASS in isolation (no regression) | invoke each via cargo test --release --test \<name\>; show 1/1 PASS for each |
| I | The 2 pressure-induced failures (stream + lru) now PASS under workspace pressure | `cargo test --release --workspace --no-fail-fast` shows neither in the failure set (failure count drops from 11 to ≤ 9; lifeline flake may persist independently — that's a separate concern outside Phase 3 scope) |
| J | The 9 pre-existing failures (svc-test, tmp, lifeline flake, wat-cli) are UNCHANGED — Phase 3 doesn't touch them; they fail with the same diagnostic | compare failure set to baseline pre-FD-multiplex (in `/tmp/baseline-pre-fdmpx.log`) |

## Constraints

- NO Mutex / RwLock / CondVar.
- NO new wall-clock timers; libc::poll(2) only.
- NO changes outside src/fork.rs and src/spawn_process.rs.
- DO NOT touch existing probe files.
- DO NOT modify the legacy `child_branch` path.
- DO NOT change the canonical sequence's contents — only extract them. The 5 steps are already verified correct; this phase moves them to one place.
- Per `feedback_inscription_immutable`: do NOT edit Slice C's INSCRIPTION / SCORE doc / BRIEF.

## STOP-at-first-red

- `cargo build` fails → STOP, report. Likely cause: visibility (pub(crate) needed across modules) or function-name collision.
- Any existing probe fails in isolation → STOP. The extraction was supposed to be behavior-preserving; if a probe regresses, the sequence was reordered incorrectly.
- Row I (stream + lru tests under workspace pressure) STILL FAIL → the FD-pressure hypothesis was wrong; surface as honest delta + STOP. We need to re-diagnose before continuing.

## On completion

Write `SCORE-FD-MULTIPLEX-PHASE-3-CANONICAL-CHILD-INIT.md`. 10 rows. Note honest deltas — especially: the workspace test result (this is the meaningful empirical signal for Row I).

Do NOT commit. Orchestrator commits atomically after independent verification.
