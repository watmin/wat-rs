# SCORE — AMEND 251.8c: the loose-assert gate

Folded into the stone. **No follow-up commit.** Stone is now `bc93125aa` (was `4e9fb9e17`).
AMEND brief remains `c5e267958` on top. **Not pushed.** Floor/clippy: orchestrator's row.

## The red

`no_loose_string_assert::tests_carry_no_loose_string_assert` at the five `contains` sites
in `must_reject_string_arg` and the `nope` row. Captured at the floor; not re-run as the
disposition. Gate is main-only.

## The assertion shape

Not a whole-error `assert_eq!` (spans differ). Not a rune. Not a `.edn` golden.

For each shape, colon and slash are **paired**:
1. Extract the `TypeMismatch` whose `callee` field is exactly `":user::f"` (the kwarg row also
   emits an `assertion-failed!` mismatch; that is not the stone).
2. Colon row must equal `(":user::f", "#1", ":wat::core::i64", ":wat::core::String")` — the
   **control**. Pairing alone would pass if both broke the same wrong way.
3. Slash row must `assert_eq!` those four fields against the colon row of the same shape.

`nope` matches `StartupError::Resolve(UnresolvedReferences)` and `assert_eq!(path, ":user::nope")`.

## Walls I ran (not the floor)

- `stone_251_8c_slash_and_colon_heads_share_the_call_path` — 1 passed.
- `no_loose_string_assert::tests_carry_no_loose_string_assert` — 1 passed, 5940 skipped.

Floor + clippy + census: **not run** (brief: orchestrator, uncontended). Do not push. 8d does not start.
