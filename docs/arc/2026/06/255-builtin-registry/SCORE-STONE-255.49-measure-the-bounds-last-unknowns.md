# SCORE — STONE 255.49: measure the bound's last unknowns

Measurement only. Drawn against `518f85a88`. Measured on the draw
`78b8152a2`, in a worktree. Nothing landed on `main` but this file.

The instrument is `MEASURE25549` lines from one `wat --check` of each
tracked file: 2286 `.wat` and 376 `.wat.bad` (2662). The first file was
checked with `MEASURE_ALL`, so stdlib sites are in the log once; every
later file logged only spans in that file. Counts below are unique by
the log line. Flags on `=` were computed independently of the
short-circuit, then combined the way the arm combines them.

## 1. Tuples

A tuple is `TypeExpr::Tuple(Vec<TypeExpr>)` (`src/types.rs` 104–109).
It is not a `Parametric` head. The list spelling
`(:wat::core::Tuple :- [A B])` parses to that variant (`src/types.rs`
6344–6350). `format_type` prints `:(a,b)` (`src/check.rs` 18184–18192).
That keyword does not lex: a comma in a keyword body is refused. The
probe `tuple-keyword-child.wat` exited 1 with that lex error, and the
message names `(:wat::core::Tuple :- [T1 T2 T3])` as the spelling to use.

`extend-type` can take the list spelling as its child. The concrete
probe loaded the declaration and then refused both calls:

```
:probe::take: parameter #1 expects :probe::Orderable; got :(wat::core::i64,wat::core::i64)
:probe::take: parameter #1 expects :probe::Orderable; got :(wat::core::i64,wat::core::i64,wat::core::i64)
```

The edge is stored under the rendered tuple string
(`register_subtype` of `format_type`). `assignable` walks an
`extend-type` edge only when both sides are `Path` (`src/check.rs`
17335). A tuple never enters that arm, so the stored edge is not
consulted. A binder on a tuple does not become a generic edge:
`register_generic_edge` returns immediately for anything that is not a
`Path` or a `Parametric` (`src/types.rs` 1473–1479). The one-element
probe `(:wat::core::Tuple :- [T])` loaded and still refused
`(:wat::core::Tuple 1)` (`got :(wat::core::i64,)`).

Operator sites whose operand is a tuple, top level:

| op | arity | sites |
|---|---|---|
| `<` | 2 | `tests/types/ord_tuple_lt.wat:3`, `ord_tuple_recursion_shallow.wat:3`, `ord_tuple_recursion_deep.wat:3` (the element is itself a 2-tuple) |
| `>` | 2 | `ord_tuple_gt.wat:3` |
| `>=` | 2 | `ord_tuple_ge.wat:3` |
| `<=` | 3 | `ord_tuple_le_equal.wat:3` |

No `=` or `not=` site has a tuple operand. No empty tuple reaches any
of these operators. No arity other than 2 or 3 does, except the nested
pair inside the deep `<`.

Shapes, not a choice:

| shape | what was measured |
|---|---|
| one `extend-type` per arity, finite set | the corpus uses 2 and 3, and also a 2-tuple of a 2-tuple. An edge on one concrete 2-tuple of `i64` does not make that value satisfy the surface. The ordering walk (`src/check.rs` 13683–13685) does not read edges |
| one rule on the class, "a tuple when each element is" | that is the walk that exists. It is not a binder. `Tuple` has no single parameter for `T`, and the generic-edge store drops it. A finite list of clauses `(Tuple :- [A B])`, `(Tuple :- [A B C])` does not cover the nested site or an arity the checker already accepts and the corpus does not use |
| leave the walk beside the clauses | the operators keep admitting every non-empty tuple whose elements are orderable, which is what the six sites rely on. Nothing in the 2662 files asks an `extend-type` edge to do this |

## 2. Every aggregate is equatable

`family_extends` after one `register_subtype` of a nature root to
`:probe::Equatable`. The child edges are the ones registration already
writes (`src/types.rs` 1237–1240): a struct's parent is
`:wat::core::Struct`, a record's is `:wat::core::Record`, a holon
record's is `:wat::holon::Record`.

