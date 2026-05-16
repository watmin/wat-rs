# Arc 170 Stone B BRIEF — walker collapse: hide `*_join-result` from user namespace

**Phase:** Stone B of the bracket-combinator implementation chain. See `BRACKET-IMPLEMENTATION-STONES.md`.
**Predecessor:** Stone A SHIPPED (commit `2a198bd`) — `Thread/drain-and-join` + `Process/drain-and-join` substrate-vended user-callable helpers exist.
**Successor:** Stone C (mint Client/Server type pairs); independent — does not depend on Stone B.

## Goal

Add a NEW walker check in `src/check.rs` that rejects user-namespace calls to `:wat::kernel::Thread/join-result` and `:wat::kernel::Process/join-result`. Substrate-namespace callers (`:wat::*`) remain unaffected.

The new rule is binary:
- Caller's enclosing wat definition is in `:wat::*` namespace → allowed
- Caller's enclosing wat definition is in any other namespace (`:user::*`, `:my::*`, etc.) → compile error with helpful message

**Existing arc 117/133 scope-deadlock walker machinery STAYS in this stone** — it retires in Stone G after Stone F migration. Stone B is ADDITIVE: a new walker rule alongside the existing ones, not a replacement.

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures this session. THIS BRIEF describes the TARGET SHAPE. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — the exact walker entry point, how namespace context is tracked during traversal, the error type to emit, the precise check fn signature. Do NOT trust orchestrator claims about walker internals without grep verification.

Substrate state pointers (verified by orchestrator):
- `src/check.rs:7898` — `check_let_for_scope_deadlock_inferred` (arc 117/133; STAYS in Stone B)
- `src/check.rs:6858` — call site where arc 117/133 walker fires during type-check
- `src/check.rs` — somewhere has the call-site walker that catches verb calls like `Thread/join-result` (sonnet to locate)
- Error formatting precedent: arc 117 + arc 133 errors with helpful messages like "scope-deadlock at {join}: ... use ... instead"

## Target shape

The new rule (rough sketch — sonnet refines exact form):

```rust
fn check_join_result_caller_namespace(
    node: &WatAST,
    enclosing_def_name: Option<&str>,  // ":wat::*" or ":user::*" etc.
    errors: &mut Vec<CheckError>,
) {
    // For each ":wat::kernel::Thread/join-result" or ":wat::kernel::Process/join-result" call:
    //   if enclosing_def_name is None or doesn't start with ":wat::":
    //     emit CheckError with message:
    //       "Calling :wat::kernel::{Thread,Process}/join-result from user code is forbidden.
    //        Use :wat::kernel::{Thread,Process}/drain-and-join instead, OR use the bracket combinator
    //        (run-threads / run-processes — when shipped in Stones D/E)."
    ...
}
```

Hook in the walker's existing traversal — wherever the check.rs walker descends into fn bodies, track the enclosing def name and pass it to the new check.

## Implementation protocol (per `feedback_iterative_complexity` + `feedback_test_first`)

1. **Read current state.** `src/check.rs` walker structure; find where verb calls are visited during type-check; find where the arc 117/133 walker fires. Find existing error types + format conventions.

2. **Write 4 tests FIRST.** Add to a new test file `tests/wat_arc170_stone_b_walker_collapse.rs`:
   - **Negative (Thread):** user-namespace fn (e.g., `:my::test::call-thread-join`) calls `Thread/join-result` → check fails with `CheckError` whose message names `Thread/drain-and-join` as the canonical replacement.
   - **Negative (Process):** user-namespace fn calls `Process/join-result` → similar.
   - **Positive (Thread):** `:wat::test::*` (or `:wat::kernel::*`) namespace fn calls `Thread/join-result` → check passes.
   - **Positive (Process):** same for Process.
   RUN tests; CONFIRM all 4 fail (rule not yet implemented).

3. **Implement the new walker check.** Add `check_join_result_caller_namespace` (or equivalent) to `src/check.rs`. Hook into the existing walker traversal where verb-call sites are visited. Track enclosing def namespace during descent. Emit `CheckError` with the helpful message for user-namespace callers.

4. **Build clean + run new tests.** All 4 green.

