# Arc 202 Slice 1 BRIEF — `ProcessJoinHoldsStdinSender` walker rule

**Phase:** Single slice. Substrate gap surfaced by hung workspace test on 2026-05-16; closing it with a freeze-time refusal that mirrors Gap K's `ProcessJoinBeforeOutputDrain` (arc 170 slice E).

**Originating diagnostic:** `tests/wat_run_sandboxed_ast.rs::ast_entry_prints_hello` hung the workspace `cargo test` run launched 17:33 for 35+ min. `/proc/<pid>/wchan` traced the deadlock: parent's `Process/join-result proc` blocks waiting for child; child's structural `StdInService` blocked on `read(fd 0)` waiting for EOF; parent never closed write-end of child stdin pipe. All parties alive; no shutdown event; the existing FD-multiplex "you cannot escape" guarantees don't apply.

The expression that enabled it: `wat/test.wat:515-565` `run-hermetic-driver` extracts stdout-r + stderr-r in an inner scope (drains before outer join) but never touches `Process/stdin proc`. The proc handle carries the stdin Sender for the entire outer-let scope; join runs while it's still held.

## Goal

Mint `CheckError::ProcessJoinHoldsStdinSender` + walker rule that fires when `(:wat::kernel::Process/join-result <p>)` appears in a let-form scope WITHOUT a preceding `(:wat::kernel::Process/stdin <p>)` extraction in an inner-scoped let (where the extracted Sender would drop before join).

Mirror Gap K's machinery exactly. Then fix `wat/test.wat:524-533` `run-hermetic-driver`'s inner let to extract `stdin-w` so the existing hung test passes and the new rule is satisfied for substrate-side wat helpers.

## Required path (NO new substrate types/structs/special-forms/verbs)

