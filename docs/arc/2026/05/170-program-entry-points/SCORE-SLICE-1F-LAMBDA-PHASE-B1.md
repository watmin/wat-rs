# Arc 170 slice 1f-λ Phase B1 — SCORE (kernel-API test sweep)

**Result:** Mode A clean. 20/20 tests pass in `wat_arc170_program_contracts.rs` on first compile. All 16 original tests in the two input files dispositioned; both files deleted via `git rm`.

**Runtime:** ~40 min sonnet.

**Files:** 3 modified — `tests/wat_arc103_spawn_program.rs` DELETED; `tests/wat_fork.rs` DELETED; `tests/wat_arc170_program_contracts.rs` extended with T14 + T15 + T16.

**Workspace: 2165/32 → 2168/16** — +3 passes / -16 failures. Net **-16 from the slice 1f-λ B1 scope of 16** (13 CONSOLIDATE+DELETE dispositions + 3 REPLACEs = 3 new tests closing 16 failures).

## § Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `tests/wat_arc103_spawn_program.rs` dispositioned (file deleted) | ✓ `git rm` confirmed; 6 failures gone |
| B | `tests/wat_fork.rs` dispositioned (file deleted) | ✓ `git rm` confirmed; 10 failures gone |
| C | Canonical home extended with T-numbered tests for surviving scenarios | ✓ `grep -c "^fn t" tests/wat_arc170_program_contracts.rs` = 20 (> 17) |
| D | Disposition table: every original test has REPLACE / DELETE / CONSOLIDATE with rationale | ✓ disposition table below |
| E | All B1 tests in canonical home pass | ✓ 20/20 in `wat_arc170_program_contracts.rs` |
| F | Workspace BareLegacy* failure count drops by ≥ 14 | ✓ 32 → 16 (dropped by 16) |
| G | `cargo check --release` green | ✓ clean compile, 0 errors |
| H | Honest deltas surfaced (≥ 3 categories) | ✓ 4 categories below |

**8/8 rows pass.** Mode A clean.

## § Scenario inventory + disposition — `tests/wat_arc103_spawn_program.rs` (6 tests)

| Original test | Disposition | Rationale |
|---|---|---|
| `spawn_program_ast_child_writes_stdout_parent_reads_line` | CONSOLIDATE | T12 covers one-way child emit: child sends without recv'ing first; scenario identical under spawn-process |
| `spawn_program_ast_round_trip_via_pipes` | CONSOLIDATE | T4-T6 cover bidirectional parent-sends/child-reads/child-responds round-trip via typed channels; scenario structure maps 1:1 |
| `spawn_program_ast_stdout_eof_after_child_returns` | CONSOLIDATE | T13 covers parent observing disconnect after child exits; "second read returns None" maps to typed_recv returning Disconnected after child returns nil |
| `spawn_program_ast_stderr_is_separate_pipe` | DELETE | Worker fn cannot write to the stderr pipe via WAT under spawn-process: `:wat::kernel::eprintln` requires ThreadIO (via `with_thread_io`), which spawn-process does NOT install in the child branch. The scenario is obsolete on the new surface. |
| `spawn_program_ast_join_returns_unit_on_clean_exit` | CONSOLIDATE | T13 via `wait_child_exit_ok` proves exit 0 on clean nil return |
| `spawn_program_source_string_entry` | DELETE | `spawn-program` (source-string entry) is a retired surface; under spawn-process there is no source-string path — the worker fn is always a compiled WAT defn |

**Net arc103:** 3 CONSOLIDATE, 2 DELETE, 0 REPLACE. No new tests needed.

## § Scenario inventory + disposition — `tests/wat_fork.rs` (10 tests)

| Original test | Disposition | Rationale |
|---|---|---|
| `fork_child_writes_stdout_parent_reads_line` | CONSOLIDATE | T12 covers one-way child emit; identical scenario |
| `fork_child_writes_stderr_parent_reads_line` | DELETE | Same rationale as arc103 stderr test: worker fn cannot write to stderr pipe via WAT under spawn-process; eprintln requires ThreadIO not installed in child branch |
| `wait_child_returns_zero_on_success` | CONSOLIDATE | T13 covers clean child exit + wait_child_exit_ok asserts exit 0 |
| `wait_child_is_idempotent` | REPLACE → T14 | `wait_or_cached()` idempotency (OnceLock caching) not proven in T1-T13; T14 calls wait_child_exit_ok twice on the same handle |
| `wait_child_surfaces_startup_error_exit_code` | DELETE | Startup error scenario obsolete under spawn-process: the worker fn is compiled in the parent's world at freeze time; type errors surface at freeze (not at spawn time); no startup-error exit path exists for spawn-process |
| `wait_child_surfaces_panic_exit_code` | REPLACE → T15 | Child panic → recv Disconnected + handle exit ≠ 0; distinct from T13's clean disconnect. T15 uses Option/expect on None to trigger panic_any in the child |
| `wait_child_surfaces_runtime_error_exit_code` | CONSOLIDATE | Same observable behavior as T15 (child exits non-zero before sending → recv Disconnected); exit code 1 (runtime) vs 2 (panic) not meaningfully distinct at the typed channel level; T15 covers the non-zero exit scenario |
| `multiple_sequential_forks_no_leak` | REPLACE → T16 | Three sequential spawn+exit cycles from one parent; proves no fd/zombie accumulation; not covered by T1-T15 |
| `wait_child_surfaces_nonzero_exit_code` | DELETE | Scenario is `:user::main` signature mismatch (EXIT_MAIN_SIGNATURE=4). Under spawn-process, the worker fn IS the contract — there is no separate main-signature validation step; the exit-code path doesn't exist |
| `fork_child_reads_stdin_from_parent` | CONSOLIDATE | Bidirectional parent-sends-child-reads-child-responds round-trip; T4-T6 cover this structure |

