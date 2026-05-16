# Arc 198 Slice 2 Stone 4 SCORE — loop closure: delete Stone B's ad-hoc rule + update tests

**BRIEF:** `BRIEF-STONE-4-LOOP-CLOSURE.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-4-LOOP-CLOSURE.md`
**Predecessors:**
- Stone 1 (commit `51c69a1`) — inventory wiring shipped
- Stone 2 (commit `6775510`) — proc-macro attribute shipped
- Stone 3 (commit `fe2e0eb`) — attribute applied to `eval_kernel_*_join_result`; both walkers fire

**Successor:** arc 170 Stone C (Client/Server type pairs) — bracket chain resumes after this loop closure.

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `validate_join_result_user_namespace` fn DELETED | **YES** | `grep -n "validate_join_result_user_namespace" src/check.rs` → only two doc-comment references survive (line 1959 inside an arc 198 hook comment recording the historical deletion; line on the deleted fn's mention is gone). No fn definition, no call site. |
| B | `CheckError::JoinResultUserNamespace` variant DELETED (variant + Display + Diagnostic) | **YES** | `grep -n "JoinResultUserNamespace" src/check.rs` → only one doc-comment reference at line 627 ("Subsumes arc 170 Stone B's hard-coded `JoinResultUserNamespace`"). Variant body, Display arm, Diagnostic arm all gone. |
| C | Stone B walker hook in `check_program` REMOVED | **YES** | The pre-Stone-4 loop `for (name, func) in sym.functions.iter() { validate_join_result_user_namespace(&func.body, name, &mut errors); }` (formerly at lines 2023-2025) replaced with an explanatory comment block; the surviving arc 198 walker loop is the sole call site. |
| D | Stone B's 4 tests pass with arc 198's error format assertions | **YES** | `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse` → `test result: ok. 4 passed; 0 failed`. The two negative tests assert on `DefRestrictedCallerNotAllowed` (the arc 198 variant name in the Debug output); the two positive tests are unchanged and still pass. |
| E | All other predecessor tests still green (Stone A, arc 198 slice 1, Stone 1, Stone 2, Stone 3) | **YES** | `wat_arc170_stone_a_drain_and_join` 4/4; `wat_arc198_def_restricted` 5/5; `wat_arc198_slice2_stone_1_inventory_wiring` 1/1; `wat_arc198_slice2_stone_2_attribute` 3/3; `wat_arc198_slice2_stone_3_apply` 2/2. Plus Stone B 4/4 (above). **Total 19/19 across all six stones.** |
| F | Workspace test failure count ≤ baseline (3 stable + flake variance) | **YES** | `cargo test --release --workspace --no-fail-fast` → `error: 4 targets failed`. Per-target failures: `lifeline_pipe_zero_orphans_across_100_trials` (known flake, hit this run), `deftest_wat_tests_tmp_totally_bogus` (pre-existing), `t6_spawn_process_factory_with_capture_round_trips` (pre-existing), `startup_error_bubbles_up_as_exit_3` (pre-existing). 3 stable + 1 flake = baseline exactly. **Zero new failures.** |

**6/6 PASS.**

## Honest deltas

### Test-assertion substring choice — `DefRestrictedCallerNotAllowed` (not `allowed-caller whitelist`)

The BRIEF suggested replacing "drain-and-join" with arc 198's wording such as "def-restricted binding" or "allowed-caller whitelist". The first attempt at `assert!(err.contains("allowed-caller whitelist"))` FAILED with the diagnostic:

```
got: Check(CheckErrors([JoinResultUserNamespace { ... canonical: ":wat::kernel::Thread/drain-and-join" ... },
                       DefRestrictedCallerNotAllowed { callee: "...", prefixes: [":wat::"], ... }]))
```

**Root cause:** the test uses `format!("{:?}", e)` to render the error bundle. Debug format prints struct field NAMES and values, never the Display message. Arc 198's "allowed-caller whitelist" phrase lives only in the Display impl; the Debug output contains struct fields (`callee`, `prefixes`, `enclosing_fn`). The substring unique to arc 198 in the Debug rendering is the variant name itself: `DefRestrictedCallerNotAllowed`. Switched to this; all 4 tests green pre-deletion (both walkers firing) AND post-deletion (only arc 198's walker firing).

