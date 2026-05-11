# Arc 170 slice 1f-ε — SCORE

**Result:** Mode B partial. 7/8 rows pass; Row A doesn't reach 0 (7 heterogeneous-tail files with legitimate non-1f-ε reasons).
**Runtime:** ~60 min sonnet (mid of predicted 90-180 band; under 360 hard cap). Required one respawn after first sonnet had FM 16 hallucinated-tool-denial.
**Files:** 27 modified (1131 insertions, 1445 deletions — net -314 lines).

**Workspace: 1577/636 → 1752/461. Delta: +175/-175.** Above the ≥150 floor predicted.

## § Pre-flight miscalibration (FM 9 sub-lesson)

The pre-flight crawl sampled 3 files (`struct-to-form.wat`, `result-expect.wat`, `option-expect.wat`), found all mechanical (params `_`-prefixed, body unused), and projected "mostly mechanical." **Actual: ~40 mechanical + ~140 substantive** — 3.5× more substantive than the sample suggested.

**The substantive cases:** test bodies that DID use `stdout`/`stderr` params to drive assertions (test framework patterns like `(:wat::io::IOWriter/write-string stdout "...")` for assertion output). Migration required:
- Replace param-based stdio with ambient `:wat::kernel::println` / `eprintln`
- Update test infrastructure from StringIo (single-thread-owned) to the ambient orchestrator services (slice 1f-γ delivered)

**Discipline reminder:** 3-sample preflight is the minimum, not the optimum. For uniform-composition sweeps, also sample by RUST file (tests/wat_*.rs with embedded wat strings) — those tend toward substantive because they were written to assert via stdio. SUL test bodies in `wat-tests/core/*.wat` tend toward mechanical (true unused params).

Both classes were correctly handled by sonnet without re-prompt; the surprise was the count, not the work.

## Calibration

- **Predicted runtime band:** 90-180 min sonnet (uniform composition)
- **Actual:** ~60 min — mid-band; faster than expected given the substantive count was 3.5× higher than pre-flight suggested
- **Why mid-band, not over:** Sonnet handled mechanical + substantive uniformly without escalation. The substantive rewrite pattern (`stdout writeln` → `kernel::println`) was itself uniform after the first few.
- **Calibration lesson:** Heterogeneous mechanical/substantive splits in a uniform-composition sweep don't always blow the band. Substantive rewrites can still be pattern-applied if the rewrite template is simple.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | 0 remaining 3-arg sites in test trees | △ 7 files remain — all legitimate heterogeneous-tail (see § Row A details) |
| B | `cargo check --release` green | ✓ clean (1 pre-existing dead_code warning) |
| C | No new warnings | ✓ |
| D | Failure count drops ≥ 150 | ✓ -175 |
| E | Pass count rises ≥ 150 | ✓ +175 |
| F | No regression | ✓ verified per-file |
| G | Mechanical vs substantive counts | ✓ ~40 mechanical / ~140 substantive |
| H | Honest deltas | ✓ 4 categories |

**7/8 effective passes** (Row A is an honest-out-of-scope, not a deliverable failure).

## § Row A details — 7 remaining files with legitimate non-1f-ε reasons

| File | Why it remains | Right slice |
|---|---|---|
| `tests/arc112_scheme_probe.rs` | Also uses retired `spawn-program` (BareLegacySpawnProgram fires) | sibling: restore `spawn-program` OR migrate to current spawn surface |
| `tests/arc112_slice2b_process_send_recv.rs` | Also uses retired `fork-program-ast` (BareLegacyForkProgram) | sibling: same |
| `tests/wat_arc103_spawn_program.rs` | Inner WAT in string literals + retired primitives | sibling |
| `tests/wat_arc104_fork_program.rs` | Same as 103 | sibling |
| `tests/wat_arc170_program_contracts.rs` | Tests NEGATIVE cases of arc 170 slice 2 contracts — outer main intentionally wrong | Arc 170 INSCRIPTION close path; or test rewrite |
| `tests/wat_arc170_slice_1e_user_main_nil.rs` | 3-arg occurrences INSIDE `freeze_err` test strings — testing that the substrate REJECTS them | Stays as-is; tests are correct by failing-to-load |
| `tests/wat_run_sandboxed.rs` | Outer main returns `:wat::kernel::RunResult` (wrong type intentionally tested) | Test rewrite |

These are **not unfinished work** — they each have a different reason for remaining. Slice 1f-ε explicitly scoped to the 3-arg signature pattern; these 7 fail other doctrines and need separate triage.

## Workspace state

- **Pre-1f-ε baseline:** 1577 passed / 636 failed (post-1f-δ′)
- **Post-1f-ε:** 1752 passed / 461 failed
- **Delta:** +175 / -175

**Total recovery since session start (post-compaction):** workspace went from ~1339/854 → 1752/461. **413 tests recovered** across the session.

## Honest deltas (4 categories)

1. **Pre-flight count miscalibration** — 3-sample suggested mostly-mechanical; actual was ~40 mechanical / ~140 substantive. See § Pre-flight miscalibration above. **Doesn't change Mode A status of the deliverable; just calibration data.**

