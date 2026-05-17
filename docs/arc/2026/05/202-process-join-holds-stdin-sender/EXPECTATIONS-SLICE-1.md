# Arc 202 Slice 1 EXPECTATIONS

**BRIEF:** `BRIEF-SLICE-1.md`
**Drafted:** 2026-05-16, pre-spawn.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- New CheckError variant + Display + Diagnostic: ~40 LOC mirroring Gap K's existing variants
- Collector extension or new finder: ~30-50 LOC
- Hook in check_let: ~10 LOC
- wat/test.wat fix: ~3 LOC
- New test file: ~80-120 LOC (3 tests)
- Build cycle + targeted test runs + full workspace verify
- Investigation time: sonnet reads Gap K's existing implementation to mirror shape — ~10 min
- Bug-investigation time: if Gap K's actual behavior differs from BRIEF (decay disclosure caveat), sonnet captures + adapts — ~10 min if it fires

Mechanical scope. Smaller than arc 201 slice 1 (which was a real shape change). Comparable to slice 3 (one new feature with mirror-an-existing-helper machinery). Slightly larger than slice 4 (rename) because the new walker logic requires real thought.

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — Variant minted | YES | high (mechanical mirror of Gap K) |
| B — Rule fires on deadlock shape | YES | high (test is direct; the deadlock pattern is concrete) |
| C — Rule doesn't fire on legal shape | YES | medium-high (depends on detection precision; STOP-trigger 3 catches false positives in existing tests) |
| D — run-hermetic-driver fixed; hung test passes | YES | high (the fix is mechanical; the deadlock cause is identified) |
| E — Workspace failure count ≤ 4 | YES | medium (depends on no other latent stdin deadlocks existing — STOP-trigger 3 catches and forces in-slice fix) |

**5/5 PASS predicted; ~75% confidence overall.** Lower confidence than slice 4 because this is a substrate logic addition (not pure mechanical sweep).

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **Detection mechanism nuance.** The "inner-scope vs sibling-binding" distinction may require more care than the BRIEF describes. Sonnet may settle on a simpler "ANY Process/stdin <p> appears in the let scope" rule v1 with documented false-negative for sibling-binding case. That's an honest delta + good first version.

2. **Other wat helpers with the same shape.** A grep of `wat/` for `Process/join-result` may surface other macros/helpers that have the same gap. If found, fix in-slice; surface count.

3. **Gap K's actual mechanism differs from BRIEF description.** Per decay disclosure: orchestrator's model is partial. Sonnet investigates and either confirms the model OR surfaces the actual behavior (the BRIEF will need correction post-hoc; capture in SCORE).

4. **Test fixture wat-program shapes.** Constructing minimal wat that demonstrates the deadlock-shape vs legal-shape may require careful wat — sonnet may need to use `:wat::kernel::spawn-process` directly with concrete forms. The test's wat program shape is sonnet's design decision; pattern after existing `wat_run_sandboxed_ast.rs` tests.

5. **Possible spurious failures in existing tests.** The new rule might catch deadlock-shape PATTERNS we don't have an existing deadlock for (because the test happens to never hit the timing window). Those tests would then fail check-time. Sonnet either: (a) fixes them in-slice with explicit stdin extraction, (b) surfaces them as honest delta requiring follow-up. Per `feedback_no_known_defect_left_unfixed`: lean (a).

### Less likely surprises

6. **The Sender isn't actually held by proc.** If the Process struct doesn't carry the stdin Sender as a field (e.g., it's owned by an internal substrate-side store), the deadlock has a different root cause. STOP-trigger 4 catches.

7. **Inner-scope detection logic is hard.** If verifying "the extraction's scope closes before the join's position" requires AST positional analysis, sonnet may simplify to a more permissive check. Honest delta.

8. **Symbol name matching across scopes.** The collector matches on `id.name.clone()` — same string identifier. If wat allows shadowing or qualifier-suffixed identifiers, the rule may have edge cases. Should match Gap K's behavior.

## Workspace baseline (commit `ecc876a`, captured post-slice-4 at 2319 PASS / 4 FAIL)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 2319 passed / 4 failed
- Pre-existing failures (DO NOT BLOCK):
  - `lifeline_pipe_zero_orphans_across_100_trials` (FD-multiplex Phase 1D flake variance)
  - `deftest_wat_tests_tmp_totally_bogus` (unresolved reference in test fixture)
  - `t6_spawn_process_factory_with_capture_round_trips` (arc 170 slice 6 documented gap)
  - `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)

Post-slice-1 target:
- Pass count ≥ 2319 + 2-3 (new tests pass; the previously-hung `wat_run_sandboxed_ast::ast_entry_prints_hello` now passes cleanly)
- Fail count ≤ 4

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | ≤ 4 | TBD | TBD |
| New test count | 3 | TBD | TBD |
| LOC | ~160 across 3 files | TBD | TBD |
| Detection mechanism | (β) likely; possibly both (α+β) | TBD | TBD |
| Other wat helpers found w/ same shape | 0-1 | TBD | TBD |
| Gap K description matches reality | mostly (orchestrator decay caveat applies) | TBD | TBD |
| STOP-triggers fired | 0-1 | TBD | TBD |
| New latent deadlocks surfaced | 0-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