This is the correct substring for the test's chosen rendering. If the test fixture ever shifts to `format!("{}", e)` (Display rendering), the assertion will still match (the Display impl is `write!(f, "...DefRestrictedCallerNotAllowed..."?)` — no, actually Display doesn't print the variant name, but tests on Display can grep "allowed-caller whitelist" instead). The chosen substring is durable under the CURRENT Debug rendering.

### Substrate state pointers — verified line numbers

BRIEF estimates → actual locations (verified pre-deletion via grep):
- `validate_join_result_user_namespace` fn: BRIEF said ~3094, actual was lines 3180–3215 (drift +86, likely from intervening arcs)
- `CheckError::JoinResultUserNamespace` variant: BRIEF said ~667, actual was lines 619–653 (drift -14 to -48)
- Hook in `check_program`: BRIEF said ~1939, actual was lines 2007–2025 (drift +66 to +86)
- Helper `walk_for_join_result_call`: lines 3232–3267 (immediately following the fn)
- Const `STONE_B_FORBIDDEN_VERBS`: lines 3221–3230 (between fn + walker)

All deletions accomplished cleanly via Edit's exact-string matching; grep-then-Read confirmed boundaries before each deletion.

### Orphaned imports / helpers — NONE

Stone B's rule used `WatAST` and `Span` directly via path qualification (no separate imports). Const + walker fn were exclusively used by the deleted `validate_join_result_user_namespace` — both removed in the same delete. Post-deletion build emits exactly the same 5 wat-lib warnings as the pre-deletion baseline:

```
warning: function `parse_fn_signature_for_check` is never used
warning: function `eval_kernel_process_send` is never used
warning: function `eval_kernel_process_recv` is never used
warning: function `process_died_error_entry_form_failure` is never used
warning: function `process_died_error_entry_form_failure_value` is never used
```

All 5 are pre-existing dead-code warnings on unrelated functions; none mention `JoinResultUserNamespace` / `validate_join_result_user_namespace` / `walk_for_join_result_call` / `STONE_B_FORBIDDEN_VERBS`. **Zero orphan warnings introduced by the deletion.**

### LOC deleted

Stone B's ad-hoc rule footprint:
- Variant + docblock (lines 619–653): 35 lines
- Display arm (lines 890–899): 10 lines
- Diagnostic arm (lines 1250–1259): 10 lines
- Hook (lines 2007–2025, including comment): ~19 lines (collapsed into a comment-only block referencing the new walker as sole enforcement)
- Fn body + docblock (lines 3180–3215): 36 lines
- Const + walker fn (lines 3217–3267): 51 lines