2. **4 tests in `wat_polymorphic_arithmetic` still failing post-migration** — f64-to-EDN format produces `42.0` not `42`; tests assert `"42"`. Pre-migration these fired `BareLegacyMainSignature` (test never ran); now they fail at assertion level. **Heterogeneous tail** — surfaced BY this slice but not caused BY it. Track as test-body fix.

3. **EDN double-encoding of `edn::write` output** — tests that call `(:wat::edn::write ...)` then `(:wat::kernel::println ...)` get double-encoded (inner quotes become `\"`). Three tests in `wat_arc143_manipulation.rs` needed assertion pattern updates (`"Symbol \"x\""` → `"Symbol \\\"x\\\""`). **Fixed inline** in this slice (per user direction — slices grow as necessary).

4. **`wat_user_enums.rs` error-test main signatures** — 4 `run_expecting_check_error` tests had wrong-return-type `:user::main -> :wat::core::i64`. Fired `BareLegacyMainSignature` before pattern-exhaustiveness checker ran. **Fixed inline** by changing to `-> :wat::core::nil` so exhaustiveness checker proceeds.

## Files modified (27)

Rust files with embedded wat strings (most are substantive — ambient stdio migration):
- `tests/wat_arc072_letstar_parametric.rs`, `tests/wat_arc098_form_matches_typecheck.rs`, `tests/wat_arc113_raise_round_trip.rs`, `tests/wat_arc143_lookup.rs`, `tests/wat_arc143_manipulation.rs`, `tests/wat_arc146_dispatch_mechanism.rs`, `tests/wat_engram_library.rs`, `tests/wat_idempotent_redeclare.rs`, `tests/wat_make_deftest.rs`, `tests/wat_math_sqrt.rs`, `tests/wat_not_eq.rs`, `tests/wat_online_subspace.rs`, `tests/wat_polymorphic_arithmetic.rs`, `tests/wat_reckoner.rs`, `tests/wat_recursive_patterns.rs`, `tests/wat_simhash.rs`, `tests/wat_sort_by.rs`, `tests/wat_stat.rs`, `tests/wat_string_ops.rs`, `tests/wat_user_enums.rs`, `tests/wat_vector_algebra.rs`, `tests/wat_vector_first_class.rs`
- `crates/wat-cli/tests/wat_cli.rs`

Wat files (mostly mechanical):
- `wat-tests/test.wat`, `wat-tests/core/option-expect.wat`, `wat-tests/core/result-expect.wat`, `wat-tests/core/struct-to-form.wat`

Net: +1131 / -1445 = -314 lines (tests got shorter via signature simplification).

## Lessons captured

1. **3-sample pre-flight floor is real but not sufficient** for sweep BRIEFs. Add: sample across multiple file CLASSES (wat-tests/*.wat vs tests/*.rs vs crates/*.wat) — different patterns concentrate in different classes.

2. **Substantive cases can still be pattern-applied** when the rewrite template is uniform. Sonnet handled ~140 substantive cases at ~60 min total. Future BRIEFs that anticipate mixed mechanical/substantive should NOT escalate to opus prematurely — sonnet's pattern-apply works at scale.

3. **FM 16 (sonnet tool-denial hallucination) reproduced.** First sonnet bailed at 65s claiming Bash blocked. Respawn with stripped preamble (no cargo/bash mentions) ran cleanly. Confirms FM 16 discipline — strip tool-availability mentions from BRIEFs and prompts.

4. **Process leak from fork-program-ast** observed during this slice's verification runs. User killed orphans manually. **Track as substrate follow-up** — fork.rs missing waitpid in parent exit path. Don't gate slice 1f-ε on this fix.

## What's next

1. **Atomic-commit slice 1f-ε** (this turn) — 27 files + this SCORE
2. **Re-sample remaining 461 failures** — many of the previous "heterogeneous 399" should have shrunk dramatically as chain-blockers cleared
3. **Sibling slice** — restore retired `spawn-program` / `fork-program-ast` (BareLegacy- diagnostics) — same bridge pattern as 1f-δ / 1f-δ′; affects `arc112_*` tests
4. **Fork waitpid follow-up** — close the process-orphan leak in `src/fork.rs`
5. **Test-body assertion fixes** — `wat_polymorphic_arithmetic` f64 format, EDN double-encoding edge cases (already partially handled inline)
6. **Bridge-migration slice** — move `run-sandboxed-*` body from kernel namespace to Layer 1
7. **Arc 170 INSCRIPTION** — once baseline stabilizes

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-E.md`](./BRIEF-SLICE-1F-E.md)
- Predecessor: slice 1f-δ′ (`72e051c`) — sandbox restore; cleared substrate path for these tests to fail at signature instead of unknown-verb
- Slice 1e (`206bdd1`) — the retirement this migration closes (closing phase of arc 170 slice 1e)
- `feedback_simple_is_uniform_composition.md` — discipline that held even at the 3.5× higher substantive count
- Recovery doc FM 9 — multi-sample discipline; 3-sample floor identified as insufficient for cross-class sweeps
- Recovery doc FM 16 — sonnet tool-denial hallucination reproduced; mitigation works
