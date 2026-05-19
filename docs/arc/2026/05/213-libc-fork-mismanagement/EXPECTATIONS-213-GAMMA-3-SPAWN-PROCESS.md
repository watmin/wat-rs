# Arc 213 stone γ-3 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 45-75 min Mode A. Larger than γ-2 (5 binaries → 20 binaries to verify; sister fn signature change) but smaller than γ-1 (no constructor change; LifelineWriter::into_owned_fd already exists; pattern fully proven).
- **LOC changed:** ~60-100 (eval_kernel_spawn_process body ~40-60; spawn_process_child_branch signature -1 param + body -1 drop + ~10 lines of comment update)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** MEDIUM. Pattern established but 20 test binaries widen probabilistic surface for edge-case failures.

## Honest-delta watch

### Risk 1 — spawn_process_child_branch signature change (single caller)

Sister fn has ONE caller (the eval_kernel_spawn_process closure body). Dropping `lifeline_w: OwnedFd` param is mechanical. Risk is forgetting to update either site (compiler catches), or finding that `lifeline_w` is referenced elsewhere in spawn_process_child_branch body (grep shows only line 303 reference — the drop call we're removing).

### Risk 2 — Phase 1D "child is its own lifeline keeper" doctrine

spawn_process_child_branch's drop(lifeline_w) at line 303 carries explicit Phase 1D fix documentation (lines 296-302). The doctrine moves to spawn_lifelined (α). γ-3 updates the comment to note α now owns this discipline. Documentation drift risk: keep the WHY visible without confusing readers.

### Risk 3 — 19+1 test binary blast surface

19 :wat::kernel::spawn-process binaries + α regression. Many are short probes; some are heavy (wat_arc170_program_contracts has 24 tests; wat_arc208_process_io_result is comprehensive). One subtle interaction (timing-sensitive stdio, fd-count, pgroup behavior, structured-exit timing) can fail just one obscure probe.

γ-1 + γ-2 evidence: same setpgid + lifeline migration produced ZERO regressions in their test sets. γ-3 inherits that confidence — but pattern doesn't guarantee per-test outcome.

### Risk 4 — `probe_pdeathsig_*` are ACTIVE (1/1 PASS each on baseline)

Both `probe_pdeathsig_diagnostic` and `probe_pdeathsig_kills_orphan_child` are ACTIVE (1/1 PASS each) on baseline despite testing the retired PDEATHSIG-era diagnostics. They almost certainly test the LIFELINE replacement mechanism (Phase 1B/1C retired PDEATHSIG in favor of lifeline pipe; the probe names retain historical naming). γ-3's setpgid + spawn_lifelined change should preserve their behavior — but they're sensitive probes worth watching.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | eval_kernel_spawn_process body migrated to spawn_lifelined | YES |
| 2 | Manual lifeline pipe creation removed (lines 162-163) | YES |
| 3 | Parent's `drop(lifeline_r)` removed (line 202) | YES |
| 4 | Bare `libc::fork()` block replaced with spawn_lifelined closure | YES |
| 5 | LifelineWriter::into_owned_fd used for lifeline_w extraction | YES |
| 6 | spawn_process_child_branch signature: `lifeline_w: OwnedFd` param REMOVED | YES |
| 7 | spawn_process_child_branch body: `drop(lifeline_w)` (line 303) REMOVED | YES |
| 8 | Phase 1D comment block updated (notes α owns the discipline) | YES |
| 9 | Public signature of eval_kernel_spawn_process UNCHANGED | YES |
| 10 | cargo build --release clean | YES |
| 11 | α probe `probe_pidfd_primitive` still 2/2 PASS | YES |
| 12 | All 19 spawn-process test binaries: post-count == pre-count | YES |
| 13 | Zero modifications to `src/fork.rs` | YES |
| 14 | Zero modifications outside `src/spawn_process.rs` | YES |
| 15 | SCORE inscribes any closure-capture / sig-change subtleties + 20-binary count table | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; γ phase complete; ZERO bare libc::fork() calls remain in substrate (other than spawn_lifelined's clone3)
- **Mode B (acceptable):**
  - Closure-capture or sig-change complication you can describe but not resolve in this stone; REVERT + inscribe + return
  - A test fails in a way that surfaces real behavior mismatch
- **Mode C:** STOP rule broken (touched γ-1/γ-2/δ, changed public signature, modified callers in fork.rs)

## Calibration metadata

- **Orchestrator confidence:** HIGH on the design (pattern triple-proven by α+β+γ-1+γ-2; sister fn sig change is one-caller mechanical). MEDIUM-HIGH on first-attempt Mode A (20-binary surface widens probabilistic risk; spawn_process_child_branch sig change has zero non-mechanical risk but is the first stone in γ touching sister fn).
- **Risk factors:**
  - Closure capture of `forms: Vec<WatAST>` + `inherit_config: Option<Config>` — both auto-UnwindSafe (γ-1 validated)
  - spawn_process_child_branch sig change is mechanical (one caller)
  - 19 test binaries means ~3-5 minutes of verification time even if all pass cleanly
- **Why this matters:** completes arc 213's γ phase. After γ-3: substrate has ZERO bare libc::fork() (only spawn_lifelined's clone3 internal). The "every fork has a lifeline" guarantee is honest at every site. δ/ε/ζ proceed with the canonical primitive as foundation.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

Within γ, sequencing γ-3 LAST:
- γ-3 is the wider blast surface (19 binaries vs γ-2's 5) — sequencing it after pattern-establishment lowers risk
- γ-3 ships the sister-fn signature simplification — γ-1/γ-2 didn't need this; γ-3 doing it last means the simplification benefits from γ-1/γ-2's worked precedent
- After γ-3: γ phase COMPLETE. δ/ε/ζ are then unblocked; re-run tiebreaker with the canonical primitive in place at every site

After γ-3 ships → re-run tiebreaker on δ (waitpid/kill migration) vs ε (probe /proc migration) vs ζ (L2 module privacy) vs arc 212 ζ-1 (the foundation campaign).

## Cross-references

- Arc 213 DESIGN — full stone chain α/β/γ/δ/ε/ζ/η
- Arc 213 α SCORE — Pidfd + spawn_lifelined primitive
- Arc 213 β SCORE — run_in_fork migration precedent
- Arc 213 γ-1 SCORE — eval_kernel_fork_program_ast precedent + LifelineWriter::into_owned_fd introduction
- Arc 213 γ-2 SCORE — fork_program_from_source canonicalization precedent (most direct mirror)
- `src/spawn_process.rs:91-228` — migration site (eval_kernel_spawn_process)
- `src/spawn_process.rs:277-289` — spawn_process_child_branch signature (sig simplification target)
- `src/spawn_process.rs:296-303` — Phase 1D comment block (doctrine doc update)
- `feedback_tractability_tiebreaker` — sequencing discipline
- `feedback_substrate_owns_not_callers_match` — the doctrine spawn_lifelined embodies (α owns "drop inherited lifeline_w" so child branches don't repeat)
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — architectural commitment
