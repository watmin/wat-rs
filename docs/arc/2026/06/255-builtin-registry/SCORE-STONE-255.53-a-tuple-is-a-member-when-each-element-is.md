# SCORE — STONE 255.53: a tuple is a member when each element is

Struck against draw `f1726a0ad` (brief drawn at `1d191ce40`, redrawn onto `:..`).
`(:wat::core::extend-type :- [[Ts :< :u::Mark] :..] (:wat::core::Tuple :- [Ts :..]) :u::Mark)`
means a tuple is a Mark when each element is a Mark. `:..` is a keyword. It is
not a name, and it is not a rest slot on `TypeExpr`.

## The marker

`parse_binder_entries` accepts `:..` only as the keyword after the last binder
entry, once. Nothing before it, a later entry, or a second `:..` is a
`MalformedDecl` on `extend-type` that quotes the binder. `fn`, `defn`, and a
surface method refuse a well-placed `:..` with
`` `:..` is only legal on an extend-type binder ``. The variadic parameter
`(defn f :- [Ts :..] …)` stays cut.

The only child that may contain `:..` is `(Tuple :- [Name :..])`, and `Name`
must be that repeated binder entry. Any other `:..` in the child or the target
is refused and the form is named. `TypeExpr::Tuple` is unchanged. STOP-2 did
not fire.

## The edge

`GenericEdge.tuple_each` is `Some("Ts")`. The edge is keyed by
`:wat::core::Tuple`, so a `TypeExpr::Tuple` of any arity finds it. The binding
of `Ts` is the tuple. `bound_failure` asks `assignable(slot, bound)` for each
slot, the same path as 255.52. A nested tuple re-enters the edge. A vector of
a tuple uses the 255.52 edge and then this one.

A miss is `MembershipBound` with `slot` set to the 1-based position. A
parametric miss has `slot: None`. The two 255.52 rows still name `T`,
`:u::Mark`, and `:u::Out`, and their slot is absent.

`family_extends` skips an edge with `tuple_each` set, and a bounded or
repeated edge is not `register_subtype`. A concrete tuple is not admitted by
an existence walk.

## The empty tuple

Not chosen. The slot loop is the ordinary "every element" loop, and an empty
iterator succeeds. Measured with `./target/debug/wat --check`, not pinned:

| spelling | result |
|---|---|
| `(:wat::core::Tuple :- [])` as the argument of `take` | checks, rc 0. Forced against `i64`, the got is `:()`. That is `TypeExpr::Tuple(vec![])`. The uniform rule admits it. |
| `(:wat::core::Tuple)` | not that type. `MalformedForm` head `:wat::core::Tuple`, reason `tuple must have at least one element`, and a `TypeMismatch` got `:(_,)`. |
| `()` | `BareLegacyUnitValue`. `()` is not a tuple value. |

## The rows

`cargo test --test types probe_arc255_53` — 10 passed, rc 0.

| row | result |
|---|---|
| pair `(In, In)` | i64 1 |
| triple `(In, In, In)` | i64 1 |
| `(In, (In, In))` | i64 1 |
| vector of `(In, In)` | i64 1 |
| `(In, Out)` | `MembershipBound` argument `:(u::In,u::Out)` surface `:u::Mark` param `Ts` bound `:u::Mark` got `:u::Out` slot `2` |
| `(In, (In, Out))` | same kind, argument `:(u::In,(u::In,u::Out))`, slot `2`, got `:u::Out` |
| `defn` binder `[Ts :..]` | Runtime `MalformedForm` head `:wat::core::defn`, reason names `[Ts :..]` |
| binder `[[Ts :< :u::Mark] :.. U]` | Type `MalformedDecl` head `extend-type`, reason names that binder |
| binder `[:.. Ts]` | Type `MalformedDecl` head `extend-type`, reason names `[:.. Ts]` |
| child `(Tuple :- [U :..])` under repeated `Ts` | Type `MalformedDecl` head `extend-type`, reason names `U` and `Ts` |

## Proof

Floor `.floor/2026-09-27T02-39-46Z`: 6173 passed, 22 skipped, rc 0. That is
6163 at `bed5a7bcd` plus these 10 rows. The six `ord_tuple_*` tests are in
that pass. `<` still uses `is_type_orderable`.

Clippy `--release --all-targets -- -D warnings`: rc 0.

Pre-census `.census/2026-09-27T02-34-25Z.txt` (2289 files, the unmodified
draw). Post `.census/2026-09-27T02-46-50Z.txt`: `census-diff: no STOP-8`.
0 rc flips. 215 nonzero of 2290. One new file, rc 0:
`tests/types/probe_arc255_53_tuple_member.wat`.

Delta `.delta/2026-09-27T02-47-48Z`: NEW 2 / RECOVERY 0. The two files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.
