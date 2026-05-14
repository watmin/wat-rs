# Arc 170 FD-multiplex Phase 3 EXPECTATIONS

**BRIEF:** `BRIEF-FD-MULTIPLEX-PHASE-3-CANONICAL-CHILD-INIT.md`

## Independent prediction

**Runtime band:** 10–15 minutes Mode A.

Reasoning:
- Pure refactor — extract 5 lines from two functions into one. No new logic.
- One small new fn (~30 lines including doc comment) in src/fork.rs.
- Two call-site replacements (collapse 5 inline lines → 1 fn call) in src/fork.rs + src/spawn_process.rs.
- Workspace test rerun to verify Row I — adds ~3-5 min wall-clock.

**Time-box:** ScheduleWakeup at 30 minutes (2× upper-bound).

## SCORE methodology

Each row YES/NO with evidence per `feedback_four_questions_yes_no`:

- **Row A**: `grep -n "fn child_post_fork_init" src/fork.rs` returns 1 line.
- **Row B**: Read the function body; verify all 5 steps in order. Visual confirmation.
- **Row C**: `grep -nE "install_silent_panic_hook|close_inherited_fds_above_stdio" src/fork.rs` — install_silent_panic_hook appears in (a) its own definition and (b) the child_post_fork_init body; close_inherited_fds_above_stdio appears in (a) its own definition and (b) the child_post_fork_init body AND (c) the legacy child_branch call site. NEITHER appears inline in child_branch_from_source's body.
- **Row D**: Same shape for spawn_process.rs — install_silent_panic_hook + setpgid no longer appear inline in spawn_process_child_branch. child_post_fork_init call appears.
- **Row E**: `grep -n "mem::forget" src/spawn_process.rs src/fork.rs` — both call sites show forget AT CALLER scope (just after the child_post_fork_init call).
- **Row F**: `grep -n "ChildHandleInner::new\|close_inherited_fds_above_stdio" src/fork.rs` — line 591 (legacy ChildHandleInner::new(pid, None)) UNCHANGED; line ~672 (legacy child_branch's close_inherited_fds_above_stdio(&[])) UNCHANGED.
- **Row G**: `cargo build --release --workspace --tests 2>&1 | tail -3` shows Finished, zero errors.
- **Row H**: Each probe invoked separately:
  - probe_shutdown_cascade_crossbeam
  - probe_shutdown_cascade_pipefd
  - probe_lifeline_pipe_proof
  - probe_lifeline_orphan_clean_via_substrate
  - probe_lifeline_orphan_clean_via_fork_program
  - probe_pdeathsig_kills_orphan_child
- **Row I**: `cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "FAILED$"` — neither `deftest_wat_tests_std_stream_test_with_state_buffer_all_at_eos` nor `deftest_wat_lru_test_local_cache_put_then_get` appears. Total failure count ≤ 9 (down from 11; if lifeline flake also resolves, down to 8 or below).
- **Row J**: Compare the post-Phase-3 failure set to baseline pre-FD-multiplex (`/tmp/baseline-pre-fdmpx.log`). 9 failures appear in both: 5 svc-test (arc 109 `:()` rot), 2 tmp tests, 1 lifeline flake (pre-existing pressure pattern), 1 wat-cli (`startup_error_bubbles_up_as_exit_3`). These 9 are unchanged — Phase 3 doesn't touch them.

## Honest deltas to watch for

- **Visibility / module boundary.** `child_post_fork_init` needs `pub(crate)` since spawn_process.rs calls it. `install_silent_panic_hook` and `close_inherited_fds_above_stdio` are currently `fn` (private to fork.rs); they're called by child_post_fork_init so they stay accessible. spawn_process.rs no longer calls install_silent_panic_hook directly — verify no broken imports.
- **Test pressure may persist (lifeline flake).** Row I expects stream + lru to PASS under workspace pressure post-Phase-3. The lifeline_pipe_zero_orphans_across_100_trials flake (chained-fork-pressure on a pure-libc probe) is a SEPARATE issue — Phase 3's FD hygiene fix may or may not affect it. If lifeline flake persists, that's expected and acceptable for Phase 3 closure (separate issue, separate fix).
- **Order of dup2 + helper.** Both fork paths dup2 stdio BEFORE calling the helper (the helper's setpgid + later steps assume fd 0/1/2 are already wired). Verify the call sites place child_post_fork_init AFTER the existing dup2 + drop work, before any startup_from_*.
- **Error path from child_post_fork_init.** The helper's setpgid failure path is `_exit(EXIT_STARTUP_ERROR)`. Caller's `child_branch_from_source` and `spawn_process_child_branch` are both marked `-> !` (never return). The helper returns `()` normally; on error it `_exit`s. This is correct — caller proceeds with normal control flow on success.

## Workspace baseline (post-Phase-2, commit 6062cfc)

- 2260 passed / 11 failed (per `/tmp/post-fdmpx.log`)
- The 2 new failures vs pre-FD-multiplex baseline (198c30b: 2258/9):
  - `deftest_wat_tests_std_stream_test_with_state_buffer_all_at_eos` (5000ms timeout under workspace pressure; passes in 0.02s in isolation)
  - `deftest_wat_lru_test_local_cache_put_then_get` (5000ms timeout under workspace pressure; passes in 0.02s in isolation)

Post-Phase-3 target: 2260+ passed / ≤9 failed; both pressure failures resolved.

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 10–15 min | TBD |
| Scorecard rows | 10/10 PASS | TBD |
| Workspace fail count | ≤ 9 (down from 11) | TBD |
| Honest deltas | 1–2 surfaces | TBD |
| Mode | A (clean) | TBD |