| | struct edge only | struct + record | all three roots |
|---|---|---|---|
| a struct | true | true | true |
| a record | false | true | true |
| a holon record | false | true | true |
| an enum name | false | false | false |
| a name with no edge | false | false | false |

The holon root is already a subtype of the record root. Measured:
`is_subtype(":wat::holon::Record", ":wat::core::Record")` is true with
no edge added by this probe. The seed is `src/types.rs` 3284–3288.
`is_subtype` of either record root to `:wat::core::Struct` is false.
There is no nature-ladder edge. Rank (`src/types.rs` 503–509) is a
different test and `family_extends` does not call it.

So one edge on `:wat::core::Record` covers records and holon records.
One edge on `:wat::core::Struct` covers structs only. One edge on
`:wat::holon::Record` covers holon records only.

The same answers through a surface, not through `family_extends`. Each
file is `extend-type` of one root to a featureless `:probe::Equatable`,
then one call. Struct edge: struct rc 0; record, holon, enum, and
newtype rc 1, each `TypeMismatch` expected `:probe::Equatable`. Record
edge: record rc 0, holon rc 0, struct rc 1. Holon edge: holon rc 0,
record rc 1.

Enums do not register a root. A variant registers
`variant <: enum` (`src/types.rs` 1686) and nothing above the enum.
`Purity` has no `root_keyword`. `is_type_equatable` answers true for
`TypeDef::Enum` by kind (`src/check.rs` 13560), which is not
`family_extends`.

Newtypes register no subtype edge (`parse_newtype` returns the
`TypeDef` only). `family_extends` is false. `is_type_equatable`
recurses into the inner type (`src/check.rs` 13561–13563). The struct
root edge does not make `:probe::Price` satisfy `Equatable`.

## 3. Subtype compatibility

`:- [A (B <- A)]` does not parse. `binder-var-target.wat` exited 1:

```
malformed extend-type declaration: an extend-type binder `:- [P …]` declares bare parameter names; got list
```

That is `extend_type_operands` (`src/types.rs` 5838–5848): a binder
entry that is not a bare symbol is an error. The form in the ruling is
not a form the checker accepts today.

`=` / `not=` reached the compatibility test 2016 times (1994 `=`,
22 `not=`). The flags were computed on the types before `unify`.
`is_subtype` is reflexive (`src/types.rs` 7062–7063), so a same-path
compare sets the subtype flag even though the arm never looks at it
when `unify` succeeds. Counted by which arm actually admits:

| admission | sites | what they are |
|---|---|---|
| `unify` of the same printed type | 1992 | 1230 non-numeric paths, 716 a numeric path with itself, 9 a record with itself (the record flag is also true, because a record is a subtype of `:wat::core::Record`), 37 other same printed types |
| `unify` of a var with a concrete type | 8 | the printed types differ only because one side is `_`. `tests/collection/list.wat:46`, two record round-trips, `uuid_edn_roundtrip_typed.wat:7`, `wat_arc220_char.wat:79`, `probe-surface-ships.wat:24`, and two scratch probes |
| subtype arm only | 2 | `277-the-node-kind-boundary.wat:73` and `:74`. `:user::NodeKind` against `:user::NodeKind.Map` and `:user::NodeKind.Set`. `unify` fails. Neither is a record |
| both-records only | 1 | `probe_arc237_sC3_macro_split.wat:41`. `:my::Pt` against `:my::HPt` |
| both-numeric only | 10 | every one is `i64` against `f64` (one of them flipped). `=` and `not=`. The grids under `tests/types/probe_arc237_8c` and `8d`, `probe_rational_C5_mixed_compare.wat`, `probe_rational_C5c_nan_unordered.wat`, `wat_not_eq.wat`, and `probe-300-C5c-nan-unordered-gate.wat` |
| none | 3 | `Stream` against `Vector` (`probe_arc247_hof_coll_first.wat.bad:5`), and `i64` against `String` in the two cross-type `.wat.bad` grids |

No site was admitted by subtype together with both-records or
both-numeric. The both-records clause would carry the one `Pt`/`HPt`
site. The both-numeric clause would carry the ten `i64`/`f64` sites.
The subtype arm is the two enum/variant sites, and it accepts either
direction (`src/check.rs` 13389–13390). `assignable` of two paths
accepts only `is_subtype(actual, expected)` (`src/check.rs` 17335).
Those are not the same predicate. A bound `B <- A`, once both names
were concrete, would be the one-direction check. It cannot say "both
are records" or "both are numeric".

