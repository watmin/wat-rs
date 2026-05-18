# Arc 212 stone δ-comm-positions — SCORE: sharpen validate_comm_positions

## Summary

The fourth permitted slot is now recognized. `validate_comm_positions` in
`src/check.rs` can now correctly classify comm calls in a `:wat::core::let`
binding-RHS as PERMITTED when the binding-name is later consumed (as a match
scrutinee or Result/Option expect-value) in the same let. The walker migrates
to `node.children()` for generic non-List recursion, with the let-form handler
doing its own scope-aware traversal.

## Implementation

### `CommCtx` — new variant

A `LetBindingRhs` variant was added to `CommCtx`. It is included in the
`permitted` check alongside `MatchScrutinee`, `ResultExpectValue`, and
`OptionExpectValue`. The new variant is assigned when a binding-name is in the
consumed set.

### `collect_consumed_names_in_let` — new helper fn

Pre-walks a flat binding Vector (`[n0 rhs0 n1 rhs1 ...]`) and body forms to
collect names that appear as:
- Match scrutinee: position 1 of `(:wat::core::match <name> ...)`
- Result/expect value: position 3 of `(:wat::core::Result/expect -> :T <name> "msg")`
  (both the canonical and the retired form recognized)
- Option/expect value: position 3 of `(:wat::core::Option/expect -> :T <name> "msg")`
  (same dual-form recognition)

Returns a `HashSet<String>`. The walk is recursive (handles nested lets,
match arms, etc.). Uses a local `fn walk()` to avoid a proliferation of
top-level helpers.

### `validate_comm_positions` — let-form handler

Added at the TOP of the function body (before the List-head dispatch):

1. Detects `(:wat::core::let [binding-vector] body...)`.
2. Calls `collect_consumed_names_in_let` with the binding Vector items and body
   forms.
3. Walks each binding-RHS (odd-index positions in the flat Vector) with
   `CommCtx::LetBindingRhs` if the binding-name is in the consumed set, or
   `CommCtx::Forbidden` otherwise.
4. Walks body forms with `CommCtx::Forbidden` (comm in the body must be
   consumed by an enclosing match/expect within the body itself).
5. Returns — does NOT fall through to generic recursion.

### Generic recursion

The old TEMPORARY `let WatAST::List(items, _) = node else { return; }` guard
is removed. Non-let, non-List nodes now recurse via `node.children()` at the
bottom of the function (covers `Vector` and `StructPattern`; is a no-op for
leaf atoms).

## Verification

| Named test | Result |
|---|---|
| `cargo test --release --test arc112_slice2b_process_send_recv` | PASS (1/1) |
| `cargo test --release --test arc112_scheme_probe` | PASS (1/1) |
| `cargo test --release --test wat_arc208_process_io_result` | PASS (7/7) |

## Build

`cargo build --release` — clean (pre-existing warnings only, no new warnings).

## Honest-delta note

None. No previously-silent comm patterns newly caught during this run. The
test fixture `arc112_slice2b_schemes_wire_through_typechecker` exercises the
exact `recv-result (:wat::kernel::recv rx)` / `(:wat::core::match recv-result
...)` binding pattern and passes cleanly with the sharpened rule.

## Mode classification

**Mode A** — sharpening implemented; all three named tests green; cargo build clean.
