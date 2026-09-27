# SCORE — STONE 255.52: conditional membership on `extend-type`

Struck against draw `72ae5db65` (brief drawn at `b5c816de4`). Ruling C.
`(:wat::core::extend-type :- [[T :< :u::Mark]] (:wat::core::Vector :- [T]) :u::Mark)`
means a vector of T is a Mark when T is a Mark. The 255.51 refusal of that binder is lifted.

## The edge

`GenericEdge` carries `bounds: Vec<Option<TypeExpr>>` parallel to `params`
(`src/types.rs`). `None` is an unbounded parameter. `extend_type_operands`
returns the `BinderParam`s, bound included. `EdgeParamAbsentFromChild` still
requires every name in the child. `EdgeFreeTypeName` walks the child, the
target, and each bound. A free name in a bound is slot `"bound"`.

A bounded edge is stored only as that struct. It is not also
`register_subtype` of the rendered child, and it is not a
`register_parametric_extension`. An unbounded edge, including `Spawned` and
`Seqable`, still writes the string edge it wrote before.

## Where a bound is decided

`conditional_edge` (`src/check.rs`) asks `generic_edge_matches`, then
`assignable(binding, bound)` for each `Some` bound, on a cloned substitution.
`(Vector :- [(Vector :- [:u::In])])` is a Mark because the same edge matches
twice and the inner binding is `:u::In`, which has a declared edge.
`assignable` stays in `check.rs`. `types.rs` does not call it. STOP-2 did not
fire.

A miss is `CheckErrorKind::MembershipBound`: the argument type, the surface,
the letter, the bound, and what the letter was bound to. The nested refusal
keeps the inner got `:u::Out` and reports the outer vector as the argument.
A bounded `defn` still fails as 255.51's `BoundNotSatisfied`; the membership
miss is taken so it does not retitle that error.

## Paths that ignore arguments

Each one, and why a bounded edge cannot pass through it:

| path | what it does now |
|---|---|
| `family_extends` (`src/types.rs`) | pushes a generic edge's target only when every bound is `None`. `satisfies_spawned` and `subsume.rs` call this. A bounded edge is invisible to them. |
| `is_subtype` of the rendered actual, and of the head (`src/check.rs`, the parametric-actual/path-expected arm) | a bounded edge never calls `register_subtype`, so neither string is a parent. |
| `generic_edge_targets` | deleted. Both former callers use `conditional_edge` or `admitted_edge_targets`, which drop an edge whose bound fails. |
| `parametric_extensions_of` | not written for a bounded edge. |

Unbounded edges still take the old four-way OR. STOP-1 did not fire: nothing
that was admitted by an unbounded edge changed. The floor's `Spawned`,
`select`, and `Seqable` rows passed.

Runtime `family_extends` will not treat a conditional member as the surface.
Mark has no methods, so nothing dispatches on it. A later method on a
conditional surface has to ask the checker, which is where `assignable` lives.

The comment that said `enforce_type_bounds` instantiates a scheme now sits on
`instantiate`.

## The rows

`cargo test --test types probe_arc255_5` — 18 passed, rc 0, before the floor.

| call | result |
|---|---|
| `take` of `(Vector :- [:u::In])` | i64 1 |
| `take` of `(Vector :- [(Vector :- [:u::In])])` | i64 1 |
| `take` of `(Vector :- [:u::Out])` | `MembershipBound` argument `(:wat::core::Vector :- [:u::Out])` surface `:u::Mark` param `T` bound `:u::Mark` got `:u::Out` |
| `take` of the nested Out vector | `MembershipBound` argument `(:wat::core::Vector :- [(:wat::core::Vector :- [:u::Out])])` surface `:u::Mark` param `T` bound `:u::Mark` got `:u::Out` |
| `:u::f :- [[T :< :u::Mark]]` of the In vector / the Out vector | i64 1 / `BoundNotSatisfied` function `:u::f` param `T` bound `:u::Mark` got `(:wat::core::Vector :- [:u::Out])` |
| `take-any` of `(Vector :- [:u::Out])`, edge `:- [T]` onto `:u::Any` | i64 1 |
| `:- [[T :< :u::NoSuch]]` | `EdgeFreeTypeName` name `:u::NoSuch` slot `bound` |

The 255.51 fixture that pinned the refusal is now
`tests/types/probe_arc255_51_binder_extend_bounded.wat` and starts up clean.
That is the declaration this stone accepts.

## The red floor, not re-run

`.floor/2026-09-27T01-47-19Z`, exit 100:

```
Summary [ 356.131s] 6163 tests run: 6161 passed (11 slow), 2 failed, 22 skipped
```

`tests_carry_no_inlined_wat` panicked at `tests/lint/no_inlined_wat_in_tests.rs:440`.
The file was `tests/types/probe_arc255_52_conditional_membership.rs`: the
assertion strings are rendered types. The file now carries
`// rune:lint(no-inlined-wat)` saying so. The programs stay in the fixtures.

`every_wat_bad_fixture_actually_fails_shard_07` panicked at
`tests/lint/every_wat_bad_fixture_actually_fails.rs:392`.
`tests/types/probe_arc255_51_binder_extend_bounded.wat.bad` started up clean.
It was renamed to `.wat`.

## Proof

Floor `.floor/2026-09-27T01-54-57Z`: 6163 passed, 22 skipped, rc 0. That is
6155 at `161f6993f` plus these 8 rows.

Clippy `--release --all-targets -- -D warnings`: rc 0.

Pre-census `.census/2026-09-27T01-39-59Z.txt` (2287 files, the unmodified
draw). Post `.census/2026-09-27T02-02-17Z.txt`: `census-diff: no STOP-8`.
0 rc flips. 215 nonzero of 2289. Two new files, both rc 0:
`tests/types/probe_arc255_51_binder_extend_bounded.wat` and
`tests/types/probe_arc255_52_conditional_membership.wat`.

Delta `.delta/2026-09-27T02-03-16Z`: NEW 2 / RECOVERY 0. The two files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.