## 4. p11

When any argument was still a `Var`, every arity-matching clause was
tried on its own substitution, and the real dispatch was left on the
first forward match. 16 sites. Every one has more than one forward
clause. None is unique.

| head | forward | clauses | sites | why the argument is a var |
|---|---|---|---|---|
| `:wat::core::+` | 5 | 25 | `wat/rete/acc.wat:45` (second arg), `tests/wat_lang/probe_arc234_stone4_hash_destructure.wat:41` (first arg) | one side is `i64` or `f64` from the accumulator or the destructure; the other is `_`. Five numeric clauses accept that pinned side paired with an unbound partner (`i64`, `bigint`, `rational`, `f64`, and a second copy of the pinned type) |
| `:wat::core::+` | 20 | 25 | `probe_arc234_stone4_match_hash_destructure.wat:22` | both arguments are `_`. Twenty of the twenty-five clauses accept two unbound numerics |
| `:wat::rete::insert` | 2 | 2 | 7 scratch and service probes, plus `probe_arc278_query_type_safe_typo.wat.bad:19` | one argument is `:wat::rete::Session` and the other is `_` (the `.wat.bad` has the var on the session side). Both clauses are `Session` plus an unbound fact. The `.wat.bad` is the site 255.47's `*.wat` census did not see |
| `:wat::core::run!` | 3 | 3 | five `wat-scripts/scratch-pad/census-*.wat` lines | the function is a concrete `String -> ()`; the collection is `_`. The three clauses are `Vector`, `List`, and `PersistentVector` |

The unique-solution rule, applied at this point, refuses all 16. Today's
dispatch takes the first forward match and does not.

## 5. Still unresolved at the end of the definition

The orderable `Var` arm and the equatable letter arm recorded the hit.
After the function's return was checked against its declaration, the
substitution was read again. All 15 hits from 255.47 are still
unresolved there.

| hit | at the end |
|---|---|
| `:wat::test::assert-eq`, `wat/test.wat:62` | rigid `:T`. It is the parameter of that definition. It is not a substitution variable |
| `:wat::core::dedupe-walk`, `wat/seq.wat:560` | rigid `:T`, same |
| `:user::eq-generic`, `probe-eq-generic-instantiation.wat:32` | rigid `:T`, same |
| `:wat::doctest::verify-examples` `wat/doctest.wat:118` | the var is unbound |
| `:wat::rete::query-read` `wat/rete/syntax.wat:50` | unbound |
| three scratch `=` probes (`255-home-10`, `255-home-12`, `255-struct-field-is-a-constant-projection`) | unbound |
| the three one-line `<` / `<=` / `>` files under `tests/resolve/` | there is no definition. The check of the top-level form ends with the var unbound |
| the four `ord_result_*.wat` `:user::compute` bodies | the var is unbound at the end of `compute`, not only at the `<=` call. `Ok` still has not pinned the error parameter when the definition is finished |

Refuse, worded as "still unresolved at the end of the enclosing
definition", sees the four `Result` tests. It also sees the three rigid
`:T` parameters, the five unbound equality vars, and the three forms
that are not definitions.

## What the rulings did not foresee

1. The binder the ruling writes, `:- [A (B <- A)]`, is a parse error.
   Entries are bare names.
2. An `extend-type` whose child is a tuple loads and does not make the
   tuple a member of the surface. Generic edges are not stored for
   tuples at all.
3. One record-root edge covers holon records, because
   `:wat::holon::Record` is already a subtype of `:wat::core::Record`.
   It does not cover structs. Enums and newtypes are equatable by a
   kind test and a recursive inner test, not by a root edge.
4. The subtype arm fired twice, both enum against its own variant. The
   both-records arm fired once. The both-numeric arm fired ten times,
   all `i64`/`f64`.
5. Every var-argument defclause site in the 2662 files is ambiguous
   under the unique-solution rule. There is no unique one.
6. The `Result` error variable is still unbound when `:user::compute`
   has finished checking.
