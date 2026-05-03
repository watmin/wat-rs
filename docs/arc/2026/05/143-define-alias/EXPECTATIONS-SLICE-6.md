# Arc 143 Slice 6 — Pre-handoff expectations

**Drafted 2026-05-02 (evening)** for the userland define-alias
macro + apply slice. The end-to-end test of arc 143's
substrate-as-teacher cascade.

**Brief:** `BRIEF-SLICE-6.md`
**Output:** 1-2 new wat files (`wat/std/ast.wat` + possibly a
test file) + ~250-word written report.

## Setup — workspace state pre-spawn

- Slice 1 shipped: 3 substrate primitives (lookup-define,
  signature-of, body-of) live in src/runtime.rs +
  src/check.rs. 11 tests in tests/wat_arc143_lookup.rs all
  pass.
- Concern 1 from SCORE-SLICE-1.md flagged: synthesised AST
  renders bare type names (`:Vec<T>` not `:wat::core::Vec<T>`).
  Slice 6 TESTS whether this breaks define-alias's emitted
  define.
- Workspace state: 1 pre-existing failure
  (`deftest_wat_lru_test_lru_raw_send_no_recv` from arc 130
  RELAND v1's stepping stone hitting "unknown function:
  :wat::core::reduce"). This failure is what slice 6's
  apply step (Piece 4) should turn green. EVERYTHING else
  passes.
- Slice 6 sonnet has no conversation memory of slice 1's
  sweep. Walks in cold; reads SCORE-SLICE-1.md +
  tests/wat_arc143_lookup.rs to understand what slice 1
  shipped.

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Wat file diff | Exactly 1 wat source file created (`wat/std/ast.wat`) + at most 1 wat test file created. No Rust files modified. No substrate changes. |
| 2 | Piece 1 present (rename-callable-name) | Function `:wat::core::rename-callable-name` defined in `wat/std/ast.wat`. Signature matches BRIEF: takes `(head :HolonAST) (from :keyword) (to :keyword) -> :HolonAST`. |
| 3 | Piece 2 present (extract-arg-names) | Function `:wat::core::extract-arg-names` defined. Signature: `(head :HolonAST) -> :Vector<keyword>`. |
| 4 | Piece 3 present (define-alias defmacro) | Defmacro `:wat::core::define-alias` registered. Signature: `(alias-name :AST<keyword>) (target-name :AST<keyword>) -> :AST<unit>`. Body uses `signature-of`, `Option/expect`, `rename-callable-name`, `extract-arg-names`, quasiquote. |
| 5 | Piece 4 present (apply line) | A line `(:wat::core::define-alias :wat::core::reduce :wat::core::foldl)` present in `wat/std/ast.wat` (or wherever the macro is defined). |
| 6 | Piece-by-piece workflow | Sonnet ran `cargo test --release --workspace` AFTER each piece (per BRIEF Step 7-13). Verifiable in the report's piece-by-piece roll-up — sonnet should describe at least 2-3 intermediate cargo test runs. |
| 7 | **`cargo test --release --workspace`** (after Piece 4) | Two acceptable outcomes: <br>**Mode A**: exit=0; the formerly-failing arc 130 stepping stone (`deftest_wat_lru_test_lru_raw_send_no_recv`) now reports `... ok` (FQDN concern was unfounded; reduce alias resolves). <br>**Mode B**: clean diagnostic — sonnet stopped at first red, surfaced WHICH piece broke + the failure mode. Pieces 1-3 (helpers + macro) MUST work; only Piece 4 (the apply) is allowed to fail in Mode B if the FQDN concern materialises. |
| 8 | Per-piece test coverage | Pieces 1 + 2 each have 2-3 unit tests (helper inputs/outputs verified). Piece 3 has 1-2 expansion tests (the macro produces the expected AST shape). Piece 4's verification IS the formerly-failing arc 130 test transitioning. |
| 9 | **No substrate edits** | Sonnet did NOT modify `src/runtime.rs` / `src/check.rs` / any other Rust file. Verifiable via `git diff --stat` showing only wat files. |
| 10 | Honest report | 250-word report includes: piece-by-piece pass/fail roll-up, cargo test totals, the macro expansion verbatim, four-questions verdict on the wat code, honest deltas, file LOC. |

**Hard verdict:** all 10 must hold. Row 7 is the load-bearing
end-to-end test; either Mode A or a clean Mode B diagnostic
counts. Row 9 is load-bearing for the discipline (substrate
fixes belong in slice 5, not here).

## Soft scorecard (5 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC budget | wat/std/ast.wat: ~50-100 LOC (helpers + macro + apply). Test file: ~30-80 LOC. Total slice diff: 80-200 LOC. |
| 12 | Complectens discipline | Each test in the test file is a named-helper-style deftest per /complectens. Body 3-7 lines. No monolithic deftests. |
| 13 | Wat-side stdlib placement | Macro + helpers in `wat/std/ast.wat` (new file) per DESIGN's slice 6 plan. Tests in `wat-tests/std/ast.wat` (or analogous) per the project's test-mirror convention. |
| 14 | Quasiquote style match | The macro's quasiquote body matches the style of `:wat::test::make-deftest` at `wat/test.wat:387-403`. Comma-splice for parameters; comma-at-splice for vector args. |
| 15 | String primitive surfacing | If wat lacks string primitives needed for `rename-callable-name` (Piece 1), sonnet surfaces it as a Mode B blocker BEFORE attempting Piece 2. The brief explicitly authorises this stop. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~40%) — Mode A clean ship:** the FQDN concern
  was unfounded. The TypeScheme registry's bare names work
  because the type checker resolves them via the same registry
  at re-parse time. Pieces 1-4 ship; arc 130 stepping stone
  turns green. ~15-25 min runtime.

