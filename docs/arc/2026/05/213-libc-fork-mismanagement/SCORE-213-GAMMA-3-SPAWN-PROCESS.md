# Arc 213 stone γ-3 — SCORE: migrate eval_kernel_spawn_process to spawn_lifelined

## Summary

Site migrated. `eval_kernel_spawn_process` (src/spawn_process.rs, fork call formerly at
line 167) replaced with `spawn_lifelined`-based pattern. Manual lifeline pipe creation
(Phase 1B/1D `make_pipe` + `as_raw_fd` + parent `drop(lifeline_r)`) removed —
`spawn_lifelined` owns the lifeline pipe atomically. `spawn_process_child_branch`
signature simplified: `lifeline_w: OwnedFd` param removed + `drop(lifeline_w)` at line
303 removed — spawn_lifelined drops the child's inherited lifeline_w internally before
the closure body runs. Phase 1D comment block updated to document that the "child is its
own lifeline keeper" discipline now lives at the spawn_lifelined level. Public signature
of `eval_kernel_spawn_process` UNCHANGED.

γ-3 completes arc 213's γ phase. After this stone, the substrate has ZERO bare
`libc::fork()` calls — only `spawn_lifelined`'s internal clone3 implementation in α.

## File changes

Single file modified: `src/spawn_process.rs` only.

### 1. Import additions (line 66)

`spawn_lifelined` added to the fork import group.
`FromRawFd` and `IntoRawFd` added to the `std::os::fd` import.
`Pidfd` and `LifelineWriter` NOT imported — the compiler infers return type of
`spawn_lifelined` without explicit name in scope; no spurious imports.

### 2. `eval_kernel_spawn_process` body (lines ~138-228 pre-migration)

**Removed:**
- `let (lifeline_r, lifeline_w) = make_pipe(...)` — manual lifeline pipe (Phase 1B)
- `let lifeline_r_raw = lifeline_r.as_raw_fd()` — raw fd capture for manual pipe
- Bare `libc::fork()` block with `if pid < 0` error path + `if pid == 0` child branch
- `drop(input_r)` / `drop(output_w)` / `drop(stderr_w)` — parent-side drops (replaced
  by `libc::close` on raw fd integers)
- `drop(lifeline_r)` — parent drop (spawn_lifelined handles internally)

**Added:**
- `input_r_raw` / `output_w_raw` / `stderr_w_raw` as `as_raw_fd()` captures (kept for
  child branch raw fd args; mirror of γ-2 pattern)
- `into_raw_fd()` for all six stdio OwnedFds before `spawn_lifelined` closure
- `spawn_lifelined(move |lifeline_r_raw: i32| { ... })` closure reconstructing OwnedFds
  + calling `spawn_process_child_branch` without `lifeline_w` arg
- Parent branch: `unsafe { libc::close(...) }` for three child-side fds
- Parent branch: `OwnedFd::from_raw_fd(...)` reconstruction for three parent-side fds
- `lifeline_writer.into_owned_fd()` for ChildHandleInner (γ-1 added this method)
- `pidfd.pid()` + `drop(pidfd)` (δ-deferral note; same pattern as γ-1/γ-2)
- `ChildHandleInner::new(pid, Some(lifeline_w))` — unchanged shape; lifeline_w now
  comes from LifelineWriter instead of manual make_pipe

### 3. `spawn_process_child_branch` signature simplification (line 277)

**Removed:**
- `lifeline_w: OwnedFd` parameter (was line 288)
- `drop(lifeline_w)` call (was line 303)

**Updated:**
- Phase 1D comment block (lines 296-302 pre-migration) — replaced with doc noting that
  spawn_lifelined (arc 213 α) owns the "child is its own lifeline keeper" discipline;
  no manual drop needed in spawn_process_child_branch

ONE caller updated mechanically (the new `spawn_lifelined` closure body in
`eval_kernel_spawn_process`).

## Verification: pre/post pass counts

| Test binary | Pre-γ-3 | Post-γ-3 |
|---|---|---|
| `probe_pidfd_primitive` (α regression) | 2/2 PASS | 2/2 PASS |
| `arc112_scheme_probe` | 1/1 PASS | 1/1 PASS |
| `arc112_slice2b_process_send_recv` | 1/1 PASS | 1/1 PASS |
| `probe_closure_body_prelude_lift` | 5/5 PASS | 5/5 PASS |
| `probe_counter_actor_process_diag` | 3/3 PASS | 3/3 PASS |
| `probe_declaration_form_lift` | 6/6 PASS | 6/6 PASS |
| `probe_def_not_special` | 5/5 PASS | 5/5 PASS |
| `probe_lifeline_orphan_clean_via_fork_program` | 1/1 PASS | 1/1 PASS |
| `probe_lifeline_orphan_clean_via_substrate` | 1/1 PASS | 1/1 PASS |
| `probe_pdeathsig_diagnostic` | 1/1 PASS | 1/1 PASS |
| `probe_pdeathsig_kills_orphan_child` | 1/1 PASS | 1/1 PASS |
| `probe_run_hermetic_no_deadlock` | 2/2 PASS | 2/2 PASS |
| `probe_spawn_process_parent_type` | 3/3 PASS | 3/3 PASS |
| `probe_spawn_process_stdin` | 1/1 PASS | 1/1 PASS |
| `probe_spawn_process_stdio` | 1/1 PASS | 1/1 PASS |
| `wat_arc170_program_contracts` | 24/24 PASS | 24/24 PASS |
| `wat_arc170_stone_a_drain_and_join` | 4/4 PASS | 4/4 PASS |
| `wat_arc208_process_io_result` | 7/7 PASS | 7/7 PASS |
| `wat_process_peer_ipc_round_trip` | 3/3 PASS | 3/3 PASS |
| `wat-cli wat_cli` | 15/15 PASS | 15/15 PASS |

