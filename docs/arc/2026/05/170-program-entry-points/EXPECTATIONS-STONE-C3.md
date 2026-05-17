# EXPECTATIONS — Arc 170 Stone C3: type-keyword honesty fix

**BRIEF:** `BRIEF-STONE-C3.md`
**Drafted:** 2026-05-17, pre-spawn.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- ~2-4 lines in src/types.rs (ThreadPeer + ProcessPeer field-type heads)
- ~2-4 lines in src/check.rs (from-pipe registrations)
- Consumer sweep: unknown surface; probably 10-30 sites (grep first)
- Tests: zero new tests; existing tests verify (Counter actor proofs + Counter/Client capability proof + arc 170 D-family tests)
- Aliases are documented as unifying (arc 109 K-channel rename + arc 133 inference-time path) — should be a pure rename with no behavior change

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — ThreadPeer + ProcessPeer field types honest | YES | high |
| B — from-pipe return types honest | YES | high |
| C — consumer sweep clean | YES | medium-high (depends on sweep surface; may need to keep some internal `:rust::crossbeam_channel::*` references) |
| D — workspace baseline preserved | YES | medium-high (aliases should unify; if they don't fully, some tests may need adjustment) |
| E — runtime behavior unchanged | YES | high (rename only; no logic changes) |

**5/5 PASS predicted; ~80% confidence overall.** Lower than purely-additive slices because this is a structural rename that depends on alias unification working as documented.

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **Alias unification edge case** — somewhere in the substrate (parser? inference?) may treat the two names as distinct in a specific context. If found, surface the site + suggest either: (a) strengthen alias resolution, or (b) keep the rename limited to sites where unification works
2. **Consumer surface larger than expected** — if grep returns 50+ sites, the sweep itself becomes the slice's main work. Surface and discuss splitting
3. **`:rust::crossbeam_channel::*` references that legitimately stay** — substrate-internal Rust code references the actual crate (Sender::drop, channel::bounded, etc.). Those stay; only USER-FACING type-annotation positions get renamed
4. **Test fixtures that hard-code the dishonest names** — they'll fail until updated; sonnet should sweep them as part of consumer migration

### Less likely surprises

5. **Walker fires on new names but not old** (or vice versa) — would mean the walker is alias-unaware. If found, surface immediately (this would block C3 and need a sub-stone to make walker alias-aware first)
6. **Type inference produces different error messages** under the renamed types — minor; surface for any test that asserts on error message strings

## Workspace baseline (verified post-arc-203-slice-2 commit `e8101d8`)

`cargo test --release --workspace --no-fail-fast` baseline:
- 3 pre-existing stable failures (deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips)
- t6 deadlocks; orchestrator handles reaping

Post-C3 target:
- Pass count: = baseline (no new tests)
- Fail count: ≤ 3 (no regressions)

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | ≤ 3 | TBD | TBD |
| New test count | 0 | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-3 (alias edge cases, sweep surface size, internal-vs-user-facing references) | TBD | TBD |
| BRIEF corrections suggested | 0-2 | TBD | TBD |
| STOP-triggers fired | 0-1 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
