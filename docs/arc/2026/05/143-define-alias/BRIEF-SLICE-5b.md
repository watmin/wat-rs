# Arc 143 Slice 5b — Sonnet Brief — value_to_watast handles HolonAST

**Drafted 2026-05-02 (late evening).** Substrate-informed: orchestrator
crawled `src/runtime.rs:5878-5895` (value_to_watast body) +
`src/runtime.rs:8326+` (eval_holon_to_watast — the existing
HolonAST→WatAST converter via `holon_to_watast(&h)`). The fix is
a 1-line addition; sonnet's value is adding the test for the new
path + verifying it lets slice 6 ship.

**Goal:** extend `value_to_watast` to handle
`Value::holon__HolonAST` by calling the existing `holon_to_watast`
converter. This unblocks slice 6's `define-alias` macro splicing.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/SCORE-SLICE-6.md`** — Mode B
   diagnosis identifying Gap 1 (`value_to_watast` rejecting
   `Value::holon__HolonAST`).
2. **`src/runtime.rs:5878-5895`** — `value_to_watast` body. Note the
   `_ => Err(RuntimeError::TypeMismatch { expected: "primitive ... or
   :wat::WatAST", ... })` catch-all arm.
3. **`src/runtime.rs:8326+`** — `eval_holon_to_watast`. Calls
   `holon_to_watast(&h)` to convert; this is the converter we'll
   reuse.
4. **`tests/wat_arc143_define_alias.rs`** (slice 6 output) — the 2
   failing tests (`define_alias_foldl_to_user_fold_delegates_correctly`
   + `define_alias_length_to_user_size_delegates_correctly`). After
   this slice, the foldl test should pass; the length test will still
   fail with Gap 2 (separate concern; slice 5c).

## What to ship

### Substrate change — extend `value_to_watast`

Add ONE arm to the `match v` in `value_to_watast`
(`src/runtime.rs:5882`):

```rust
Value::holon__HolonAST(h) => Ok(holon_to_watast(&h)),
```

`holon_to_watast` is the helper in scope (used by
`eval_holon_to_watast` at line 8326+). Verify its signature is
`fn holon_to_watast(h: &HolonAST) -> WatAST` (sonnet should grep to
confirm before adding).

That's it. The catch-all arm continues to error for other Value
variants we haven't bridged yet.

### Test (mandatory)

Add ONE test to `src/runtime.rs::tests` (or wherever
value_to_watast is currently tested — verify by grepping
`value_to_watast` in test code) that:

1. Constructs a `Value::holon__HolonAST` directly (e.g., via
   `HolonAST::symbol(":foo")`).
2. Calls `value_to_watast("test_op", value, Span::unknown())`.
3. Asserts the result is `Ok(WatAST::Keyword(":foo", _))` (or
   whatever the `holon_to_watast` of a Symbol leaf produces — verify
   by reading `holon_to_watast`).

OR: re-run slice 6's existing failing test (`cargo test --release
--test wat_arc143_define_alias define_alias_foldl_to_user_fold_delegates_correctly`)
and verify it now passes. That's the load-bearing end-to-end
verification.

## Constraints

- **ONE Rust file modified:** `src/runtime.rs`. ~5 LOC total
  (1-line fix + 4-line test, give or take).
- **No wat files. No new test crate.** New test goes in
  `src/runtime.rs::tests` (existing module).
- **Workspace stays GREEN at the substrate level:** `cargo test
  --release --workspace` exit non-zero ONLY because of:
  - 1 pre-existing arc 130 LRU failure
  - Slice 6's `define_alias_length_to_*` test still fails (Gap 2;
    that's slice 5c's territory)
  - The other slice 6 test (`define_alias_foldl_to_*`) should now
    PASS — that's the verification of this slice's fix.
- **No commits, no pushes.**

## What success looks like

1. `value_to_watast` has the new HolonAST arm.
2. The new unit test for value_to_watast(HolonAST) passes.
3. `cargo test --release --test wat_arc143_define_alias
   define_alias_foldl_to_user_fold_delegates_correctly` PASSES (the
   foldl alias works end-to-end).
4. `cargo test --release --workspace` shows: 1 pre-existing LRU
   failure + 1 remaining slice 6 length test failure (Gap 2). NO
   other regressions.

## Reporting back

Target ~150 words (slice is small):

1. **The `value_to_watast` change** — quote the new arm verbatim.
2. **The new unit test** — quote it verbatim.
3. **Verification of slice 6's foldl test** — confirm it now passes
   (quote the passing test name from cargo test output).
4. **Test totals** — `cargo test --release --workspace` confirming:
   - 1 pre-existing LRU failure unchanged
   - Slice 6's length test still fails (Gap 2)
   - All other tests pass; ZERO new regressions
5. **Honest deltas** — anything you needed to investigate/adapt.

## Sequencing

1. Read SCORE-SLICE-6.md, value_to_watast at 5878+,
   eval_holon_to_watast at 8326+ (to confirm holon_to_watast's
   signature).
2. Add the arm to value_to_watast.
3. Add a unit test for the new path.
4. Run `cargo test --release --test wat_arc143_define_alias
   define_alias_foldl_to_user_fold_delegates_correctly` — verify it
   transitions from FAILED to PASSED.
5. Run `cargo test --release --workspace` — confirm overall state.
6. Report.

Then DO NOT commit. Working tree stays modified for orchestrator
to score.

## Why this slice matters

Slice 5b is the GATING fix for slice 6's macro working at all. Tiny
fix; massive unblock. After 5b ships, slice 6's foldl path works
end-to-end — the whole substrate-as-teacher cascade demonstrably
held. Slice 5c (length scheme registration) is the second gap;
slice 7 then ships the alias application.