**Total: 87/87 → 87/87. Zero regressions.**

## Subtleties

### No UnwindSafe complication (contrast γ-2)

γ-2 captured `Arc<dyn SourceLoader>` which required a local `wrap` module with a private
field to defeat Rust 2021 edition closure capture refinement. γ-3 captures `Vec<WatAST>`
and `Option<Config>` — both are plain data with no interior mutability. They auto-satisfy
`UnwindSafe` without any wrapper. Predicted in EXPECTATIONS and confirmed: the `LoaderWrap`
pattern does not recur.

### spawn_process_child_branch signature simplification — first in γ

γ-1 and γ-2 left their sister functions UNCHANGED. γ-3 is the first stone that simplifies
a sister function: `spawn_process_child_branch` loses the `lifeline_w: OwnedFd` parameter
because spawn_lifelined drops the child's inherited lifeline_w before the closure body
runs. One-caller mechanical update; compiler caught any inconsistency. No non-mechanical
complication surfaced.

### OwnedFd ownership across clone3

Identical to γ-1/γ-2 pattern: `into_raw_fd()` strips RAII wrappers before the closure.
Child reconstructs six OwnedFds (three stdio pairs) + one lifeline OwnedFd from
`lifeline_r_raw` (spawn_lifelined provides). Parent closes child-side fds via
`libc::close` + reconstructs parent-side OwnedFds via `from_raw_fd`.

`input_r_raw`, `output_w_raw`, `stderr_w_raw` are captured into the closure as `i32`
integers (via `as_raw_fd()` before `into_raw_fd()`). These are the raw-fd args to
`spawn_process_child_branch` (the dup2 targets). Captured as `i32` so they remain valid
after the `into_raw_fd()` calls (same pattern as γ-2).

### setpgid double-call

Same as γ-1/γ-2: `spawn_lifelined` calls `setpgid(0,0)` in the child before the closure
body; `child_post_fork_init` (called inside `spawn_process_child_branch`) also calls
`setpgid(0,0)`. Idempotent. No test regression. `probe_pdeathsig_*` both PASS; 
`wat_arc170_program_contracts` 24/24 confirms no pgroup interaction.

### spawn_lifelined drops parent's lifeline_r

`spawn_lifelined` drops the lifeline read-end on the parent side internally. The original
`drop(lifeline_r)` from Phase 1B's manual plumbing is removed — it is now redundant.

### pidfd dropped after pid extraction

Same δ-deferral as γ-1/γ-2: `pidfd.pid()` retrieves the pid; `pidfd` dropped immediately.
Stone δ migrates `ChildHandleInner` to hold a `Pidfd` instead of raw `pid_t`.

### Warning count

`cargo build --release` produces 5 pre-existing warnings, 0 errors. γ-2 had 6 warnings;
γ-3's import cleanup (no `LifelineWriter`/`Pidfd` import) eliminates 1 pre-existing
unused-import warning from spawn_process.rs's former import of `AsRawFd`-adjacent items.
No new warnings introduced.

## Expectations scorecard

| # | Criterion | Result |
|---|---|---|
| 1 | `eval_kernel_spawn_process` body migrated to spawn_lifelined | YES |
| 2 | Manual lifeline pipe creation removed (lines 162-163) | YES |
| 3 | Parent's `drop(lifeline_r)` removed (line 202) | YES |
| 4 | Bare `libc::fork()` block replaced with `spawn_lifelined` closure | YES |
| 5 | LifelineWriter::into_owned_fd used for lifeline_w extraction | YES |
| 6 | `spawn_process_child_branch` signature: `lifeline_w: OwnedFd` param REMOVED | YES |
| 7 | `spawn_process_child_branch` body: `drop(lifeline_w)` (line 303) REMOVED | YES |
| 8 | Phase 1D comment block updated (notes spawn_lifelined owns the discipline) | YES |
| 9 | Public signature of `eval_kernel_spawn_process` UNCHANGED | YES |
| 10 | cargo build --release clean | YES (5 pre-existing warnings, 0 errors) |
| 11 | α probe `probe_pidfd_primitive` still 2/2 PASS | YES |
| 12 | All 19 spawn-process test binaries: post-count == pre-count | YES |
| 13 | Zero modifications to `src/fork.rs` | YES |
| 14 | Zero modifications outside `src/spawn_process.rs` | YES |
| 15 | SCORE inscribes any closure-capture / sig-change subtleties + 20-binary count table | YES |

**All 15 criteria satisfied.**

## Mode classification

**Mode A.** Site migrated; `spawn_process_child_branch` signature simplified (lifeline_w
param removed, Phase 1D drop removed, comment updated); `cargo build --release` clean (5
pre-existing warnings, zero errors); all 87 baselines preserved (87/87 → 87/87); SCORE
written; mode-classified.

**γ phase COMPLETE.** After γ-3, the substrate has ZERO bare `libc::fork()` calls — only
`spawn_lifelined`'s internal clone3 implementation in α. δ/ε/ζ proceed.

No honest deltas requiring orchestrator review. Signature simplification was one-caller
mechanical. UnwindSafe complication (γ-2 Risk 2) did not recur. `probe_pdeathsig_*`
both PASS, confirming spawn_lifelined's setpgid + lifeline discipline is compatible with
the active lifeline probes.
