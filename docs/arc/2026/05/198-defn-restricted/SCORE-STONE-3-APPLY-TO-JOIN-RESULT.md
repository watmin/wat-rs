# Arc 198 Slice 2 Stone 3 SCORE — apply `#[restricted_to(...)]` to `eval_kernel_*_join_result`

**BRIEF:** `BRIEF-STONE-3-APPLY-TO-JOIN-RESULT.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-3-APPLY-TO-JOIN-RESULT.md`
**Predecessors:**
- Stone 1 (commit `51c69a1`) — inventory wiring shipped
- Stone 2 (commit `6775510`) — proc-macro attribute shipped

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `#[restricted_to(":wat::kernel::Thread/join-result", ":wat::")]` applied to `eval_kernel_thread_join_result` | **YES** | `grep -B 1 "^fn eval_kernel_thread_join_result" src/runtime.rs` → `#[restricted_to(":wat::kernel::Thread/join-result", ":wat::")]` on the line above the fn signature. |
| B | `#[restricted_to(":wat::kernel::Process/join-result", ":wat::")]` applied to `eval_kernel_process_join_result` | **YES** | `grep -B 1 "^fn eval_kernel_process_join_result" src/runtime.rs` → `#[restricted_to(":wat::kernel::Process/join-result", ":wat::")]` on the line above the fn signature. |
| C | New verification test passes — both names land in `defined_value_restrictions` after startup | **YES** | `cargo test --release -p wat --test wat_arc198_slice2_stone_3_apply` → `test result: ok. 2 passed; 0 failed`. Both `thread_join_result_restriction_lands_in_symbol_table` and `process_join_result_restriction_lands_in_symbol_table` assert the `Some(&vec![":wat::".to_string()])` entry shape and both pass. |
| D | All predecessor tests still pass (17/17 across Stones A + B + arc 198 slice 1 + Stone 1 + Stone 2) | **YES** | `wat_arc170_stone_a_drain_and_join` 4/4; `wat_arc170_stone_b_walker_collapse` 4/4; `wat_arc198_def_restricted` 5/5; `wat_arc198_slice2_stone_1_inventory_wiring` 1/1; `wat_arc198_slice2_stone_2_attribute` 3/3. Total 17/17. |
| E | Workspace test failure count ≤ baseline (3 stable + lifeline flake variance) | **YES** | `cargo test --release --workspace --no-fail-fast` → `error: 4 targets failed`. Per-target failures: `lifeline_pipe_zero_orphans_across_100_trials` (known flake, hit this run), `deftest_wat_tests_tmp_totally_bogus` (pre-existing), `t6_spawn_process_factory_with_capture_round_trips` (pre-existing), `startup_error_bubbles_up_as_exit_3` (pre-existing). 3 stable + 1 flake = baseline exactly. **Zero new failures.** New Stone 3 test target adds +2 passes. |

**5/5 PASS.**

## Honest deltas

### Import path used — `use wat_macros::restricted_to;`

Added a single `use wat_macros::restricted_to;` line at the top of `src/runtime.rs` alongside the existing `use crate::*;` / `use std::*;` block (line 50). The `wat-macros` crate is already a non-dev dependency of `wat` (Cargo.toml line 58, no change needed). Stone 2's proc-macro export `pub fn restricted_to(attr, item) -> TokenStream` (`crates/wat-macros/src/lib.rs:246`) resolved cleanly with the brought-into-scope name — no `quote!`-style absolute-path qualification needed at the consumer site (the macro itself emits absolute `::wat::restriction_entry::RestrictionEntry` / `::inventory::submit!` paths in its expansion, so no `use` graph collision risk).

No re-export through `wat::*` was attempted; Stone 2's settled pattern is consumers import the attribute directly from `wat-macros`, mirroring how the `tests/wat_dispatch_*` test files import `wat_macros::wat_dispatch`.

### Both walkers fire during transition state — observed YES

