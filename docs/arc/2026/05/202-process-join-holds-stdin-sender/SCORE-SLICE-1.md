# Arc 202 Slice 1 SCORE

**Date:** 2026-05-16
**Executor:** Claude Sonnet 4.6

## Score rows

| Row | What | Result | Evidence |
|-----|------|--------|----------|
| A | `CheckError::ProcessJoinHoldsStdinSender` minted (variant + Display + Diagnostic) | YES | `src/check.rs` — variant after `ProcessJoinBeforeOutputDrain`, Display arm mirrors Gap K's one-liner, Diagnostic arm with 3 fields |
| B | Walker rule fires on the documented deadlock shape | YES | `process_join_without_stdin_extraction_fails_check` passes; error contains `ProcessJoinHoldsStdinSender` and `proc` |
| C | Walker rule does NOT fire on the canonical legal shape | YES | `process_join_with_stdin_extraction_passes_check` passes; full stdlib (including fixed `run-hermetic-driver`) loads cleanly |
| D | `wat/test.wat` `run-hermetic-driver` updated; `ast_entry_prints_hello` no longer hangs | YES | `cargo test --release -p wat --test wat_run_sandboxed_ast` completes in 0.03s; both tests ok |
| E | Workspace failure count ≤ baseline (4); no new failures introduced | YES | 4 failures = same 4 pre-existing (`lifeline_pipe_zero_orphans_across_100_trials`, `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`) |

**All 5 rows: YES.**

## Honest deltas

### Detection mechanism chosen

**Option β only (absence detection).** When the outer let's scope tree is scanned with `find_process_join_holds_stdin_sender`, the rule fires when `Process/join-result proc` is found but `Process/stdin proc` does NOT appear anywhere in the tree. This directly catches the `run-hermetic-driver` case.

Option α (extend `collect_process_calls` to also track stdin as an "accessor") was evaluated and rejected. The Gap K co-presence logic wouldn't apply to the stdin case — the BRIEF's option α would catch sibling bindings `[stdin-w (Process/stdin p) joined (Process/join-result p)]`, but option β catches this too (stdin is present → rule silent). More importantly, option α alone would NOT catch the absent-stdin case that caused the actual hang.

### Inner-scope detection precision (false negative acknowledged)

The v1 rule is absence-only: if `Process/stdin proc` appears ANYWHERE in the scope tree, the rule does not fire — even if `Process/stdin` is a sibling binding at the outer let level (where the Sender technically stays alive at join time). This is a deliberate v1 trade-off:

- **True positives caught:** absent-stdin shape (the canonical deadlock — `run-hermetic-driver`'s pre-fix form).
- **False negatives admitted:** sibling-binding shape where `Process/stdin` is extracted but NOT explicitly closed before join. In practice, the sibling shape only appears in `run-hermetic-with-io-driver` (where the child exits via typed I/O disconnect, not fd 0 EOF) and `drive-sandbox` (which explicitly calls `IOWriter/close` before the inner scope). Both are structurally safe.

Test 3 (`process_join_with_stdin_present_does_not_fire_stdin_rule`) documents this boundary explicitly — `ProcessJoinHoldsStdinSender` does not fire when stdin is present, but arc 198's restriction still fires for user-namespace callers.

### Vector traversal — critical implementation finding

`collect_process_calls` (Gap K's collector) only recurses into `WatAST::List` nodes. This is correct for Gap K because the conflicting output accessor calls (`Process/stdout proc`) appear as List-form RHS values at the OUTER let's binding level.

My `collect_process_stdin_and_joins` initially mirrored this List-only pattern — causing the new rule to falsely fire even after the wat-side fix, because `(:wat::kernel::Process/stdin proc)` inside the inner let's `WatAST::Vector` bindings was invisible to the scanner.

**Fix:** `collect_process_stdin_and_joins` recurses into BOTH `WatAST::List` and `WatAST::Vector` nodes. This is correct because the inner let's binding vector `[stdin-w (Process/stdin p) ...]` is a `WatAST::Vector`, and the scanner must descend into it to find the stdin extraction. Gap K didn't need this because it looks for co-presence at the same scope level; absence detection requires scanning the full subtree.

This is NOT a defect in Gap K — the two rules detect structurally different shapes.

### Gap K mechanism vs BRIEF description

The BRIEF's description of Gap K is accurate:
- `collect_process_calls` scans for joins AND accessors (stdout/stderr/output) in the let's synthetic scope node
- `find_process_join_before_drain` pairs them by process identifier
- Hook in `check_let` builds `let_scope_items` from binding RHS values + body, NOT from a flat walk of the whole form

One nuance the BRIEF didn't emphasize: `let_scope_items` is built by pushing `items[1]` (the RHS expression) from each `(name expr)` binding pair. This means the scanner only sees the RHS values, not the binding names themselves. For both rules this is correct — the forbidden calls appear in RHS positions, not as binding names.

### Other wat helpers found

Searched all `wat/` files for `Process/join-result`. Found 5 call sites:

1. `wat/test.wat:541` — `run-hermetic-driver` — **FIXED in-slice** (added `stdin-w` to inner let).
2. `wat/test.wat:936` — `run-hermetic-with-io-driver` — `Process/stdin proc` appears at line 921 as a sibling binding `tx (:wat::kernel::Sender/from-pipe (:wat::kernel::Process/stdin proc))`. Rule does not fire (stdin is present). Structurally safe: child exits via typed I/O disconnect.
3. `wat/kernel/sandbox.wat:108` — `drive-sandbox` — `stdin-w (:wat::kernel::Process/stdin proc)` at line 90, explicitly closed via `IOWriter/close` before inner scope. Rule does not fire. Structurally safe.
4. `wat/kernel/hermetic.wat:146` — `run-sandboxed-hermetic-ast` — `stdin-wr (:wat::kernel::Process/stdin proc)` in an inner let before the outputs scope. Rule does not fire. Structurally safe.

**Only `run-hermetic-driver` was broken. All other helpers had correct stdin discipline already.**

### Workspace pass count

- Baseline: 2319 passed / 4 failed
- Post-slice: 2322 passed / 4 failed
- Delta: +3 (3 new arc202 tests) + the previously-hung `ast_entry_prints_hello` now completes cleanly (it was hung, not counted in baseline)
- The intermittent `deftest_wat_tests_std_test_test_run_hermetic_with_prelude_proof` failure seen once in a full workspace parallel run was transient (passes reliably when run in isolation); not a regression.

### Substring collision check

No naming collisions found. `ProcessJoinHoldsStdinSender` is distinct from all existing variant names. `collect_process_stdin_and_joins` and `find_process_join_holds_stdin_sender` are new unique function names.

## Files changed

- `src/check.rs` — variant + Display + Diagnostic + finder function + hook (~90 LOC added)
- `wat/test.wat` — `run-hermetic-driver` inner let: added `stdin-w (:wat::kernel::Process/stdin proc)` line + updated comment (~4 LOC changed)
- `tests/wat_arc202_process_join_holds_stdin.rs` — new test file, 3 tests (~140 LOC)

Total: ~234 LOC across 3 files (estimated 160 in BRIEF; actual larger due to Vector-traversal fix and comprehensive test documentation).