This slice adds:
- 1 new `CheckError` variant (`ProcessJoinHoldsStdinSender`)
- Display + Diagnostic impl for the new variant (mirror Gap K's existing impls)
- Extension to `collect_process_calls` (or sibling helper) to track stdin extractions
- New finder fn (e.g., `find_process_join_holds_stdin_sender`) OR extension of existing pairing logic
- Wire detection into `check_let` next to the existing Gap K hook
- 3-line edit to `wat/test.wat:524-533` (add `stdin-w` to inner let)
- 1 new test file proving the rule fires + the inner-scope-extraction satisfies it

NO runtime substrate change. NO new primitive. NO new type. NO Layer A auto-close at `Process/join-result` (the alternative was disqualified on Honest per DESIGN.md § Four questions). Pure freeze-time refusal.

## Implementation hint (sonnet verifies + adjusts)

**Reference precedent:** `ProcessJoinBeforeOutputDrain` exists at:
- Variant: `src/check.rs:192-206`
- Display impl: `src/check.rs:724-731` (one-line giant string)
- Diagnostic impl: `src/check.rs:1047-1066` area
- Collector: `src/check.rs:3502-3530` `collect_process_calls`
- Finder: `src/check.rs:3476-3500` `find_process_join_before_drain`
- Hook site: `src/check.rs:7137-7165` (inside the let-check)

Mirror this shape. Sonnet picks between:
- **(α) Extend `collect_process_calls`** to also collect `:wat::kernel::Process/stdin` into accessors. Then existing pairing detects (join, stdin) at sibling-binding level. Does NOT cover the run-hermetic-driver-style absent-stdin case.
- **(β) Add a parallel `find_process_join_holds_stdin_sender`** that scans for joins and verifies an `Process/stdin <same-proc>` extraction exists in a nested inner scope before the join. Covers the run-hermetic-driver case + any future macro that forgets.

**Likely both needed:** (α) catches sibling-binding shape (proc + stdin-w + join all in same let → stdin-w drops with proc at outer-let exit, AFTER join); (β) catches absent-stdin shape (proc + join with no stdin touch anywhere). The diagnostic naming should be consistent for both.

**Inner-scope detection:** the run-hermetic-driver pattern is the canonical legal shape — `(Process/stdin proc)` appears inside a nested `(:wat::core::let ...)` whose RHS is a binding's value in the outer let, evaluating before the join binding. The walker may treat "inner-scoped extraction" as "appears within a nested let whose terminal expression closes before the join's position in the bindings list."

If finder logic gets gnarly, sonnet surfaces in SCORE; simpler check ("ANY `Process/stdin <p>` call exists in the same let-scope tree before the join") may be acceptable v1 even if it admits some false negatives. Capture trade-off in SCORE Honest deltas.

## Wat-side fix (in-slice; non-optional)

`wat/test.wat:524-533` `run-hermetic-driver`'s inner let:

```scheme
(:wat::core::let
  [stdin-w        (:wat::kernel::Process/stdin proc)   ;; ← ADD
   stdout-r       (:wat::kernel::Process/stdout proc)
   stderr-r       (:wat::kernel::Process/stderr proc)
   stdout-lines   (:wat::kernel::drain-lines stdout-r)
   stderr-lines   (:wat::kernel::drain-lines stderr-r)]
  (:wat::core::Tuple stdout-lines stderr-lines))
```

stdin-w drops alongside stdout-r/stderr-r at inner-let exit → child's StdInService sees EOF → child exits → outer join returns. Verify: `wat_run_sandboxed_ast::ast_entry_prints_hello` no longer hangs.

If sonnet finds OTHER substrate-side wat helpers with the same shape (search wat/ for `Process/join-result` calls), fix them similarly. Run-thread-driver does NOT have this concern (threads, not processes; different transport per FM 7-ter).

## Tests

`tests/wat_arc202_process_join_holds_stdin.rs` (sonnet picks final name):

1. `process_join_without_stdin_extraction_fails_check` — minimal program with proc binding + `Process/join-result proc` but NO `Process/stdin proc` anywhere → check returns `ProcessJoinHoldsStdinSender` with both spans correct.
2. `process_join_with_stdin_extraction_in_inner_scope_passes_check` — analogous to run-hermetic-driver fix shape; check succeeds; no false positive.
3. `process_join_with_sibling_stdin_binding_fails_check` (per option α coverage; optional based on rule shape) — `[proc ... stdin-w (Process/stdin proc) joined (Process/join-result proc)]` → check fires because stdin-w drops at SAME scope as join, not before. Captures the subtle case.
4. Existing `wat_run_sandboxed_ast::ast_entry_prints_hello` passes (post-driver-fix verification).

## Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_arc202_process_join_holds_stdin  # new
cargo test --release -p wat --test wat_run_sandboxed_ast  # verifies hang resolved
cargo test --release --workspace --no-fail-fast  # baseline preservation
```

## Workspace baseline (commit `ecc876a`, captured 2026-05-16)

- 2319 passed / 4 failed
- 4 known pre-existing failures: `lifeline_pipe_zero_orphans_across_100_trials` (FD-multiplex flake), `deftest_wat_tests_tmp_totally_bogus` (unresolved reference), `t6_spawn_process_factory_with_capture_round_trips` (arc 170 slice 6 documented gap), `startup_error_bubbles_up_as_exit_3` (wat-cli pre-existing)

Post-slice-1 target:
- Pass count ≥ 2319 + 3-4 new (the new tests; the originally-hung test now passes cleanly)
- Fail count ≤ 4 (no regressions)

**Discipline note (orchestrator decay disclosure):** orchestrator's mental model of the existing Gap K rule's exact behavior — particularly WHY it doesn't already fire on the run-hermetic-driver's nested-let-with-output-accessors shape — is incomplete. Sonnet verifies the actual machinery against the source before relying on orchestrator's description. If the rule's actual scope is different from "let-form syntactic tree recursion," surface in SCORE.

## STOP triggers (true emergencies — surface, do not paper over)

1. **Gap K's actual detection logic differs from the BRIEF's description** — surface what you found. The decay-disclosure above acknowledges orchestrator's model may be partial. If detection is more nuanced (e.g., only fires on direct let-binding siblings, not recursive children), update the mirror accordingly.
2. **A different wat helper file uses the same deadlock-shape** — surface + fix in-slice (one extra ~3-line edit).
3. **Existing passing tests start failing after the rule lands** — STOP. Either (a) the rule has false positives — refine the rule, OR (b) other wat helpers had this latent deadlock — fix in-slice with explicit acknowledgment.
4. **Implementation surfaces a substrate gap in how stdin Sender flows from spawn-process to Process struct** — e.g., the Sender isn't actually held by proc; in that case the deadlock has a different root cause and the BRIEF's framing needs revision. STOP, surface diagnostic.
5. **Workspace baseline regresses** — fail count > 4 or pass count drops materially. STOP, surface diff.
6. **Any urge to ship Layer A (runtime auto-close at `Process/join-result`) instead of Layer B (walker rule)** — STOP. Layer A was explicitly disqualified per DESIGN.md § Four questions. Honest YES requires freeze-time refusal.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may report `.claude/worktrees/agent-<id>/` paths — ignore; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT modify arc 202 DESIGN.md (orchestrator owns).
- DO NOT modify Gap K's existing `ProcessJoinBeforeOutputDrain` variant, finder, or hook. New rule is ADDITIVE — strictly alongside Gap K, never replacing.
- DO NOT mint a runtime auto-close (no eval-handler change to `Process/join-result`). Freeze-time refusal only.
- DO NOT touch historical artifacts (past BRIEFs/SCOREs/INSCRIPTIONs for prior arcs; INTERSTITIAL-REALIZATIONS; recovery doc).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `CheckError::ProcessJoinHoldsStdinSender` minted (variant + Display + Diagnostic) | grep `ProcessJoinHoldsStdinSender` in src/check.rs returns variant + Display + Diagnostic arms |
| B | Walker rule fires on the documented deadlock shape | `process_join_without_stdin_extraction_fails_check` test passes (rule fired with correct spans) |
| C | Walker rule does NOT fire on the canonical legal shape | `process_join_with_stdin_extraction_in_inner_scope_passes_check` test passes (no false positive) |
| D | `wat/test.wat` `run-hermetic-driver` updated; `wat_run_sandboxed_ast::ast_entry_prints_hello` no longer hangs | targeted test run completes in expected time (seconds, not minutes) |
| E | Workspace failure count ≤ baseline (4); no new failures introduced | full workspace cargo test failure count ≤ 4 |

## Honest deltas to capture in SCORE

- **Detection mechanism chosen.** Option (α) only / (β) only / both. Surface why.
- **Inner-scope detection precision.** Exact rule shape (recursive descent vs structural nesting check). False-positive / false-negative trade-offs.
- **Other wat helpers with the same shape.** Were any found beyond run-hermetic-driver? Were they fixed in-slice?
- **Gap K mechanism differences.** Did the orchestrator's description of Gap K match the actual code? If not, what differed?
- **Substring corruption check.** Did the new variant naming collide with any existing identifier? (Should be 0; SoP STOP-trigger.)

## Time-box

60-90 min predicted. Hard stop 120 min.

## On completion

1. Write `docs/arc/2026/05/202-process-join-holds-stdin-sender/SCORE-SLICE-1.md` per § SCORE methodology + § Honest deltas.
2. Return final summary: rows passed/failed + workspace baseline delta + detection mechanism chosen + run-hermetic-driver fix verified + any other wat helpers found.

You are launching now. T-minus 0.
