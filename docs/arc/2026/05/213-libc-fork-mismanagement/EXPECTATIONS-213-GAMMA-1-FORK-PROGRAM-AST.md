# Arc 213 stone γ-1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 45-75 min Mode A. Heavier than β because child_branch signature extends + LifelineWriter↔OwnedFd plumbing requires a small design choice + 6 test binaries to verify.
- **LOC changed:** ~50-80 (eval_kernel_fork_program_ast body ~30-40 LOC; child_branch signature + child_post_fork_init call ~5-10; possibly ChildHandleInner field change ~10-30 if option (c) selected)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** MEDIUM. Three risk surfaces.

## Honest-delta watch

### Risk 1 — LifelineWriter ↔ OwnedFd plumbing

`LifelineWriter` (arc 213 α) wraps an `OwnedFd` privately. `ChildHandleInner::lifeline_w` field is `Option<OwnedFd>`. Three options to bridge:

- **(a) `LifelineWriter::into_owned_fd(self) -> OwnedFd`** — extract the inner OwnedFd by consumption. Add a one-line method to LifelineWriter (additive). ChildHandleInner unchanged. Smallest blast radius.
- **(b) `LifelineWriter::as_fd() -> &OwnedFd`** — borrow-only access. Doesn't work — ChildHandleInner needs owned, not borrowed. **DISQUALIFIED.**
- **(c) Change `ChildHandleInner::lifeline_w` field type to `Option<LifelineWriter>`** — cleaner; substrate-honest (LifelineWriter is the canonical handle). But ChildHandleInner has other callers; blast radius depends on grep.

Sonnet greps ChildHandleInner field usage; picks based on blast count (>5 sites favor (a); ≤5 sites favor (c)). EXPECTATIONS predicts (a) most likely — ChildHandleInner already integrates with many consumer paths; adding `into_owned_fd` to LifelineWriter is the lowest-disruption shape.

### Risk 2 — dup2 + child_post_fork_init ordering

`child_branch` currently does dup2 of stdio pipes onto fd 0/1/2 (this happens after the existing "Drop parent-side pipe ends" calls). `child_post_fork_init` needs to be called AT THE RIGHT TIME relative to dup2:
- BEFORE dup2: lifeline_r_raw might collide with stdio fd numbers being dup2'd
- AFTER dup2 + close_inherited_fds_above_stdio: lifeline_r might get closed by the cleanup

Sister fn `child_branch_from_source` line 1111 has the correct ordering — sonnet mirrors it. If sister's pattern doesn't fit (different child_branch shape), document the deviation.

### Risk 3 — Double catch_unwind

spawn_lifelined wraps body in catch_unwind + _exit. child_branch internally has its own panic-exit paths (it returns `!` and uses _exit directly). The nesting:
- spawn_lifelined's catch_unwind catches anything child_branch's body panics
- child_branch's own _exit calls happen INSIDE the closure → execution never reaches spawn_lifelined's `Ok(()) => _exit(0)` branch
- Behavior: equivalent to current (child always _exits via its own paths); spawn_lifelined's catch_unwind is defensive net

No test should change behavior. Document if observed deviation.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `eval_kernel_fork_program_ast` body migrated to spawn_lifelined | YES |
| 2 | `child_branch` signature extended with `lifeline_r_raw: i32` | YES |
| 3 | `child_branch` calls `child_post_fork_init(lifeline_r_raw)` (mirroring sister fn line 1111) | YES |
| 4 | `ChildHandleInner::new(pid, Some(lifeline_w))` replaces `(pid, None)` — lifeline gap closed | YES |
| 5 | LifelineWriter↔OwnedFd plumbing choice documented in SCORE | YES |
| 6 | cargo build --release clean | YES |
| 7 | α probe `probe_pidfd_primitive` still 2/2 PASS | YES |
| 8 | `arc112_slice2b_process_send_recv` post-count == pre-count | YES |
| 9 | `wat_arc170_program_contracts` post-count == pre-count (t6 + t14 stay failing as pre-existing) | YES |
| 10 | `probe_run_hermetic_ast_stdout_capture` post-count == pre-count | YES |
| 11 | `probe_run_hermetic_no_deadlock` post-count == pre-count | YES |
| 12 | `wat-cli wat_cli` post-count == pre-count | YES |
| 13 | Zero modifications outside `src/fork.rs` | YES |
| 14 | Public signature of `eval_kernel_fork_program_ast` unchanged | YES |
| 15 | SCORE inscribes any subtleties (dup2 ordering, double-catch_unwind observed, etc.) | YES |

