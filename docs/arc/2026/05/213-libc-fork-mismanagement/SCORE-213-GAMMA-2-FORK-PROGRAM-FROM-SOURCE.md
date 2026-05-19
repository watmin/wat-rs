# Arc 213 stone γ-2 — SCORE: migrate fork_program_from_source to spawn_lifelined

## Summary

Site migrated. `fork_program_from_source` (src/fork.rs, fork call formerly at line 968)
replaced with `spawn_lifelined`-based pattern. Manual lifeline pipe creation (Phase 1C
`make_pipe` + `as_raw_fd` + parent `drop(lifeline_r)`) removed — `spawn_lifelined`
owns the lifeline pipe atomically. `child_branch_from_source` UNCHANGED (already
had correct lifeline_r_raw + lifeline_r parameters from Phase 1C). Public signature
of `fork_program_from_source` UNCHANGED.

## File changes

Single file modified: `src/fork.rs` only.

### 1. `fork_program_from_source` body (lines ~947-1013 pre-migration, ~947-1060 post)

**Removed:**
- `stdin_r.as_raw_fd()` / `stdout_w.as_raw_fd()` / `stderr_w.as_raw_fd()` raw
  captures (three `as_raw_fd` calls that predated spawn_lifelined pattern)
- `let (lifeline_r, lifeline_w) = make_pipe(OP)?;` — manual lifeline pipe (Phase 1C)
- `let lifeline_r_raw = lifeline_r.as_raw_fd();` — raw fd capture for manual pipe
- Bare `libc::fork()` block (lines ~968-994) — the entire `if pid < 0` error path +
  `if pid == 0` child branch call
- `drop(stdin_r)` / `drop(stdout_w)` / `drop(stderr_w)` / `drop(lifeline_r)` —
  parent-side drops (replaced by manual libc::close + spawn_lifelined's internal
  lifeline_r drop)

**Added:**
- `into_raw_fd()` for all six stdio OwnedFds before spawn_lifelined closure
  (mirrors γ-1 pattern; clone3 inherits all fds; raw ints bypass Rust single-ownership)
- `owned_source` / `owned_canonical` string captures (moved before the closure;
  were already present in original but now explicitly named for clarity)
- `wrap` local module containing `LoaderWrap` — see UnwindSafe note below
- `spawn_lifelined(move |lifeline_r_raw: i32| { ... })` closure calling
  `child_branch_from_source` with reconstructed OwnedFds
- Parent branch: `unsafe { libc::close(...) }` for three child-side fds
- Parent branch: `OwnedFd::from_raw_fd(...)` reconstruction for three parent-side fds
- `lifeline_writer.into_owned_fd()` for ChildHandleInner (γ-1 added this method)
- `pidfd.pid()` + `drop(pidfd)` (same δ-deferral note as γ-1)
- `ChildHandleInner::new(pid, Some(lifeline_w))` — unchanged (already Some; γ-2
  preserves the lifeline_w that Phase 1C had wired)

## Verification: pre/post pass counts

| Test binary | Pre-γ-2 | Post-γ-2 |
|---|---|---|
| `probe_pidfd_primitive` (α regression) | 2/2 PASS | 2/2 PASS |
| `probe_lifeline_orphan_clean_via_fork_program` | 1/1 PASS | 1/1 PASS |
| `wat_arc170_stone_b_walker_collapse` | 4/4 PASS | 4/4 PASS |
| `wat_arc170_program_contracts` | 24/24 PASS | 24/24 PASS |
| `wat-cli wat_cli` | 15/15 PASS | 15/15 PASS |

**Total: 47/47 → 47/47. Zero regressions.**

## Subtleties

### UnwindSafe and Rust 2021 edition closure capture refinement

This was the primary friction surface (predicted as Risk 2 in EXPECTATIONS).

`Arc<dyn SourceLoader>` does not auto-implement `UnwindSafe` because `dyn SourceLoader`
lacks a `RefUnwindSafe` bound. `spawn_lifelined` requires `F: FnOnce(i32) + UnwindSafe`.

**First attempt:** `let loader = std::panic::AssertUnwindSafe(loader)` + `loader.0` inside
closure. Rejected by compiler. Root cause: Rust 2021 edition closure capture refinement
— the compiler analyzes the closure body and captures individual FIELDS rather than whole
bindings. `loader.0` inside the closure body causes the compiler to capture the inner
`Arc<dyn SourceLoader>` directly, bypassing the `AssertUnwindSafe` wrapper. The closure
struct field is `Arc<dyn SourceLoader>`, not `AssertUnwindSafe<Arc<dyn SourceLoader>>`.

**Second attempt (explicit `let` binding inside closure):** same result — field capture
happens at closure-construction analysis time, not at runtime.

