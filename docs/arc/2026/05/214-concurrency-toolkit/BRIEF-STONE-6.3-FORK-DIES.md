# BRIEF — Stone 6.3: fork.rs dies — the process family rehomes to src/process/

> Builder's call: "fork.rs dies in this marathon." The home name AND the full
> internal layout were RATIFIED BY THE INTUERI MINT CAST (the cast names —
> neither builder nor orchestrator; the rule is now memory). Your job is the
> VERBATIM execution of the cast's map. fork.rs is the most dangerous file in
> the tree (clone3, signals, child-context fds) — the lift is move-only.

## Required reading (in order)
1. This brief's placement table (the cast's verdict — execute it exactly).
2. `src/fork.rs` whole (1846), `src/spawn.rs` (342), `src/spawn_process.rs`
   (486), `src/process_stdio.rs` (141).
3. `src/services/mod.rs` + `src/channel/mod.rs` — the index house style.

## The gate (committed, RED at HEAD)
`tests/nursery/probe_arc214_stone63_fork_dead.rs` — all four flat files gone
+ zero `fork::`/`spawn_process::` paths (self-excluding scanner).

## THE RATIFIED LAYOUT (the cast's map — execute verbatim)

**`src/process/mod.rs`** — index + module doc + flat pub-use re-exports
(services/channel house style). The doc names the TWO TIERS explicitly:
OS-process tier (clone3/pidfd/lifelines) and in-thread tier (std::thread
over kernel pipes) — both producing the ONE `:wat::kernel::Process` value
shape (stdin IOWriter / stdout+stderr IOReader / join handle). That shape is
the home's unifying noun (the cast's keystone).

**`src/process/clone.rs`** — the Linux process-creation primitives:
`CloneArgs`, `ExitStatus`, `Pidfd` (+impls), `LifelineWriter`,
`spawn_lifelined`, `spawn_lifelined_any`, `make_pipe`,
`extract_exit_status_from_siginfo` (private).

**`src/process/child.rs`** — the child-side envelope (post-clone3, pre-user
code): `install_substrate_signal_handlers` + the four `extern "C"` handlers,
`run_in_fork`, `child_post_fork_init`, `child_post_fork_init_preserving`,
`close_inherited_fds_above_stdio` (private), `install_silent_panic_hook`
(private), `emit_structured_exit` (private), `emit_panics_to_stderr`
(private), `child_branch` (private), `child_branch_from_source` (private).

**`src/process/handle.rs`** — the parent-side handles: `ChildHandleInner`
(+new, wait_or_cached, Drop), `extract_exit_code` (private),
`ForkedProgramHandles`.

**`src/process/verbs.rs`** — the wat dispatch arms + their helpers:
the EXIT_* constants, `eval_kernel_fork_program_ast`,
`eval_kernel_fork_program`, `fork_program_from_source`,
`eval_kernel_spawn_process`, `spawn_process_child_branch` (private),
`eval_kernel_spawn_program`, `eval_kernel_spawn_program_ast`,
`spawn_with_world_into_result` (private), `startup_error_result` (private),
`arity_2`/`expect_string`/`expect_option_string`/`expect_vec_ast` (private).
NOTE: spawn_process.rs's own copies of `emit_structured_exit` +
`emit_panics_to_stderr` are DUPLICATES of fork.rs's — the cast's ruling:
**keep both copies verbatim in this lift** (fork-safety; the merge is 6.w's).
Place spawn_process.rs's copies as privates in verbs.rs.

**`src/process/stdio.rs`** — verbatim lift of `src/process_stdio.rs`
(`lend_ambient`, `emit_panic_envelope`).

## The kill + sweep
- `git rm`-equivalent: all four flat files end deleted (use git mv where a
  file maps mostly to one destination — fork.rs does NOT, it splits; plain
  create+delete is fine, git will detect what it detects).
- `src/lib.rs`: `pub mod fork/spawn/spawn_process/process_stdio` die;
  `pub mod process;` + re-export repoints.
- Sweep every `crate::fork::`/`wat::fork::`/`crate::spawn_process::`/
  `wat::spawn_process::` (and plain `crate::spawn::`/`crate::process_stdio::`
  — grep for them) → `crate::process::`/`wat::process::`. Consumers from the
  6.1/6.2 era: runtime.rs, check.rs?, comms/process.rs, freeze.rs, services/,
  kernel/spawn.rs, value/value.rs + the test tree (MiniUniverse uses
  `wat::fork::make_pipe`!). `cargo check --all-targets` + the gate scanner
  are the completeness checks.

## HARD CONSTRAINTS (fork-safety)
- VERBATIM moves: no logic edits, no renames, no merges (the duplicate
  bodies STAY duplicated), no signature changes. Doc comments travel with
  their items. The cast's NAMING-DEBTS (the fork_* clone3 lie,
  ChildHandleInner's phantom outer, ForkedProgramHandles) are 6.w's — do
  not fix them here.
- Nothing new lands in src/channel/ or src/services/ beyond path-sweep
  lines if any reference the old modules.

## Gates
1. Gate-probe 63 → 2/2 GREEN.
2. `cargo test --release --lib -p wat` → 943/0/1.
3. `cargo test --release --test nursery` → 865/4/4 (the 4 known parked;
   your gate +2).
4. `cargo test --release --test wat_arc170_slice_1f_alpha_helpers` → 12/0/0.
5. Process-tier binaries, ENVELOPED (`setsid timeout 120 cargo test
   --release --test <bin> -- --test-threads=1`): `wat_arc170_channel_pipes`
   (23/0), `wat_arc170_slice_1f_gamma_orchestrator` (5/0),
   `probe_run_hermetic_no_deadlock` (2/0), `comms` (all green).
6. `cargo check --all-targets` → 0 errors.
7. `cargo clippy --release --lib -p wat` → zero findings in src/process/.

## STOP triggers (rejection criteria)
- STOP-1: any move that cannot be verbatim (a private fn needed by two
  destination files forcing a visibility change beyond pub(crate)/pub(super)
  — pub(crate) IS allowed where the split demands it; report each such
  visibility widening in your deltas).
- STOP-2: an enveloped process test fails or HANGS (kill it via the
  timeout; report which and the output) — fork-context breakage is the
  one risk this stone carries; ship nothing.
- STOP-3: untraceable red outside the known baseline.

## Constraints
- Commit NOTHING — the orchestrator scores and commits.
- Probe files are read-only ground truth.
- Work only in /home/watmin/work/holon/wat-rs/.
