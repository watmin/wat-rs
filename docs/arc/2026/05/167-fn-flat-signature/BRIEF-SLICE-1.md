# Arc 167 slice 1 — `WatAST::Vector` substrate foundation

## Goal

Mint `WatAST::Vector` as a first-class AST node distinct from
`WatAST::List`. Parser handles `[...]` brackets producing the new
variant. At eval/check time, Vector at value position errors with a
clear message; legal-position consumers (fn/defn signatures) come
in slice 2. After this slice ships, existing wat code is unchanged
(no test failures); the substrate has a new node kind ready for
slice 2's consumer.

## Branch + commit policy

- You are working on branch `arc-167-slice-1-vector-foundation`,
  branched from main.
- You may make multiple WIP commits on this branch as you progress,
  including pushes to origin for remote backup. Main stays at
  last-known-green; do NOT push to main.
- The orchestrator merges your branch into main as a single atomic
  commit after scoring.
- Suggested cadence: commit when each milestone is locally clean —
  e.g., after the parser change builds, after match-arms propagate
  cleanly, after tests pass. Smaller commits are easier to review.

## Background context (read these first)

- `docs/arc/2026/05/167-fn-flat-signature/DESIGN.md` — full arc scope
  (5 slices). This slice = slice 1, the substrate foundation.
- `docs/arc/2026/05/166-core-defn-form/INSCRIPTION.md` — most recent
  closed substrate arc; defn shipped, recursive name binding +
  reflection fixed. Arc 167 evolves defn's surface; slice 1 is the
  precondition.
