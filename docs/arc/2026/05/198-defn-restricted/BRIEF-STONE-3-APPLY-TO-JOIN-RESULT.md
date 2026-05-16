# Arc 198 Slice 2 Stone 3 BRIEF — apply `#[restricted_to(...)]` to `eval_kernel_*_join_result`

**Arc:** 198 slice 2, Stone 3 of 4.
**Task:** #328
**Predecessors:**
- Stone 1 (commit `51c69a1`) — inventory wiring shipped
- Stone 2 (commit `6775510`) — proc-macro attribute shipped

**Successor:** Stone 4 — delete arc 170 Stone B's ad-hoc walker rule + update Stone B's tests

## Goal

Apply the `#[restricted_to(...)]` proc-macro attribute (Stone 2) to the two substrate fns that arc 170 Stone B currently protects via the ad-hoc walker rule:

- `eval_kernel_thread_join_result` (`src/runtime.rs`)
- `eval_kernel_process_join_result` (`src/runtime.rs`)

This stone is **application-only**: no proc-macro design (Stone 2 done); no Stone B rule deletion (Stone 4).

**Transition state during this stone:** BOTH walkers (Stone B's ad-hoc `validate_join_result_user_namespace` AND arc 198's `walk_for_def_restricted_call`) will fire on user-namespace calls to `*_join-result`. That's expected — Stone B's tests should still pass (their assertions grep for verb-name + "drain-and-join" — Stone B's walker produces both substrings). Stone 4 cleans up the redundancy.

## Form (per Stone 2's settled attribute syntax)

```rust
#[restricted_to(":wat::kernel::Thread/join-result", ":wat::")]
pub(crate) fn eval_kernel_thread_join_result(...) -> Result<Value, RuntimeError> {
    // body unchanged
}

#[restricted_to(":wat::kernel::Process/join-result", ":wat::")]
pub(crate) fn eval_kernel_process_join_result(...) -> Result<Value, RuntimeError> {
    // body unchanged
}
```

First arg = wat name (the symbol that gets restricted in `defined_value_restrictions`); second arg = `:wat::` namespace prefix (any caller in `:wat::*` namespace is allowed).

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — exact import path for the attribute (`wat_macros::restricted_to` vs other), Cargo.toml dependency wiring between wat crate and wat-macros, whether `pub(crate)` visibility interacts oddly with the attribute. Do NOT trust orchestrator claims without grep verification.

## Substrate state pointers (verified)

- `src/runtime.rs:16722` — `eval_kernel_thread_join_result` (annotate this)
- `src/runtime.rs:16340` — `eval_kernel_process_join_result` (annotate this)
- `crates/wat-macros/src/lib.rs` — Stone 2's `#[restricted_to(...)]` attribute definition
- `Cargo.toml` (wat crate root) — likely already depends on wat-macros for `#[wat_dispatch]`; verify
- `src/restriction_entry.rs` (Stone 1) — `RestrictionEntry` struct that the attribute references via `::wat::restriction_entry::RestrictionEntry`
- `src/freeze.rs` step 6.8 — Stone 1's iteration; drains `inventory::iter::<RestrictionEntry>` into `symbols.defined_value_restrictions`
- `tests/wat_arc170_stone_b_walker_collapse.rs` — Stone B's 4 tests (MUST stay green in this stone; assertions grep both verb-name + "drain-and-join")
- `src/check.rs:3094` (Stone B) — `validate_join_result_user_namespace` (still fires; deletion is Stone 4)
- `src/check.rs:3094+` (arc 198 slice 1) — `walk_for_def_restricted_call` (will ALSO fire now)

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read substrate state.** Pointers above. Verify wat crate's import path to `wat_macros::restricted_to` works (or whatever path Stone 2 actually exports).

2. **Write 1 verification test FIRST** in `tests/wat_arc198_slice2_stone_3_apply.rs`:
   - Use `startup_from_source` with minimal valid wat
   - Assert `frozen.symbols.defined_value_restrictions.get(":wat::kernel::Thread/join-result")` returns `Some(vec![":wat::".to_string()])`
   - Assert same for `:wat::kernel::Process/join-result`
   - RUN; CONFIRM test fails (attributes not yet applied)

3. **Apply attribute** to `eval_kernel_thread_join_result` and `eval_kernel_process_join_result` in `src/runtime.rs`. Add necessary `use wat_macros::restricted_to;` or absolute path.

4. **Build clean.** `cargo build --release --workspace --tests`.

5. **Run new test.** Green.

6. **Regression check (CRITICAL — both walkers should fire):**
   - `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse` → 4/4 still green (Stone B's tests unchanged in this stone)
   - `cargo test --release -p wat --test wat_arc198_def_restricted` → 5/5 still green
   - `cargo test --release -p wat --test wat_arc198_slice2_stone_1_inventory_wiring` → 1/1 still green
   - `cargo test --release -p wat --test wat_arc198_slice2_stone_2_attribute` → 3/3 still green
   - `cargo test --release -p wat --test wat_arc170_stone_a_drain_and_join` → 4/4 still green

7. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (3 stable + lifeline flake variance).

8. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd; verify with `pwd` periodically.
- DO NOT modify Stone 2's `#[restricted_to(...)]` proc-macro attribute.
- DO NOT modify Stone 1's `RestrictionEntry` struct or `src/freeze.rs` iteration.
- DO NOT modify `eval_kernel_*_join_result` fn BODIES — only ADD the attribute above the fn definition.
- DO NOT delete Stone B's ad-hoc walker rule (`validate_join_result_user_namespace`) — that's Stone 4.
- DO NOT update Stone B's 4 tests — they MUST stay green in this stone (both walkers fire; both messages combine in the diagnostic; Stone B's assertions pass).
- DO NOT touch arc 198 slice 1's wat-side `def-restricted` / `defn-restricted` forms.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs/EXPECTATIONS/SCOREs / this BRIEF / this EXPECTATIONS / superseded slice-2 monolithic BRIEF + EXPECTATIONS.
- DO NOT update USER-GUIDE / docs.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.

## Scorecard (5 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `#[restricted_to(":wat::kernel::Thread/join-result", ":wat::")]` applied to `eval_kernel_thread_join_result` | `grep -B 3 "fn eval_kernel_thread_join_result" src/runtime.rs` shows the attribute |
| B | `#[restricted_to(":wat::kernel::Process/join-result", ":wat::")]` applied to `eval_kernel_process_join_result` | same for Process |
| C | New verification test passes — both names land in `defined_value_restrictions` after startup | `cargo test --release -p wat --test wat_arc198_slice2_stone_3_apply` → green |
| D | All predecessor tests still pass (17/17 across Stones A + B + arc 198 slice 1 + Stone 1 + Stone 2) | targeted runs all green |
| E | Workspace test failure count ≤ baseline (3 stable + flake variance) | full workspace cargo test failures ≤ baseline + variance |

## STOP triggers

- Attribute import path doesn't resolve cleanly (e.g., wat-macros not in wat crate's deps) → STOP and surface
- Attribute application causes a compile error on the annotated fn → STOP and surface
- Stone B's tests fail after attribute application → STOP and investigate (both walkers SHOULD fire; combined output SHOULD contain both wordings)
- Any predecessor test regresses → STOP and investigate
- > 2 unexpected substrate-finding surfaces → STOP

## Workspace baseline (commit `6775510`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures (t6, totally_bogus, startup_error) + lifeline flake (rotation band)

Post-Stone-3 target:
- ≥ baseline + 1 pass (new application-verification test)
- ≤ baseline failures (purely additive — attribute application doesn't change behavior)

## Time-box

30 min predicted. Hard stop 60 min. If approaching stop, write partial SCORE.

## On completion

Write `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-3-APPLY-TO-JOIN-RESULT.md`:
- 5 rows YES/NO with grep-able evidence
- Honest deltas: attribute import path used, any visibility/scoping surprises, both-walker-firing behavior observed
- Calibration record

Return final summary: rows passed/failed + import path + both-walkers-fire? + workspace delta + path to SCORE.

You are launching now. T-minus 0.