## Mode classification

- **Mode A:** all 15 criteria satisfied; γ-1 complete; lifeline gap closed for fork-program-ast path
- **Mode B (acceptable):**
  - LifelineWriter↔OwnedFd plumbing has a non-obvious complication; REVERT + inscribe options + return
  - A test fails in a way that surfaces a real signature/behavior mismatch (process-pgroup interaction, dup2 ordering)
- **Mode C:** STOP rule broken (touched γ-2/γ-3/δ, changed public signature, scope-crept)

## Calibration metadata

- **Orchestrator confidence:** MEDIUM-HIGH on the design (α + β proved the pattern; sister fn provides canonical precedent for child_branch extension). MEDIUM on first-attempt Mode A (the LifelineWriter↔OwnedFd plumbing is a real but small choice; dup2 ordering risk is real but bounded by sister fn precedent).
- **Risk factors:**
  - LifelineWriter doesn't expose OwnedFd extraction (option (a)) — α didn't add this; sonnet adds it
  - dup2 + close_inherited_fds_above_stdio interactions with lifeline_r_raw — sister fn has the precedent; mirror should work
  - 6 test binaries means 5 minutes of verification time even if all pass cleanly
- **Why this matters:** closes the **actual orphan-leak gap** arc 213 named. Production orphans observed surviving cargo test were the fork-program-ast path lacking a lifeline. γ-1 ships this fix.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

Post-β, the next-move tiebreaker was γ vs arc 212 ζ-1. γ won on secondary criteria (β cadence — bounded + clean commit; γ continues α/β momentum; closes production-known gap before pivoting to wide campaign).

Within γ, the decomposition γ-1/γ-2/γ-3 selects γ-1 first because:
- γ-1 is the no-lifeline gap closer (production-relevant)
- γ-2 + γ-3 are canonicalizations (lifeline already there; lower urgency)
- γ-1 establishes the LifelineWriter↔OwnedFd plumbing pattern γ-2/γ-3 reuse

Per-stone trust gate: γ-1 SCORE returns → orchestrator verifies → atomic commit → re-run tiebreaker on γ-2 vs γ-3 vs ζ-1.

## Cross-references

- Arc 213 DESIGN — full stone chain α/β/γ/δ/ε/ζ/η
- Arc 213 α SCORE (`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-ALPHA-MINT-PIDFD-PRIMITIVE.md`) — Pidfd + spawn_lifelined primitive
- Arc 213 β SCORE (`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-BETA-MIGRATE-RUN-IN-FORK.md`) — run_in_fork migration precedent (signature unchanged; internal-only)
- `src/fork.rs:474` — `child_post_fork_init` (the canonical Phase 3 helper γ-1 calls)
- `src/fork.rs:600-631` — eval_kernel_fork_program_ast (the migration site)
- `src/fork.rs:674` — child_branch (the extension site)
- `src/fork.rs:1072-1111` — child_branch_from_source (sister fn; canonical pattern to mirror)
- `src/fork.rs:946` — ChildHandleInner::new(pid, Some(lifeline_w)) (sister fn precedent for the parent-side closure of the gap)
- `feedback_tractability_tiebreaker` — sequencing discipline
- `feedback_substrate_owns_not_callers_match` — the doctrine γ extends
- INTERSTITIAL § 2026-05-18 (post-PURGE) "Linux 5.3+ syscall doctrine" — architectural commitment γ operationalizes
