# BRIEF — finish `:-`: one operator, four positions, one door

`:-` is wat's parameterization operator and it has four positions. Two work everywhere, one works
only for builtin collections, and one works nowhere — not because the language rejects them, but
because **nine hand-rolled recognisers of `:-` grew up with no door between them**, and positions 3
and 4 simply aren't among the four that peel. You will build the door, route all nine through it,
teach positions 3 and 4, and raise a rune so a tenth cannot appear.

Read `DESIGN-STONE-finish-the-param-spec.md` first. Copy the report shape of
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md`, and the shape of the rune and its positive control
from `tests/lint/one_name_grammar.rs`, which shipped this session.

## The door

Beside `is_binder_marker` in `src/types.rs` — that function is the intended door and answers only
half the question (*is this node the marker*), which is why four separate peels exist.

```rust
/// `[:- [T U …] rest…]` → `(Some(&[T,U,…]), rest)`;  no marker → `(None, args)`.
pub(crate) fn peel_param_spec(args: &[WatAST]) -> (Option<&[WatAST]>, &[WatAST])
```

Return the raw nodes, **not** parsed `TypeExpr`s: `check.rs` and `runtime.rs` treat them differently
downstream, and a door that pre-commits to one grows a second door for the other.

Pin these in unit tests beside it — they are where the nine currently differ:
**no marker at all**; **`:- []`** (must be `Some(&[])`, NOT `None` — the empty binder is *expressed*
and that is the builder's rule); **`:-` with a non-Vector following**; **`:-` as the LAST element**
with nothing after it.

## The nine

```
── peel the (marker, [types], rest) TRIPLE ────────────────────────────────
src/types.rs:5037     parse_type_form
src/check.rs:12015    unwrap_type_param_bracket        two spellings of ONE peel,
src/check.rs:12083    split_type_param_bracket         same file, different names
src/runtime.rs:4086   resolve_type_slot_args

── test only the MARKER, each its own spelling ────────────────────────────
src/function/metadata.rs:49    src/function/parse.rs:165    src/argspec/parse.rs:170
src/types.rs:4656  (is_binder_marker — keep it, express it via the door)
src/types.rs:5037  (inline again, in the door's own file)
```

## Positions 3 and 4 — the ones that do not work

Measured on the current build, and these are your acceptance rows:

```
A  (:wat::core::Vector :- [:i64] 1 2 3)    →  [1 2 3]                              ✅ WORKS TODAY
B  (:u::Box           :- [:i64] :v 5)      →  ":i64 is a TYPE keyword, not a value" ⛔ user record ctor
C  (:u::pick          :- [:i64] 7)         →  ArityMismatch: expected 1, got 3      ⛔ user fn call
```

**A is your exemplar — it already does this correctly.** B reads the type vector as a field value; C
counts `:-` and `[:i64]` as two extra positional arguments. Both must peel the param-spec first, then
bind the callee's declared type params from it, exactly as A's path does.

⚠ **Follow A's implementation rather than inventing one.** Find how the builtin collection
constructor reaches `resolve_type_slot_args` / `split_type_param_bracket` and take the same route.
Two implementations of one behaviour is the defect this stone exists to remove — do not add a fifth.

## The rune — `one_param_spec`

Refuse a hand-rolled `k == ":-"` test, and the `[WatAST::Keyword(..), WatAST::Vector(..), rest @ ..]`
binder slice-pattern, anywhere outside `src/types.rs`. Copy `tests/lint/one_name_grammar.rs` — it
carries the detector unit tests and the allowlist-with-reason shape.

**Positive-control it before you report it green:** plant a hand-rolled peel in a file that should be
refused, confirm the rune fails and names the site, then remove the plant. A rune that has never
failed has not been shown to work.

## STOP triggers

- **STOP-1 — two of the nine disagree** on an edge case and no single signature satisfies both. That
  is a real finding about the operator, not a refactor detail: report both sites and what each
  expects. Do NOT add a second door.
- **STOP-2 — position 3 or 4 cannot peel without changing what a currently-green program means.**
  Report the program and both meanings; ship nothing on that position.
- **STOP-3 — the rune cannot be drawn** without either missing real sites or refusing honest ones.
  Ship the door and the conversions without it rather than shipping a rune that lies.

## Boundaries

- `src/types.rs` (the door), the nine listed sites, the two value positions, one new rune.
- **Do NOT touch `wat/service.wat` or `wat/core.wat`.** The mints emitting `:- [args]` is the NEXT
  stone and it depends on position 4 working first.
- **Do NOT apply the minting wall.** It is parked as a patch and its cascade is already measured.
- **Do NOT delete the angle machinery** (`split_type_params`, `canonical_callable_name`,
  `check.rs:5159`'s arm). Sibling stone, needs a green floor first.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- The orchestrator runs the full floor and clippy centrally. Use `./target/release/wat --check <file>`
  (~0.2s) and scoped `cargo nextest run --release -E '...'`.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

The door's signature and the four edge cases with what each returns — especially `:- []`, which must
be `Some(&[])` and not `None`. The nine conversions. Rows A/B/C verbatim in one run, all three
together. Whether the rune drew clean, and the positive control you ran on it. Any STOP that fired,
with the arm captured verbatim BEFORE you diagnosed it. And any two of the nine that disagreed —
that disagreement is the finding this stone exists to surface.
