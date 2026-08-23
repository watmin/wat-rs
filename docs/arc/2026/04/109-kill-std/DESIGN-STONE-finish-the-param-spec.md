# DESIGN — finish `:-`: one operator, four positions, one door

> *"the angle bracket expr for types is illegal - there must only be a single way to declare
> param-spec (its already `:- []` anyways)"*
> *"if they are not expressed, they are `:- []` / if they are expressed, and are empty, they are
> `:- []` / otherwise they are whatever binders the user chose"*
> *"we've got things to go fix - now - we fix them"* — the builder, 2026-08-23

## The rule, settled and measured

```clojure
absent                  ≡  :- []
:- []                   ≡  :- []
:- [A B C]              →  those binders
```

**Measured, not assumed** — `defn` and `defrecord` both: absent, `:- []`, and `:- [T]` all behave
identically where they should. Absent and `:- []` already normalise to the same empty vec, so a macro
may emit `:- []` unconditionally at zero cost and with zero branches. That is the point: **the human
surface allows omission; the machine surface is always explicit.** No mono-vs-parametric branch.

## The four positions, and where they actually stand

```
1. declaration binder   (defn :f :- [T] …)           ✅  fn defn defenum newtype typealias
                                                          typeunion defstruct defrecord
                                                          defsurface-method
2. type reference       (Vector :- [i64])            ✅
3. constructor          (Vector :- [i64] 1 2 3)      ✅ BUILTIN collections
                        (Box    :- [i64] :v 5)       ⛔ user records — "i64 is a TYPE keyword,
                                                          not a value" (the vector read as an ARG)
4. call application     (:f :- [i64] 7)              ⛔ ArityMismatch: expected 1, got 3
                                                          (`:-` and `[i64]` counted as ARGS)
```

⚠ **The form is not missing. Its consumers are.** An earlier reading of `ArityMismatch` as "the
language cannot express this" was wrong, and it is the recurring error this arc keeps paying for:
*an error names where the INSTRUMENT gave up, never what the system lacks*
(`[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]`).

## Why positions 3 and 4 fail — nine recognisers, no door

```
FOUR hand-rolled peels of the (marker, [types], rest) triple:
  src/types.rs:5037     parse_type_form
  src/check.rs:12015    unwrap_type_param_bracket   ← two spellings of one peel,
  src/check.rs:12083    split_type_param_bracket    ← same file, different names
  src/runtime.rs:4086   resolve_type_slot_args

FIVE separate spellings of "is this node the marker":
  src/function/metadata.rs:49 · src/function/parse.rs:165 · src/argspec/parse.rs:170
  src/types.rs:4656  (is_binder_marker — the intended door)
  src/types.rs:5037  (inline AGAIN, inside the door's own file)
```

`is_binder_marker` answers only *"is this `:-`"*. It never peels the triple, so every consumer that
needs the type list hand-rolls the slice pattern — and positions 3 and 4 fail simply because they are
**not among the four**. Nothing rejects them on purpose; nobody taught them.

This is precisely the shape `STONE-one-name-grammar` just closed for names, one level up: a grammar
with many implementations and no door. It is the ninth and tenth instance of that class in this arc.

## What ships

**1. `peel_param_spec` — the one door**, beside `is_binder_marker` in `src/types.rs`:

```rust
/// `[:- [T U …] rest…]` → `(Some(&[T,U,…]), rest)`;  no marker → `(None, args)`.
pub(crate) fn peel_param_spec(args: &[WatAST]) -> (Option<&[WatAST]>, &[WatAST])
```

Returning the raw nodes, not parsed `TypeExpr`s — `check.rs` and `runtime.rs` need different
downstream treatments and a door that pre-commits to one would grow a second door for the other.

**2. The nine recognisers become nine calls.**

**3. Positions 3 and 4 learn the form** — user aggregate constructors peel the param-spec before
reading fields; the call path peels it before counting arity. Both then bind the callee's declared
type params from it, mirroring what the builtin collection ctor already does.

**4. A rune** — `one_param_spec` — refusing a hand-rolled `k == ":-"` or a
`[Keyword, Vector, rest @ ..]` binder pattern outside `types.rs`. Same shape and same home as
`one_name_grammar`, which shipped this session and is positive-controlled.

## What this does NOT do

Affirmatively cut, not deferred:

- **`defservice` emitting `:- [args]`, the minting wall, and `symbol-node`'s wall.** Those are the
  next stone and they DEPEND on this one: position 4 must work before a macro can emit it. The wall
  is parked as a patch, measured, with its cascade already known (3034/4893).
- **The purge of the 48 angle parsers.** Dead only once nothing mints; needs a green floor to say so.
- **`defclause`'s shared return** (the seventh slot). Same "this consumer never learned `:-`" shape,
  but it is a declaration position and this stone is scoped to the two VALUE positions. It joins the
  door in the same sweep only if it falls out for free.

## The four questions

- **Obvious?** YES. `:-` means one thing; a reader meeting it in any of four positions gets the same
  answer. Today two positions silently read it as data.
- **Simple?** YES. One door, one signature. The consumers shrink; nothing new is invented — position 4
  is the same form the builtin constructor has always accepted.
- **Honest?** YES, and this is the failing axis today: the language documents `(head :- [T] args)` as
  canonical while two of its three value-position consumers read `[T]` as a value. A form that means
  one thing here and another thing there is the lie this stone removes.
- **Good UX?** YES. `(:f :- [i64] 7)` is what a reader would try after seeing
  `(:wat::core::Vector :- [:i64] 1 2 3)`, and today it answers with an arity error that names nothing
  real.