Stone B's tests (`wat_arc170_stone_b_walker_collapse`, 4/4) still green after Stone 3's attribute application. That's the predicted transition-state behavior: BOTH walkers now fire on user-namespace calls to `*_join-result`. Arc 198's `walk_for_def_restricted_call` produces a `DefRestrictedCallerNotAllowed` diagnostic (the new mechanism); Stone B's `validate_join_result_user_namespace` produces the original "drain-and-join" diagnostic (the legacy ad-hoc rule). Stone B's test assertions grep for the legacy verb-name + "drain-and-join" substrings → present in Stone B's still-firing rule's output → tests still pass.

Stone 4 deletes Stone B's redundant rule; until then, the additive coexistence is the correct shape per the BRIEF's transition-state note.

### Substrate fn visibility — `fn` (private), not `pub(crate) fn`

The BRIEF predicted `pub(crate)` visibility on both `eval_kernel_*_join_result` fns. The actual visibility is bare `fn` (private to `src/runtime.rs`). Stone 2's macro is pass-through (`#[item_ts]` preserves the original tokens verbatim) so the visibility delta doesn't affect codegen — both private and `pub(crate)` items accept the attribute identically. One small substrate-fact correction; no design impact; no scope expansion.

The `inventory::submit!` block lands at module scope adjacent to the annotated fn (sibling, not inside), so it doesn't need access to the fn's visibility — it just needs to be at module scope to satisfy `inventory::submit!`'s requirement (per `crates/wat-macros/src/lib.rs:247-` codegen).

### Line numbers vs BRIEF claim

BRIEF pointed to `src/runtime.rs:16722` (Thread) + `src/runtime.rs:16340` (Process). Actual at session start: `17041` (Thread) + `16531` (Process). Decay drift across 1 commit window; symbol-grep located both unambiguously. No design impact; verified locations via `grep -n "fn eval_kernel_thread_join_result\|fn eval_kernel_process_join_result"` before editing.

### Test-first discipline

Per `feedback_test_first`: wrote `tests/wat_arc198_slice2_stone_3_apply.rs` BEFORE any substrate change. Ran the test:

```
test thread_join_result_restriction_lands_in_symbol_table ... FAILED
test process_join_result_restriction_lands_in_symbol_table ... FAILED
Map has 0 entries; ... key missing.
```

Saw it fail with the expected "key missing" message. Then added `use wat_macros::restricted_to;` + both attributes, rebuilt, watched both tests turn green. Then ran 5 predecessor targets in sequence to confirm no regression. Then full workspace run for baseline match.

### Workspace test count vs baseline

| Target | Baseline (Stone 2 end) | Post-Stone-3 | Delta |
|---|---|---|---|
| `wat::wat_arc198_slice2_stone_3_apply` (NEW) | (did not exist) | **2 passed / 0 failed** | +2 passes |
| `wat::probe_lifeline_pipe_proof` | 1 fail (flake 1-2/100) | 1 fail this run (flake hit) | within flake band |
| `wat::test` (deftest_wat_tests_tmp_totally_bogus) | 176 pass / 1 fail | 176 pass / 1 fail | unchanged |
| `wat::wat_arc170_program_contracts` (t6) | 23 pass / 1 fail | 23 pass / 1 fail | unchanged |
| `wat-cli::wat_cli` (startup_error_bubbles_up_as_exit_3) | 14 pass / 1 fail | 14 pass / 1 fail | unchanged |
| `wat::wat_arc170_stone_a_drain_and_join` | 4 passed | 4 passed | unchanged |
| `wat::wat_arc170_stone_b_walker_collapse` | 4 passed | 4 passed | unchanged |
| `wat::wat_arc198_def_restricted` | 5 passed | 5 passed | unchanged |
| `wat::wat_arc198_slice2_stone_1_inventory_wiring` | 1 passed | 1 passed | unchanged |
| `wat::wat_arc198_slice2_stone_2_attribute` | 3 passed | 3 passed | unchanged |
| Every other target | passes | passes | unchanged |

**Net: +2 new passes; 0 new failures; lifeline flake in rotation band.** Workspace failures = 3 stable + 1 flake = baseline exactly.

### Substrate-discovery surprises

**Two minor, both non-blocking:**

