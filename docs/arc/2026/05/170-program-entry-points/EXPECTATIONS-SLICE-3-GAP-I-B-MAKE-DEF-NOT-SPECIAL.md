# Arc 170 slice 3 Gap I-B EXPECTATIONS (sonnet scorecard)

**One spawn.** Make `def` not special — retire validator's def-specific arm + tighten runtime arm to emit position-class error. Closes Gap I-A's honest delta (def-end-to-end-spawn-lifts) and a latent arc-157 permissive-runtime defect.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

**Hard cap:** 180 min (2×). ScheduleWakeup at T+10800s.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::core::def` arm in `validate_def_position_with_wrapper` DELETED (def now falls through `_ =>` like the 7 siblings); check-time validator silent for def | grep + read |
| B | `:wat::core::def` arm in eval dispatch (`runtime.rs:3520-3540`) emits position-class error (chosen variant per Phase 2); does NOT silently return Unit | grep + read |
| C | Error variant minted/renamed per Phase 2 decision (α: mint `DeclarationInExpressionPosition` carrying head + span; route both def + define through; retire `DefineInExpressionPosition` via sweep); Display renders the correct head | grep + read + Display test |
| D | 5+ new probes in `tests/probe_def_not_special.rs` pass: end-to-end spawn lift; runtime position error; top-level regression; define regression; mixed 8-form prelude | cargo test |
| E | All 6 Gap I-A probes + all 5 Gap H probes + all 11 prior substrate probes still pass | cargo test |
| F | `cargo check --release` green; workspace at 2238 + N (- M_updated) / 0 failed; M is the test sweep size (Phase 1 audit reveals) | full test run |

**6 rows.** All must PASS.

## Implementation approach (sonnet's path)

1. **Audit existing surface** (10-15 min) — grep DefNotTopLevel + DefineInExpressionPosition + def-at-expression-position tests; document the inventory before code changes
2. **Choose variant name** (5 min) — α/β/γ per BRIEF Phase 2; recommendation α; rationale in SCORE
3. **Retire validator def-arm** (5 min) — delete `:wat::core::def` arm in `validate_def_position_with_wrapper`
4. **Tighten runtime arm** (10-15 min) — replace permissive `_value = eval(...); Ok(Value::Unit)` with `Err(RuntimeError::DeclarationInExpressionPosition(head, span))` (or chosen variant)
5. **Mint/rename error variant** (10-15 min) — per Phase 2 choice; if α, sweep `DefineInExpressionPosition` callers to new variant
6. **Sweep tests** (15-30 min) — update tests asserting on retired/changed behavior; size depends on Phase 1 audit
7. **New probes** (15-20 min) — 5+ probes in `tests/probe_def_not_special.rs`
8. **Verify** (10-15 min) — Gap I-A + Gap H + 11 prior probes; full workspace

## What sonnet produces

- `src/check.rs` modified (validator def-arm deleted)
- `src/runtime.rs` modified (def runtime arm tightened; error variant mint/rename if α)
- Any existing test files modified per sweep (Phase 6)
- New probe file `tests/probe_def_not_special.rs`
- `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-I-B-MAKE-DEF-NOT-SPECIAL.md` with:
  - 6-row scorecard with PASS/FAIL each row
  - Phase 1 audit inventory (DefNotTopLevel sites, DefineInExpressionPosition sites, def-at-expression-position tests)
  - Phase 2 variant naming choice + rationale (α/β/γ)
  - Test sweep enumeration (which tests changed and why)
  - Public API impact assessment (if DefineInExpressionPosition was exported)
  - Honest deltas (≥ 3)

**Do NOT commit.** Orchestrator atomic-commits after scoring.

## What sonnet must NOT do

- Modify `is_declaration_form` — Gap I-A predicate ships unchanged
- Modify extract_closure lift — Gap H + Gap I-A behavior must be preserved
- Modify other 7 declarations' position discipline (the `_ =>` arm catches them via recursion; their runtime arms stay unchanged unless Phase 2 chose α and the define dispatch arm updates with the new variant)
- Add loads / config setters to the changes — separate semantic category, out of scope
- Modify deftest-hermetic macro shape — separate slice
- Touch `docs/arc/` (FM 11)
- Commit / push / git add / git restore
- Use deferral language in SCORE
- Operate outside `/home/watmin/work/holon/wat-rs/`
- Touch `~/.claude/` memory system
- Use --no-verify or skip hooks
- Delete `CheckError::DefNotTopLevel` variant — if no emitters remain post-retirement, flag as honest follow-up; do NOT remove in this slice (separate retirement sweep with consumer audit)
- If a Gap H or Gap I-A regression surfaces, STOP and report — Gap I-B must be transparent for the lift mechanism's existing forms

## Verification commands

```bash
# New Gap I-B probes
cargo test --release --test probe_def_not_special 2>&1 | tail -10

# Gap I-A regression (CRITICAL — load-bearing)
cargo test --release --test probe_declaration_form_lift 2>&1 | tail -10

# Gap H regression
cargo test --release --test probe_closure_body_prelude_lift 2>&1 | tail -10

# All 11 prior substrate probes
cargo test --release \
  --test probe_do_splice_def --test probe_let_splice_def \
  --test probe_do_splice_define --test probe_let_splice_define \
  --test probe_do_splice_struct --test probe_do_splice_enum \
  --test probe_let_splice_struct --test probe_let_splice_enum \
  --test probe_spawn_process_parent_type \
  --test probe_resolver_quote_awareness \
  --test probe_deftest_hermetic_isolation 2>&1 | tail -5

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline (post-Gap-I-A, commit `8c13631`): **2238 passed / 0 failed**
- Post-Gap-I-B: **2238 + N - M_updated passed / 0 failed**, where:
  - N = new probes (≥ 5)
  - M_updated = existing tests modified to match new behavior (assertions updated, not removed — same test count, different assertion shape)

If failed > 0 after I-B's changes:
- Regression in Gap H or Gap I-A's lift → STOP and report
- Probe failure in I-B's own new tests → expected during iteration; sonnet fixes; final must be 0 failed
- Existing tests breaking unexpectedly → likely surfacing the test sweep size from Phase 1 audit; surface in SCORE; if size unexpectedly large (>20 sites), STOP and report for orchestrator review

## Honest delta categories (anticipated)

1. **Variant naming choice** — Phase 2 sub-decision. α recommended; surface choice + rationale.
2. **`CheckError::DefNotTopLevel` orphan variant** — after retirement, if no emitters remain, flag as honest follow-up (affirmative scope-bounding, NOT deferral language). Cleanup in a separate sweep.
3. **Test sweep size** — Phase 1 audit reveals. Document inventory in SCORE.
4. **Public API impact** — does retiring `DefineInExpressionPosition` (Phase 2 α) break consumers? Surface in SCORE.
5. **Deep-recursion def-violations** — symmetric with other 7 forms post-retirement; surface as expected behavior, NOT a regression.
6. **Anything unexpected** — particularly around test sweep cascades, or runtime behavior differences sonnet finds while implementing

## Constraints — orchestrator-side discipline mirror

The orchestrator runs:
- FM 9 baseline check pre-spawn: workspace 2238/0 verified (just post-Gap-I-A)
- FM 12 model: "sonnet" explicit on Agent call
- FM 16: no Bash/cargo/tool-availability preamble in BRIEF (BRIEF doesn't mention them)
- Time-box: ScheduleWakeup at T+10800s (2× 90-min upper bound)
- Atomic-commit after scoring (FM 11 pre-INSCRIPTION grep NOT applicable — I-B is not closure paperwork)
- Push after commit (per `feedback_push_on_commit.md`)