**Final solution:** a local module `wrap` with a `struct LoaderWrap(Arc<dyn SourceLoader>)`
having a PRIVATE field. With a private `.0` field, the 2021 edition closure capture
refinement cannot look through the wrapper — the compiler must capture the whole
`LoaderWrap` value as an opaque unit. `LoaderWrap` explicitly implements
`UnwindSafe + RefUnwindSafe` (both are safe traits; no `unsafe` keyword needed). Inside
the closure body, `loader.into_inner()` extracts the `Arc<dyn SourceLoader>` at runtime
(after the closure type is fixed), passing it to `child_branch_from_source`.

Safety argument for `impl UnwindSafe for LoaderWrap`: `SourceLoader: Send + Sync`.
After clone3, parent and child have separate address spaces — no cross-unwind aliasing.
The loader is used only in the child (`startup_from_source` → `_exit`); no unwind
recovery path in the child processes this value. The assertion is sound.

γ-1 note contrast: γ-1 captured `Vec<WatAST>` + `Option<Config>` — both pure data with
no interior mutability, so they auto-satisfy `UnwindSafe` without any wrapper. γ-2's
`Arc<dyn SourceLoader>` is the first capture in the arc-213 chain that requires explicit
`UnwindSafe` attestation.

### OwnedFd ownership across clone3

Identical to γ-1 pattern: `into_raw_fd()` strips RAII wrappers before the closure.
Child reconstructs six OwnedFds (three stdio pairs) + one lifeline OwnedFd from
`lifeline_r_raw` (spawn_lifelined provides). Parent closes child-side fds via
`libc::close` + reconstructs parent-side OwnedFds via `from_raw_fd`.

### stdin_r_raw / stdout_w_raw / stderr_w_raw captures

Three raw fd `i32` values (`stdin_r_raw`, `stdout_w_raw`, `stderr_w_raw`) are captured
into the closure alongside the fd integers. These are used as the raw-fd arguments to
`child_branch_from_source` (the `dup2` targets inside the child). They are captured as
`i32` (not OwnedFd) so they remain valid after the `into_raw_fd()` calls.

### setpgid double-call

Same as γ-1: `spawn_lifelined` calls `setpgid(0,0)` in the child; `child_post_fork_init`
(called inside `child_branch_from_source`) also calls `setpgid(0,0)`. Idempotent.
No test regression. wat-cli 15/15 confirms no pgroup interaction issue.

### spawn_lifelined drops parent's lifeline_r

`spawn_lifelined` drops the lifeline read-end on the parent side internally. The explicit
`drop(lifeline_r)` from Phase 1C's manual lifeline plumbing is removed — it is redundant.

### pidfd dropped after pid extraction

Same δ-deferral as γ-1: `pidfd.pid()` retrieves the pid; `pidfd` dropped immediately.
Stone δ migrates `ChildHandleInner` to hold a `Pidfd` instead of raw `pid_t`.

## Expectations scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `fork_program_from_source` body migrated to spawn_lifelined | YES |
| 2 | Manual lifeline pipe creation removed (lines 957-958) | YES |
| 3 | Parent's `drop(lifeline_r)` removed (line 1005) | YES |
| 4 | Bare `libc::fork()` block replaced with `spawn_lifelined` closure | YES |
| 5 | LifelineWriter::into_owned_fd used for lifeline_w extraction | YES |
| 6 | `child_branch_from_source` UNCHANGED | YES |
| 7 | Public signature of `fork_program_from_source` UNCHANGED | YES |
| 8 | cargo build --release clean | YES (5 pre-existing warnings, 0 errors) |
| 9 | α probe `probe_pidfd_primitive` 2/2 PASS | YES |
| 10 | `probe_lifeline_orphan_clean_via_fork_program` 1/1 PASS | YES |
| 11 | `wat_arc170_stone_b_walker_collapse` 4/4 PASS | YES |
| 12 | `wat_arc170_program_contracts` 24/24 PASS | YES |
| 13 | `wat-cli wat_cli` 15/15 PASS | YES |
| 14 | Zero modifications outside `src/fork.rs` | YES |
| 15 | SCORE inscribes closure-capture / fd-ownership subtleties | YES |

**All 15 criteria satisfied.**

## Mode classification

**Mode A.** Site migrated; manual lifeline-pipe creation removed; `spawn_lifelined`
owns the lifeline atomically; `cargo build --release` clean (5 pre-existing warnings,
zero errors); all 47 baselines preserved (47/47 → 47/47); SCORE written.

One honest delta for orchestrator review: the `LoaderWrap` local-module pattern is
new (γ-1 didn't need it). The pattern is confined to `fork_program_from_source` body;
it is a local implementation detail and does not affect any public API or caller sites.
γ-3 (spawn_process.rs) does not capture `dyn SourceLoader` so this pattern should not
recur in γ-3.