1. **Line-number drift** (BRIEF said 16722/16340; actual 17041/16531). Cost: one extra `grep -n` to locate. No design impact.
2. **Visibility delta** (BRIEF said `pub(crate)`; actual bare `fn`). Cost: zero — Stone 2's pass-through macro absorbs both shapes identically. No design impact.

Predicted 0-1 in EXPECTATIONS; actual: 2 (both fact corrections in the orchestrator-supplied substrate state pointers, neither requiring substrate redesign). Both surfaced and reconciled via grep-first verification per `feedback_assertion_demands_evidence`. Neither hit a STOP trigger.

Everything else composed cleanly:
- `use wat_macros::restricted_to;` resolved without re-export gymnastics.
- Stone 2's macro emitted the `inventory::submit!` blocks at module scope adjacent to each annotated fn without touching the fn bodies (verified Row C — both wat names appeared in `defined_value_restrictions` post-startup).
- Both Stone B (legacy walker) and arc 198 slice 1 (`walk_for_def_restricted_call`) walkers now coexist on the same diagnostic; Stone B's substring assertions and arc 198 slice 1's diagnostic-shape assertions both still pass.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 30 min | ~12 min |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ baseline (3 stable + flake variance) | 3 stable + 1 flake (baseline exactly) |
| New test count | 1 | 2 (split Thread + Process for clarity; same proof, two assertions) |
| Import path used | `use wat_macros::restricted_to;` OR absolute | `use wat_macros::restricted_to;` (top of runtime.rs, alongside existing use block) |
| Both walkers fire | YES (expected) | YES (Stone B's 4/4 + arc 198 slice 1's 5/5 both still green; transition-state additive coexistence as predicted) |
| Substrate-discovery surprises | 0-1 | 2 (line-number drift + visibility delta; both grep-resolved, no design impact) |
| Mode | Additive (2 attribute applications + 1 test) | Additive (2 attribute applications + 2 tests = one assertion per fn) |

## STOP triggers encountered

**None reached.**

- "Import path doesn't resolve cleanly" — `use wat_macros::restricted_to;` resolved on first build; wat-macros already a non-dev dep.
- "Compile error on attribute application" — clean build; Stone 2's pass-through codegen absorbed both `fn` (private) signatures.
- "Stone B's tests fail (both walkers should fire; both messages should combine)" — Stone B's 4/4 stayed green; legacy ad-hoc rule continues to produce "drain-and-join" substring while arc 198's generic walker now also fires.
- "Predecessor test regresses" — 17/17 predecessors green.
- ">2 unexpected substrate-finding surfaces" — 2 minor surfaces (line drift + visibility delta), both grep-resolved with zero design impact.

## What this enables

After Stone 3 ships:

- **Stone 4** deletes Stone B's ad-hoc `validate_join_result_user_namespace` walker rule + the orphaned `JoinResultUserNamespace` `CheckError` variant + Stone B's 4 caller-migration tests. The substrate's restriction-declaration channel is now fully uniform: both wat-source `def-restricted` and Rust-substrate `#[restricted_to(...)]` declarations land in the same `defined_value_restrictions` map via the same walker. The `:wat::kernel::Thread/join-result` and `:wat::kernel::Process/join-result` substrate fns are policed by the generic mechanism — no hard-coded substrate-namespace exemption needed.

Stone 4 also verifies that the generic walker provides observably equivalent coverage to the legacy ad-hoc rule before the legacy rule retires (caller-migration tests get reframed against the new diagnostic shape).

## Files touched

- `src/runtime.rs` — added `use wat_macros::restricted_to;` at the top (one line); annotated `eval_kernel_thread_join_result` and `eval_kernel_process_join_result` with `#[restricted_to(...)]` (two attribute lines, one above each fn signature). Fn bodies unchanged.
- `tests/wat_arc198_slice2_stone_3_apply.rs` — NEW. Two tests, one per fn, asserting `frozen.symbols.defined_value_restrictions.get(<wat name>)` returns `Some(&vec![":wat::".to_string()])` after `startup_from_source` against a minimal valid wat source.
- `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-3-APPLY-TO-JOIN-RESULT.md` — this file (NEW).
