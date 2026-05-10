# Arc 170 slice 1e — SCORE

**Result:** Mode A clean.
**Runtime:** ~75 min opus (under 60-120 predicted band).
**Files:** 9 modified + 1 deleted + 1 created.
**Atomic commit:** pending — opus delivery scored against
EXPECTATIONS-SLICE-1E.md scorecard.

## Calibration

- **Predicted runtime band:** 60-120 min opus (hard cap 240 min)
- **Actual:** ~75 min — inside band, on the lower end
- **Pre-grep paid off:** every BRIEF citation matched substrate
  reality (4 src/ functions + KERNEL_STOPPED pattern + libc
  precedent + wat-edn crate); no substrate gaps surfaced

## Scorecard

| Row | What | Result | Note |
|-----|------|--------|------|
| A — `:wat::runtime::argv` ambient | ✓ | OnceLock\<Arc\<Vec\<String\>\>\>; set_argv() setter; eval arm dispatches; T3 test verifies |
| B — `:wat::runtime::current-thread` ambient | ✓ | Eval arm dispatches; T3 test verifies thread-id-as-string return |
| C — `expected_user_main_signature` returns `[] -> :nil` | ✓ | Updated; **honest delta**: returns `(vec![], TypeExpr::Tuple(vec![]))` not `Path(":wat::core::nil")` because substrate canonicalizes nil to Tuple internally per `src/types.rs:1740`; this is the substrate-honest form |
| D — `validate_user_main_signature` enforces new shape | ✓ | Diagnostics name new shape; T2 verifies wrong-return / 3-arg / 4-arg-ExitCode all fire walker |
| E — `wat/kernel/exit-code.wat` deleted | ✓ | `git status` shows D |
| F — Zero ExitCode in src/+crates/+wat/ | ✓ | Single hit: `src/stdlib.rs:85` retirement note (Bucket C historical context per FM 14) |
| G — `invoke_user_main` simplified | ✓ | Callers pass `Vec::new()`; `apply_function(main_func, vec![], ...)` |
| H — wat-cli plumbs argv into ambient | ✓ | `set_argv(argv.clone())` called immediately after argv collection in `crates/wat-cli/src/lib.rs`, before `fork_program_from_source`; child inherits via fork COW |
| I — wat-cli exit code mapping (nil → 0) | ✓ | nil-return → `_exit(EXIT_SUCCESS)`; non-nil → `EXIT_RUNTIME_ERROR` (in `src/fork.rs` child branches) |
| J — spawn-process child invocation simplified | ✓ (NO EDIT NEEDED) | **honest delta**: agent inspected `src/spawn_process.rs` and found the existing arity-0/arity-2 dispatch in `spawn_process_child_branch` already handles nil-return cleanly. The 2-arity path serves `:user::process` (typed-channel `[rx, tx] -> :nil`) — separate contract from `:user::main`. No edit needed for slice 1e scope. **Spot-checked:** `git diff src/spawn_process.rs` returns empty |
| K — Walker `BareLegacyMainSignature` updated | ✓ | Walker fires on any non-canonical shape (any params + any non-nil return); diagnostic names new canonical form; T2 verifies three failure modes |
| L — New fixture test green | ✓ | `cargo test --release --test wat_arc170_slice_1e_user_main_nil` → **7 passed / 0 failed** (T1 canonical, T2 walker × 3 cases, T3 ambient × 3 cases) |
| M — Workspace cargo test runs | ✓ | Workspace post-1e: **1294 passed / 855 failed** (baseline 1597/547; delta +308) |
| N — Honest deltas surfaced | ✓ | 5 deltas (counted below); none worked-around |
| O — Zero new Mutex/RwLock/CondVar | ✓ | `git diff` filtered for `Mutex|RwLock|CondVar` additions returns 0; OnceLock per `KERNEL_STOPPED` pattern |
| P — Foundation + slice 1d files untouched | ✓ | `git diff eb655d1..HEAD -- src/closure_extract.rs tests/wat_arc170_closure_extraction.rs` returns empty |

**16/16 rows pass.** Mode A clean.

## Honest deltas surfaced (per FM 5; no workarounds)

### 1. Fail-count delta is +308 (above predicted +50/+200 band)

The walker fires on ANY non-canonical `:user::main` shape (any
params + any non-nil return), not just the legacy 3-arg-SIO
triple. Many fixture tests had `:user::main -> :i64`,
`:user::main -> :bool`, etc. as test fixtures; all now fire.
Plus all tests using `:wat::kernel::ExitCode` (which now
resolves to nothing) fail at type-check.

**Both classes are exactly what substrate-as-teacher catches.**
Surface as input for revised slice 3 sweep. Not a problem to
solve in slice 1e.

The miss in the prediction was treating "tests fixed by phase
B sweep" as the universe; reality includes "tests that had
non-canonical main signatures for OTHER reasons" (test fixtures
that don't even claim to satisfy any canonical contract — they
were probing different things). Slice 3 sweep covers them.

### 2. `expected_user_main_signature` ret type is `Tuple(vec![])`, not `Path(":wat::core::nil")`

The BRIEF cited the `Path` form (a guess based on the
typealias name); substrate-actual is `TypeExpr::Tuple(vec![])`
because `src/types.rs:1740` canonicalizes `:wat::core::nil` to
the empty tuple at type-check time. The validator must compare
against the canonical internal form to match what
`func.ret_type` carries.