- **Mode B-FQDN (~30%):** Pieces 1-3 ship; Piece 4 fails on
  type-check of the synthesised define head (`:Vec<T>` not
  recognised as FQDN). Clean diagnostic; opens slice 5a
  immediately to fix FQDN rendering. After 5a, Piece 4 relands
  trivially.

- **Mode B-string-primitives (~15%):** Piece 1 fails because
  wat lacks the string primitives needed for the rename
  (split-on-`<`, substring, concat). Surfaces a substrate gap.
  Either slice 5 absorbs string primitive additions, or a
  separate arc opens.

- **Mode B-quasiquote-arity (~10%):** Pieces 1+2 ship; Piece 3
  fails because the quasiquote/unquote/splicing semantics for
  the generated define have a subtle mismatch sonnet can't
  resolve from the brief. Reland with sharper macro example.

- **Mode B-other (~5%):** something the orchestrator didn't
  anticipate. Honest delta surfaces; reland or open follow-on.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` (expect 1-2 wat files; NO Rust
   files).
4. Verify hard rows 2-5 by reading wat/std/ast.wat and
   confirming each piece's signature.
5. Verify hard row 7 by running `cargo test --release
   --workspace` locally; confirm the arc 130 stepping stone
   transition (or the Mode B failure mode).
6. Verify hard row 8 by counting tests in the test file.
7. Verify hard row 9 by `git diff --stat src/`.
8. Score; commit `SCORE-SLICE-6.md` as a sibling.

## Why this slice matters for the chain

This is the END-TO-END validation of arc 143's
substrate-as-teacher cascade. Slice 1 shipped the
introspection primitives in Rust. Slice 6 builds the userland
consumer in pure wat. If the cascade holds, every future
reflection-driven macro inherits the foundation; the substrate
gains a permanent capability for ~395 LOC of Rust + ~80 LOC
of wat across slices 1+6.

If Mode B surfaces, the diagnostic IS the data. Slice 5a (or
whatever slice owns the surfaced gap) closes it; slice 6
relands trivially; arc 130 unblocks; arc 109 v1 closes.

The user's framing 2026-05-02 evening: "prove sonnet can
demonstrate the service work from an empty slate." Slice 6 IS
that demonstration for the userland-macro layer. Slice 1
proved the substrate layer.

## What we learn

- **Mode A (all 4 pieces pass):** the cascade holds end-to-end.
  Substrate primitives compose cleanly into userland macros.
  Slices 2/3/4/5/7 ship in any order; the arc converges.
- **Mode B at Piece 1 (string primitives):** wat needs string
  manipulation primitives for substrate-name string surgery.
  Open follow-on arc; slice 6 relands after.
- **Mode B at Piece 4 (FQDN):** slice 1 needs to render FQDN
  type names. Slice 5a takes over; slice 6 relands trivially.
- **Mode B at Piece 3 (quasiquote arity):** the macro's
  quasiquote shape needs sharpening. Reland with worked
  example.
- **Soft drift on complectens (row 12):** sonnet writes
  monolithic test bodies. Reland with sharper /complectens
  citation.
