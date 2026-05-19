# Arc 213 stone γ-2 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 30-50 min Mode A. Smaller than γ-1 because pattern is established + sister child_branch_from_source already correct + LifelineWriter::into_owned_fd already exists.
- **LOC changed:** ~40-60 (fork_program_from_source body shrinks; manual lifeline plumbing removed; spawn_lifelined block replaces bare-fork block)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** LOW-MEDIUM. Pattern already proven by γ-1; main risk is closure-capture / stdio fd ownership details that γ-1 worked through.

## Honest-delta watch

### Risk 1 — OwnedFd reconstruction parity with γ-1

γ-1 converts OwnedFds to raw `i32` before the closure (`into_raw_fd()`), reconstructs inside closure for child branch consumption, reconstructs again parent-side after `spawn_lifelined` returns, then manually closes child-side fds. γ-2 mirrors this pattern. Any mismatch in fd ownership orchestration (double-close, missing close, dangling raw fd) surfaces as test failure.

### Risk 2 — `loader: Arc<dyn SourceLoader>` capture into FnOnce closure

The closure body captures `loader` (an `Arc<dyn SourceLoader>` trait object). `Arc<dyn T>` implements `UnwindSafe + Send` if T is, but `dyn SourceLoader` may not have explicit Send bounds. spawn_lifelined's closure trait bound is `F: FnOnce(i32) + UnwindSafe`. If `dyn SourceLoader` isn't UnwindSafe, the closure won't satisfy the bound.

Mitigation: γ-1 captured `Vec<WatAST>` + `Option<Config>` which auto-satisfied. γ-2's `Arc<dyn SourceLoader>` may need `AssertUnwindSafe` OR the trait may need a `+ UnwindSafe` bound. Document the resolution in SCORE.

### Risk 3 — wat-cli end-to-end stdio timing

wat-cli (`crates/wat-cli/src/lib.rs:370`) calls `fork_program_from_source` and proxies stdin/stdout/stderr between the user's shell and the child wat program. spawn_lifelined's `setpgid(0,0)` change (child becomes own pgrp leader) may affect how wat-cli's signal-forwarding logic interacts with the child (arc 104d signal forwarding). If wat-cli's tests rely on inherited pgroup, they'll surface.

γ-1 already shipped setpgid(0,0) for the fork-program-ast path and wat-cli's 15/15 still passed — so this risk is bounded.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `fork_program_from_source` body migrated to spawn_lifelined | YES |
| 2 | Manual lifeline pipe creation removed (lines 957-958) | YES |
| 3 | Parent's `drop(lifeline_r)` removed (line 1005) | YES |
| 4 | Bare `libc::fork()` block replaced with `spawn_lifelined` closure | YES |
| 5 | LifelineWriter::into_owned_fd used for lifeline_w extraction | YES |
| 6 | `child_branch_from_source` UNCHANGED (sister fn; already correct) | YES |
| 7 | Public signature of `fork_program_from_source` UNCHANGED | YES |
| 8 | cargo build --release clean | YES |
| 9 | α probe `probe_pidfd_primitive` still 2/2 PASS | YES |
| 10 | `probe_lifeline_orphan_clean_via_fork_program` post-count == pre-count | YES (orchestrator reports pre-count in spawn prompt) |
| 11 | `wat_arc170_stone_b_walker_collapse` post-count == pre-count | YES (orchestrator reports pre-count) |
| 12 | `wat_arc170_program_contracts` 24/24 PASS preserved | YES |
| 13 | `wat-cli wat_cli` 15/15 PASS preserved | YES |
| 14 | Zero modifications outside `src/fork.rs` | YES |
| 15 | SCORE inscribes any closure-capture / fd-ownership subtleties | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; substrate's second fork site canonicalized
- **Mode B (acceptable):**
  - `dyn SourceLoader` UnwindSafe issue surfaces; sonnet documents resolution choice (AssertUnwindSafe wrap, trait bound add, etc.)
  - wat-cli pgroup interaction surfaces; document + REVERT if can't resolve in stone
- **Mode C:** STOP rule broken (touched γ-1/γ-3/δ, changed signature, modified caller sites, touched child_branch_from_source)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (pattern established by γ-1; sister fn already correct; γ-1 proved wat-cli compatibility with setpgid). HIGH on first-attempt Mode A.
- **Risk factors:**
  - `dyn SourceLoader` UnwindSafe is the most likely surface friction
  - wat-cli signal interaction is theoretically risky but γ-1 evidence indicates it's bounded
- **Why this matters:** continues arc 213's substrate-consistency closure. After γ-2 ships, only γ-3 (spawn_process.rs) remains in the fork-site canonicalization phase. δ/ε/ζ can then proceed cleanly.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

Within γ, sequencing γ-2 before γ-3:
- γ-2 is the smaller migration (fewer callers; pattern matches γ-1 more directly)
- γ-3 touches the heaviest spawn path (spawn-process; widest blast radius if Mode B)
- γ-2 first establishes second worked example post-γ-1 → γ-3 builds on N=2 examples

After γ-2 ships → re-run tiebreaker on γ-3 vs ζ-1.

## Cross-references

- Arc 213 DESIGN — full stone chain α/β/γ/δ/ε/ζ/η
- Arc 213 γ-1 SCORE (`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-GAMMA-1-FORK-PROGRAM-AST.md`) — pattern precedent + LifelineWriter::into_owned_fd
- `src/fork.rs:933-1013` — migration site (fork_program_from_source)
- `src/fork.rs:1134` — child_branch_from_source (already correct; do not touch)
- `src/fork.rs:584-693` — γ-1's migrated eval_kernel_fork_program_ast (the pattern to mirror)
- `feedback_tractability_tiebreaker` — sequencing discipline
- `feedback_substrate_owns_not_callers_match` — doctrine γ extends
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — architectural commitment