- `docs/SUBSTRATE-AS-TEACHER.md` — the failure-engineering
  discipline. Slice 1 itself doesn't trigger a substrate-as-teacher
  cascade (it's additive substrate plumbing), but slice 2 will.

## Critical scope constraint (user direction 2026-05-08)

Vectors are **expressions in binding-syntax positions only**, not
value literals. Per user:

> "i'm not ready to support vec literals as values... just vecs as
> exprs... e.g. fn's args. `(fn [x <- i64] -> i64 (+ 0 x))` is what
> i want to support now. `(conj [0 1] y)` — not this... we don't
> know how to entertain this yet."

- Parser produces `WatAST::Vector` whenever it sees `[...]`
- `eval` on `Vector` errors clearly (value-position not yet supported)
- `check`/`infer` on `Vector` similarly errors at value position
- The fn-sig consumer (slice 2) is the only legal Vector-consumer
  in arc 167; let-binding (arc 168) follows
- Future arcs deliberately extend the legal positions

## Substrate edits

### 1. `src/ast.rs` — variant + accessors

Add `Vector(Vec<WatAST>, Span)` to the `WatAST` enum (after the
`List(...)` variant). Update:

- `span()` accessor — return `s` from the new variant
- `Display` impl (if exists) — render as `[child1 child2 ...]`
- `PartialEq` (if hand-written) — structural equality on inner Vec
- Any other trait impls on `WatAST` — pattern-match the new variant
  identically to how `List` is handled where appropriate

### 2. `src/parser.rs` — bracket parsing

Find the existing list-parsing logic that produces `WatAST::List`
(currently around lines 159, 184). Add a parallel path for `[...]`:

- When the lexer/wat-edn yields a `[` token, consume tokens until
  matching `]`, producing `WatAST::Vector(items, span)` with the
  span covering `[` to `]` inclusive
- Wat-edn's existing `Value::Vector` parsing should already handle
  the bracket tokenization — verify and consume the resulting
  Vector at the wat-language layer

If the existing parser path goes through wat-edn → conversion to
WatAST: extend the conversion to map `Value::Vector` →
`WatAST::Vector`. If it's a hand-rolled tokenizer, add a new arm.
Audit the parser FIRST and report which path is in use, then make
the matching addition.

### 3. `src/runtime.rs` — `eval` arm

In `pub fn eval(...)`, the match on `WatAST` needs an explicit
`WatAST::Vector(_, span)` arm that errors:

```rust
WatAST::Vector(_, span) => Err(RuntimeError::MalformedForm {
    head: "<vector literal>".into(),
    reason: "vector literals at value position are not supported \
             in arc 167. Vectors are currently consumed only in \
             :wat::core::fn / :wat::core::defn signature positions \
             (slice 2 wires those consumers). A future arc enables \
             vector literals as `Value::Vec` values.".into(),
    span: span.clone(),
}),
```

### 4. `src/check.rs` — `infer` arm

Parallel to eval — `infer` (the type-checker dispatch) gets a new
arm for `WatAST::Vector` at value position. Error with the same
message shape as eval, surfaced as a `CheckError::MalformedForm`
or equivalent.

The fn-sig parsing path (`parse_fn_signature_for_check`) should NOT
be changed in slice 1 — that's slice 2's territory. Slice 1's
infer/check arm fires whenever Vector appears in a context where
slice 2 hasn't yet wired a consumer.

### 5. Match-arm propagation across the workspace

`cargo build --release --workspace` will name every site that
exhaustively matches on `WatAST` and now needs an explicit
`WatAST::Vector(_, _) => ...` arm. For most sites, a fall-through
no-op or pass-through is correct:

- Walkers that recurse through children — recurse into vector items
  the same way they recurse into list items
- Span/identifier helpers — extract span identically
- Pattern-matching utilities — pass through unchanged if the
  utility isn't position-aware

For substrate-judgment cases where the right behavior isn't
obvious, default to mirroring the `List` arm and report the site
in your final notes for orchestrator review.

### 6. `wat-edn` round-trip — verify

`crates/wat-edn/src/value.rs:50` shows `Value::Vector(Vec<Value<'a>>)`
already exists at the EDN layer. If conversion sites between
`wat-edn::Value` and `WatAST` exist (in `src/edn_shim.rs` perhaps),
they need a `Value::Vector ↔ WatAST::Vector` arm. Audit and update.

## Tests

Create `tests/wat_arc167_vector_ast.rs` with:

1. **`vector_at_top_level_parses_as_vector`** — a wat program
   `[1 2 3]` parses cleanly without erroring at the parser layer
   (assert via debug-print of the parsed AST, or via a substrate-
   level entry point that returns the AST).

2. **`empty_vector_parses`** — `[]` parses as empty Vector.

3. **`nested_vector_in_list_parses`** — `(:foo [1 2 3])` parses
   with the inner `[1 2 3]` as a Vector child of the outer List.

4. **`vector_at_value_position_errors_clearly`** — a program with
   `[1 2 3]` in evaluation position fires the
   "vector-literals-not-supported" error. Assert the error message
   contains "vector literals at value position are not supported".

5. **`vector_at_value_position_in_define_body_errors`** —
   `(:wat::core::define (:user::main -> :wat::core::Vector<wat::core::i64>) [1 2 3])`
   errors at type-check or eval with the same message.

Tests use the standard `startup_ok` / `startup_err` helpers per
arc 153/154/155/166 precedent. Copy their definitions if needed.

## Verification (per scorecard in EXPECTATIONS-SLICE-1.md)

- `cargo build --release --workspace` green (all match-arms
  propagated)
- `cargo test --release --workspace --no-fail-fast`: 0 failed
  (existing tests unaffected; new arc 167 tests pass)
- `cargo clippy --release --workspace`: no new warnings
- The branch `arc-167-slice-1-vector-foundation` is up-to-date on
  remote; orchestrator can merge cleanly into main

## Discipline reminders

- DO NOT push to main; only push to your slice branch
- DO NOT modify any wat-source files (`wat/*.wat`,
  `wat-tests/*.wat`) — those changes are slice 2/3 territory
- DO NOT touch `parse_fn_signature` / `parse_fn_signature_for_check`
  — slice 2's territory
- DO NOT delete or modify the existing `WatAST::List` paths — slice
  1 is purely additive at the variant level
- If you hit a substrate decision that isn't covered by this brief
  (e.g., "should the EDN round-trip preserve vector semantics or
  treat it like a list?"), STOP and report; orchestrator decides

## Report shape

When complete, report:

1. Final cargo test summary (passed/failed counts)
2. Each substrate site you edited (file + line range) with a
   one-line description of the change
3. Match-arm propagation count: how many sites needed explicit
   Vector arms, and what the chosen behavior was for each
4. Test names + pass/fail status of each of the 5 cases
5. Honest deltas — any substrate decision you made beyond what the
   BRIEF specified, especially around EDN round-trip, walker
   recursion, or match-arm pass-through behavior
6. Branch state: confirm `arc-167-slice-1-vector-foundation` is
   up-to-date on remote with all work
7. Actual runtime in minutes vs predicted band in EXPECTATIONS

## Time-box

Predicted in EXPECTATIONS. If you exceed the upper bound still
iterating, STOP and report current state.