5. **Sweep existing callers.** Grep the codebase for `Thread/join-result` and `Process/join-result` user-namespace call sites:
   ```bash
   grep -rn "Thread/join-result\|Process/join-result" wat/ wat-tests/ tests/ crates/ src/
   ```
   For each call site, classify:
   - In `:wat::*` namespace (e.g., `wat/test.wat`, `wat/kernel/sandbox.wat`, `wat/kernel/hermetic.wat`): STAYS.
   - In user namespace (e.g., test fixtures in `tests/wat_*.rs` embedding `:my::*` or `:user::*` wat code): MIGRATE to `*_drain-and-join`.
   - In Rust code: STAYS (Rust callers aren't subject to wat-level walker).

6. **Run full workspace test.** `cargo test --release --workspace --no-fail-fast`. Existing tests that hit user-namespace `*_join-result` calls must now use the migrated form OR the new walker rule rejects them at parse time.

7. **Verify workspace baseline maintained.** Baseline at Stone A end: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error). Post-Stone-B should match or improve.

8. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/` per `feedback_no_worktrees` (FM 7-bis).
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs / past SCORE-STONE-* docs / past EXPECTATIONS docs.
- DO NOT modify or retire the existing arc 117/133 scope-deadlock walker — that's Stone G.
- DO NOT modify the existing `eval_kernel_thread_join_result` / `eval_kernel_process_join_result` substrate fns — they STAY user-callable from substrate-namespace code in this stone.
- DO NOT update USER-GUIDE / docs — Stone H handles documentation.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.

## Scorecard (6 rows, YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | New walker check fn defined in `src/check.rs` (binary namespace check on `*_join-result` call sites) | `grep -nA 30 "check_join_result_caller_namespace\|join.result.*namespace\|join.result.*user.namespace" src/check.rs` shows the new fn |
| B | Walker hooked into existing traversal (call sites caught during type-check) | grep shows the call/registration in check.rs's main check fn or wherever traversal happens |
| C | 4 new tests pass — Negative Thread + Negative Process + Positive Thread + Positive Process | `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse` → all green |
| D | Existing user-namespace `*_join-result` callers migrated to `*_drain-and-join` (where applicable) | grep `Thread/join-result\|Process/join-result` shows user-namespace count is zero or fully justified (substrate-namespace only) |
| E | `cargo build --release --workspace --tests` clean | build output Finished |
| F | Workspace test failure count ≤ baseline (Stone A end: 4 target failures — lifeline, t6, totally_bogus, startup_error) | full workspace cargo test failures ≤ 4 |

## STOP triggers

- Walker doesn't have an obvious traversal entry point for tracking enclosing def namespace → STOP and surface; structural refactor may be needed first.
- Existing tests have heavy user-namespace `*_join-result` usage that can't migrate cleanly to `*_drain-and-join` because of semantic differences → STOP; reconsider migration shape.
- Migration breaks more than 5 existing tests → STOP and surface; the new rule may need a phased rollout.
- Walker error format doesn't fit existing CheckError conventions → STOP; surface.
- > 5 unexpected substrate-finding surfaces → STOP; this stone's scope may need decomposition.

## Workspace baseline (commit `2a198bd`)

- `cargo build --release --workspace --tests`: clean per Stone A SCORE
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (`wat::probe_lifeline_pipe_proof` flake, `wat::wat_arc170_program_contracts::t6_spawn_process_factory_with_capture_round_trips`, `wat::test::deftest_wat_tests_tmp_totally_bogus`, `wat-cli::wat_cli::startup_error_bubbles_up_as_exit_3`). All unrelated to this stone.

Post-Stone-B target:
- ≥ baseline + 4 passed (4 new Stone B tests add passes)
- ≤ 4 failed (no regressions; the new rule SHOULD NOT break existing tests because all existing user-namespace `*_join-result` calls migrate to `*_drain-and-join` in step 5)

## Time-box

90-120 min predicted. Hard stop 180 min. If approaching stop, write a partial SCORE describing state-at-stop.

## On completion

Write `SCORE-STONE-B-WALKER-COLLAPSE.md`. 6 rows YES/NO. Honest deltas — especially:
- Where in `check.rs` the walker hook landed; structural pattern (separate fn vs extension of existing walker)
- Namespace classification logic (string-prefix on def name? other mechanism?)
- Error message exact text (was it teaching enough?)
- Caller migration count (how many sites; user-namespace vs substrate-namespace breakdown)
- Workspace test count vs baseline
- Calibration record

## What this stone enables

After Stone B ships:
- User wat code can no longer accidentally call `*_join-result` and fall outside the bracket discipline
- The path forward is: bracket (when minted in Stone D/E) OR `*_drain-and-join` for one-off freestyle cases
- Stone G later retires the arc 117/133 sibling-binding machinery (replaced by this binary rule + bracket semantics)
- Stone F (migrate -with-io callers) builds on top of this constraint — by then, freestyle `*_join-result` is gone from user code

The substrate refuses; users compose; the canonical path becomes the only path.