**Net wat_fork:** 4 CONSOLIDATE, 3 DELETE, 3 REPLACE (T14 + T15 + T16).

## § Honest deltas (4 categories)

1. **Stderr scenario obsolete on both files.** Two tests (one per file) asserted "child writes to stderr, parent reads it." Under spawn-process, the child's stderr fd IS piped to the parent (via dup2 in `spawn_process_child_branch`), but WAT-level access from the worker fn is blocked: `:wat::kernel::eprintln` requires ThreadIO (`with_thread_io`), which spawn-process does NOT install in the child branch. The scenario is theoretically observable when the child panics or has a runtime error (those write to fd 2 directly via `write_direct_to_stderr`), but T4's Disconnected error-drain path already surfaces that. No new test is needed or possible at the WAT level.

2. **Three DELETE dispositions for obsolete exit-code paths.** Startup-error (exit 3), nonzero-main-signature (exit 4), and source-string-entry tests all rely on infrastructure that doesn't exist in spawn-process. The startup error and main-signature paths require a separate WAT program parsing/validation step that happens inside the child — under spawn-process the worker fn is compiled in the parent's world before fork. Source-string entry (`spawn-program`) is a retired primitive. All three deletions are clean: no scenario survives, no test is written.

3. **Consolidation rate higher than Phase A's.** Phase A reduced 4 tests to 2 REPLACEs + 2 DELETEs. Phase B1 reduces 16 tests to 3 REPLACEs + 5 CONSOLIDATEs + 5 DELETEs + 3 CONSOLIDATEs. The canonical home (T4-T6, T12, T13) absorbed 8 of the 16 scenarios directly. This confirms Phase A's canonical pattern is appropriately broad — it already covers the key structural scenarios (round-trip, one-way emit, clean disconnect, exit 0).

4. **Idempotency scenario surface (T14) required Arc::clone discipline.** `wait_child_exit_ok` takes `Arc<ProgramHandleInner>` by value; calling it twice requires `handle.clone()` before the first call, then passing the original for the second. The `Arc<ChildHandleInner>::wait_or_cached` caches via `OnceLock::set()` — the first call does `waitpid`, subsequent calls read the cache. T14's two sequential `wait_child_exit_ok` calls prove both the caching mechanism and the arc-clone-safe ownership pattern.

## § Files modified

- `tests/wat_arc103_spawn_program.rs` — DELETED (257 lines; arc-103 spawn-program/spawn-program-ast surface retired)
- `tests/wat_fork.rs` — DELETED (304 lines; arc-012 fork-program-ast surface retired)
- `tests/wat_arc170_program_contracts.rs` — +93 lines (T14 + T15 + T16 at canonical home)

**Net: -468 lines** (561 deleted, 93 added; canonical home grew by 3 tests).

## § Workspace delta

- **Baseline (post Phase A):** 2165 passed / 32 failed
- **Post B1:** 2168 passed / 16 failed
- **Delta:** +3 passes (T14+T15+T16 added), -16 failures (16 input-file tests gone)
- **Remaining 16 failures:** Pattern B2 scope (wat-cli + examples; B2 ships in spawn 2) + 4 `slice4_*` heterogeneous-dispatch failures (independent of arc 170)

## § What's next

1. Orchestrator atomic-commits Phase B1 (deletions + canonical extension + this SCORE); push
2. Phase B2 BRIEF authored — Pattern B2 (wat-cli + examples), 12 tests, const-string replacement shape
3. Spawn sonnet for Phase B2 per the B2 BRIEF
4. Triage the 4 `slice4_*` heterogeneous-dispatch failures (independent arc 146 territory)
5. Arc 170 INSCRIPTION when workspace baseline is clean (or clean minus slice4)

## § Cross-references

- BRIEF (Phase B): [`BRIEF-SLICE-1F-LAMBDA-PHASE-B.md`](./BRIEF-SLICE-1F-LAMBDA-PHASE-B.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1F-LAMBDA-PHASE-B1.md`](./EXPECTATIONS-SLICE-1F-LAMBDA-PHASE-B1.md)
- Phase A SCORE (disposition-table precedent): [`SCORE-SLICE-1F-LAMBDA-PHASE-A.md`](./SCORE-SLICE-1F-LAMBDA-PHASE-A.md)
- Canonical home: [`tests/wat_arc170_program_contracts.rs`](../../../../../tests/wat_arc170_program_contracts.rs) (now 20 tests; T1-T16 + sub-variants T1b/T2b/T8b/T9b)