**Gross deletion: ~161 lines.** Net (after substituting the explanatory comment in check_program and the dochint corrections to arc 198's existing docs): approximately **140 lines removed, ~16 lines of new explanatory commentary added** for historical-context preservation (the "Subsumes arc 170 Stone B..." doc updates in `DefRestrictedCallerNotAllowed`'s variant docblock and `walk_for_def_restricted_call`'s fn docblock).

### Test-file docblock update

`tests/wat_arc170_stone_b_walker_collapse.rs`'s module-level docblock and two negative-test docblocks updated to record:
- The enforcement contract is UNCHANGED (same binary rule, same exempt namespace)
- The mechanism shifted from Stone B's hard-coded walker to arc 198's generic `walk_for_def_restricted_call` (triggered by `#[restricted_to(...)]` on the callees)
- Stone B's tests now verify the SAME contract via the NEW mechanism — history preserved, no semantic regression

### Doc-comment historical references preserved

Three doc comments still mention Stone B's deleted artifacts by NAME (intentionally):
1. `CheckError::DefRestrictedCallerNotAllowed`'s docblock (line 627): "Subsumes arc 170 Stone B's hard-coded `JoinResultUserNamespace` rule (deleted in arc 198 slice 2 Stone 4)..."
2. `check_program` walker loop comment (line 1959): "...Subsumes arc 170 Stone B's hard-coded `validate_join_result_user_namespace` rule (deleted in arc 198 slice 2 Stone 4...)"
3. `walk_for_def_restricted_call`'s docblock (line 3118): "Subsumes arc 170 Stone B's deleted `walk_for_join_result_call` — Stone B's rule was one hard-coded restriction; arc 198 lets any binding declare its own whitelist at the binding site."

This preserves the historical genealogy: a future maintainer encountering arc 198's walker can trace the lineage back to Stone B without having to read git history. The references are documentation, not code (`grep -n` confirms no fn / variant / call site remains).

## Calibration record

### Prediction vs. observation

| Prediction | Observed | Calibration |
|-----------|----------|-------------|
| Stone B's tests might pass pre-deletion (both walkers fire, combined output has both wordings) | First assertion `allowed-caller whitelist` FAILED pre-deletion because the test uses Debug format, not Display | **Partial miss** — the Debug-vs-Display rendering subtlety wasn't predicted; the substring choice required one iteration. Caught via test-first discipline + observation, not theorizing. |
| Deletion would yield zero orphan imports | Confirmed — 5 pre-existing warnings, zero new ones | **Hit** |
| Predecessor 19/19 stays green | Confirmed: 4+5+1+3+2+4 = 19/19 | **Hit** |
| Workspace failures = 3 stable + 1 lifeline flake | Confirmed: 4 targets failed, exact baseline match | **Hit** |
| 45 min predicted, 90 min hard stop | Actual: ~25 min wall (grep + read + 4 edits + 2 cargo builds + workspace test) | **Hit** (under predicted budget) |

### Test-first discipline upheld

Sequence:
1. Updated 4 test assertions FIRST (before any deletion)
2. Ran tests → 2 FAILED with the first substring choice
3. Read the actual Debug output to choose a valid substring
4. Updated assertion → tests PASSED with both walkers still firing
5. THEN deleted the Stone B walker code
6. Ran tests → still PASSED with only arc 198's walker firing

The intermediate "tests pass with BOTH walkers" was the proof point — it demonstrated the test assertions were independent of Stone B's residual output, so deletion was safe.

### Failure surface caught

The "Debug rendering shows struct names, not Display message" gotcha is the kind of substrate-fact failure the orchestrator's decay disclosure warned about. The test-first workflow caught it in <30 seconds (run, read, switch substring); a code-first workflow (delete then update tests) would have hit the same gotcha but with the rule already deleted — harder to roll back. **Test-first paid off.**

### Substrate cleanup completeness

`grep -rn "JoinResultUserNamespace\|validate_join_result_user_namespace\|walk_for_join_result_call\|STONE_B_FORBIDDEN_VERBS" --include="*.rs"` returns ONLY the three intentional doc-comment references. No source code references survive; no test references survive (Stone B's tests moved to arc 198 assertions). The deletion is complete and surgical.

## Workspace verification

- `cargo build --release --workspace --tests`: clean (56s, 5 pre-existing warnings)
- `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse`: 4/4 green
- `cargo test --release -p wat --test wat_arc198_def_restricted`: 5/5 green
- `cargo test --release -p wat --test wat_arc198_slice2_stone_1_inventory_wiring`: 1/1 green
- `cargo test --release -p wat --test wat_arc198_slice2_stone_2_attribute`: 3/3 green
- `cargo test --release -p wat --test wat_arc198_slice2_stone_3_apply`: 2/2 green
- `cargo test --release -p wat --test wat_arc170_stone_a_drain_and_join`: 4/4 green
- `cargo test --release --workspace --no-fail-fast`: 4 targets failed = baseline (3 stable + 1 lifeline flake), zero new failures

## Closure summary

Arc 198 slice 2 Stone 4 closes the loop: arc 170 Stone B's ad-hoc walker rule (one hard-coded restriction) is now subsumed by arc 198's generic `walk_for_def_restricted_call` (any binding declares its own whitelist). The two `*_join-result` callees carry `#[restricted_to(":wat::kernel::*_join-result", ":wat::")]` per Stone 3 and are enforced via the generic walker.

**Arc 198 slice 2 is COMPLETE.** Stones 1 → 2 → 3 → 4 all green; substrate has one walker (not two) enforcing `:wat::core::def-restricted`; Stone B's enforcement contract preserved unchanged; Stone B's tests preserved unchanged in spirit (updated only to assert on the new mechanism's variant name).

Arc 198 itself is ready for INSCRIPTION at a separate-slice boundary; arc 170 Stone C (Client/Server type pairs) is the next bracket-chain stop.