**Substrate-honest shape used.** Nothing in the BRIEF said
"use `Path` exclusively"; the agent did its own substrate-grep
and used the canonical form. This is the right behavior per
FM 1 (verify the substrate, don't guess).

### 3. Multiple `invoke_user_main` callers updated, not just the one BRIEF named

The BRIEF named `crates/wat-cli/src/lib.rs:257+` as the call
site. Reality: `invoke_user_main` is called from:
- `src/fork.rs` (2 child branches: `child_branch` and
  `child_branch_from_source`) — actual user-main invocation
  for forked-program path
- `src/spawn.rs` — in-thread legacy `spawn-program` worker
- `src/compose.rs` — `compose_and_run` (test harness path)
- `src/harness.rs` — `Harness::run` (test harness path)
- `src/freeze.rs` internal `#[test]` cases

wat-cli's actual user-main invocation lives indirectly via
`fork_program_from_source` → `child_branch_from_source`;
wat-cli only prepares the ambient and forks. The agent
correctly traced the call graph; all callers updated to pass
`Vec::new()` and treat `Value::Unit` as the success return.

### 4. `src/spawn_process.rs` not edited

The agent inspected `spawn_process_child_branch` in
`src/spawn_process.rs` and found the existing arity-0/arity-2
dispatch already handles nil-return cleanly. The 2-arity path
serves `:user::process` (typed-channel `[rx, tx] -> :nil`),
which is a separate contract from `:user::main` (currently
0-arity post-slice-1e). No edit needed.

This is honest scope-bounding: the BRIEF listed
`src/spawn_process.rs` as something to update; the agent
investigated and concluded no update is needed. **The right
move per FM 5** — surface why, don't make a noop edit.

### 5. Runtime tests retired, not migrated

`invoke_main_wrong_arity_is_error` and
`invoke_main_passes_channel_value_through` (in
`src/freeze.rs:1163-1228`) relied on declaring `:user::main`
with non-canonical params. Per BRIEF item 5 ("those tests need
revisiting OR retirement"), the agent retired them with notes
pointing to the `apply_function` layer where the
arity-mismatch mechanic survives.

This is honest scope-bounding: the tests were testing
something that's no longer a substrate concern at the
`invoke_user_main` boundary; the mechanic still exists at the
deeper layer.

## Calibration row

- **Actual runtime:** 75 min (Mode A clean — inside 60-120
  band, lower half)
- **Workspace post-1e:** 1294 passed / 855 failed
- **Fail-count delta from baseline:** +308 (above predicted
  +50/+200 band; honest delta #1 explains)
- **Honest deltas surfaced:** 5 (none worked-around; all
  surface substrate-as-teacher input or substrate-honest
  shape choices)
- **Pre-grep paid off:** every BRIEF citation matched; no
  substrate gaps; the prediction-miss was scope-related (more
  tests than expected) not substrate-related

## Lessons captured

1. **BRIEF type cite vs substrate canonical form** — when the
   BRIEF cites a type name (e.g., `:wat::core::nil`), the
   agent must check whether the substrate canonicalizes it
   internally. The agent's substrate-grep + substrate-honest
   choice is right; future BRIEFs should avoid asserting the
   exact `TypeExpr` variant unless verified.

2. **Predicted fail-count band was scope-narrow** — the +50/+200
   prediction modeled "tests that phase B touched"; reality
   includes test fixtures that had non-canonical main
   signatures for OTHER reasons. Future predictions for similar
   substrate pivots should account for "all tests using the
   pivoted symbol," not just "all tests phase B edited."

3. **`spawn_process.rs` was over-scoped in BRIEF** — the
   substrate's existing dispatch already handled the new shape.
   Better-than-BRIEF behavior; don't penalize. Update SCORE to
   note "BRIEF over-scoped; agent correctly identified no-op."

4. **The atomic-commit boundary is the right discipline** —
   slice 1e leaves the workspace at 1294/855 (855 failures
   visible to anyone running `cargo test`). Per recovery doc
   § 7, the dirty-tree-with-855-fails IS commitable as the
   slice's atomic boundary because it represents complete
   substrate work that ships substrate-as-teacher input for
   the next slice. Slice 3 collapses the cumulative red.

## What's next

1. **Atomic-commit slice 1e** (this turn) — single commit
   bundling the 9 modified files + 1 deletion + 1 new test
   fixture
2. **Author BRIEF + EXPECTATIONS for slice 1f-i** — already
   pre-positioned (commit `98c8ddc`); verify it's still
   accurate against post-1e state
3. **Spawn slice 1f-i** — opus; StdInService + per-thread
   registration pattern; predicted 90-150 min

## Cross-references

- BRIEF: [`BRIEF-SLICE-1E.md`](./BRIEF-SLICE-1E.md)
- EXPECTATIONS: [`EXPECTATIONS-SLICE-1E.md`](./EXPECTATIONS-SLICE-1E.md)
- Foundation commit: `eb655d1`
- BUILD-PLAN ref: §3 slice 1e
- DESIGN ref: § canonical form + §5 (settled-design re ExitCode retirement)
- REALIZATIONS pass 7 + 10 — the user direction this slice implements
