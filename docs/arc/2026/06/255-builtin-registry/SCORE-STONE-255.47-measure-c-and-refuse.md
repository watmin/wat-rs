# SCORE — STONE 255.47: where C and Refuse land

Measurement only. The checker was instrumented in a worktree
(`/tmp/255.47-scratch/wt`, removed). The corpus run is `wat --check` of
all 2285 tracked `.wat` files, 16 at a time, with the instrumented
release binary. Each hit is one source span. Stdlib hits are counted
once. Nothing in `src/` or `wat/` on main changed.

## Hole B

The arm is `assignable`'s structural surface check
(`src/check.rs`, the block whose comment is "structural surface
satisfaction (row-polymorphic width subtyping)"). When the actual type
is an aggregate and the expected type is a surface,
`struct_satisfies_surface` asks that every member be present
(`src/types/surface.rs` 58, `members.iter().all`). An empty member list
is vacuously true. The nature floor then asks only that the aggregate's
rank meet the surface's nature (`src/check.rs`, `nature_ok`). No
`extend-type` edge is consulted on this path.

An edge takes an earlier arm and never reaches this one: path-to-path
`is_subtype` returns first (`src/check.rs`, the `ap != ep && is_subtype`
arm). The instrument logged a hit only when the member list and the
type-parameter list were both empty. It tagged the hit `HOLEB-edge` when
`is_subtype` was also true, and `HOLEB-noedge` otherwise.

Driven on `wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat`
(`wat --check`, rc 0). The instrument printed one line:

```
MEASURE25547 HOLEB-noedge wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat:34:72 head=:hello::takes-orderable :hello::Opaque -> :hello::Orderable
```

`:hello::Point` has an edge and produced no line. It is admitted by
`is_subtype`, not by this arm. The vector-of-Opaque call is admitted by
the generic edge, which is the parametric arm, also earlier.

The module doc states the design (`src/types/surface.rs` 1–6):

> A surface declares a structural interface: a set of required named
> members with types. Structs satisfy a surface by having (at least)
> those members with assignable types (width subtyping). No
> `:satisfies`, no `:parent`, no declaration at the use site.

The empty case is that sentence with nothing required. `Reason` says it
on purpose (`wat/query.wat` 73–76):

> `Reason` has zero features: any pure record satisfies it ambiently
> (an OPEN Record surface) — no `extend-type`/`derive` needed.

So the rule is deliberate width subtyping, and `Reason` is the documented
marker that uses the empty case. A featureless `:nature :Struct` surface
such as the hello-world `Orderable` falls through the same arm, and a
record outranks `Struct`, so the edge on `Point` is decorative. The
comments do not say that a marker with no members was meant to be a
closed class.

Featureless non-parametric surfaces in `wat/`, `wat-tests/`,
`wat-scripts/`, and `tests/` (a `defsurface` whose name line has no
`:-` and whose `:features` is `[]`):

| surface | where |
|---|---|
| `:wat::query::Reason` | `wat/query.wat:76` |
| `:hello::Orderable` | the 255.46 hello world |
| `:env::Portable` | `tests/types/probe_arc293_holder_root_symbol.wat:9` |
| `:probe::Reason` | four arc-170 probes and `tests/rete/probe_arc278_open_surface_dispatch.wat:29` |

The corpus produced 68 `HOLEB-noedge` hits and 0 `HOLEB-edge` hits.
Every one of the 68 is admitted only by the vacuous arm. None of them
has an edge. Three of the 68 have no source span (a macro-expanded
call). The pairs:

| actual | expected | sites |
|---|---|---|
| `:wat::query::Fault` | `:wat::query::Reason` | 45 |
| `:probe::SqliteReason` | `:probe::Reason` | 10 |
| `:probe::RedisReason` | `:probe::Reason` | 9 |
| `:probe::MongoReason` | `:probe::Reason` | 1 |
| `:probe::Note` | `:wat::query::Reason` | 1 |
| `:hello::Opaque` | `:hello::Orderable` | 1 |
| `:env::Rec` | `:env::Portable` | 1 |

`Fault` is the open-`Reason` case the comment describes. `Opaque` is the
same arm used as a closed class.

## Ordering and equality as declarations

C's shape is a leaf `extend-type` or a conditional edge. A flat edge
cannot carry "when".

### `is_type_orderable`

| arm | declaration |
|---|---|
| `i64`, `u8`, `f64`, `String`, `bool`, `keyword`, `Instant`, `Duration` | a leaf `extend-type` of that path to `Orderable` |
| `:wat::holon::Vector` | the same, one leaf. The runtime compares its `i8` data (`src/runtime.rs` 6074–6079). The checker and the runtime agree on this leaf |
| `(Vector :- [T])`, `(Option :- [T])` | one conditional edge, `(T <- Orderable)`. An empty argument list is true today (`is_none_or`). A missing `T` has no declaration |
| `(Result :- [T E])` | one conditional edge with two bounds. That is C's binder as a list. Declarable |
| tuple, non-empty, every element | not one edge. Arity is open. The declaration is "a tuple is orderable when each element is", which is not a binder on a head |
| empty tuple | the arm is false (`src/check.rs` 13683–13684). The runtime tuple arm compares lengths and would answer equal for two empty tuples (`src/runtime.rs` 6049–6056) if one arrived. The checker refuses. Not a declaration; a disagreement |
| `Fn` | no edge. The arm is false. The runtime has no arm and returns `None`. They agree |
| any other path or parametric head | no edge |
| variant widened to its enum | not an edge on the variant. The checker rewrites the variant to the enum and asks again (`src/check.rs` 13636–13648). A declaration would have to say "a variant is orderable when its enum is" |
| `Var(_) => true` | not a declaration. It is Refuse, below |

`bigint` and `rational` are not leaves of this predicate. Same-type
`(< 1N 2N)` is refused. Measured with the uninstrumented `wat --check`
of `wat-scripts/scratch-pad/255-47-order-bigint.wat`, rc 1:

```
parameter #1 expects an orderable type (...); got :wat::core::bigint
```

The runtime compares bigint to bigint, bigint to `i64`, and rational to
both (`src/runtime.rs` 5995–6020). A class copied from the checker still
refuses a value the runtime can order. Adding the two leaves is a
change, not a transcription.

`both_numeric` (`src/check.rs` 13738–13744) fires when unify fails and
both paths are in `is_numeric_check_path`: `i64`, `f64`, `bigint`,
`rational` (`13310–13315`). `u8` is orderable and is not in that list.
A second clause `([a <- A] [b <- B] :- [(A <- Numeric) (B <- Numeric)])`
says this, if `Numeric` is those four leaves. Measured: `(< 1 2.0)`
checks, rc 0, on `255-47-order-cross.wat`. The runtime has the same
cross pairs (`5975–5988`). This clause is expressible. It is not the
`(T, T)` clause.

### `is_type_equatable`

| arm | declaration |
|---|---|
| the scalar paths, `Uuid`, `char`, `Instant`, `Duration`, `holon/Vector`, `HolonAST`, `WatAST` | a leaf edge each. `values_equal` has a same-type arm for each scalar (`src/runtime.rs` 5700–5758) |
| aggregate, enum | a leaf is wrong: every aggregate and every enum is equatable, by `TypeDef` kind (`13558–13560`). One edge per type is not the arm. The arm is "any aggregate". The runtime compares aggregates and enums structurally (`5879`, `5844–5858`) |
| newtype, alias | recurse. A declaration would follow the wrapper, which an edge does not do by itself |
| union | every member. Not one edge |
| `Vector`, `List`, `Option` | conditional, one bound, same hole for a missing argument |
| `Result` | two bounds |
| tuple | every element, including the empty tuple (vacuous true). Not one edge. `values_equal` agrees for tuples (`5815`) |
| `HashMap`, `HashSet`, `PersistentVector` | blanket true, arguments not asked (`13585`). That is an unconditional generic edge, which is today's hole, not C's conditional edge. C's shape would refuse a map whose key is not equatable. The runtime answers `Some(a == b)` for two hash maps (`5902`) and does not walk `values_equal` on the entries. A conditional class and the runtime would disagree |
| `:wat::core::Value`, a param letter, `TypeExpr::Var` | defer, true. `is_type_param_letter` returns true for a `Var` (`src/check.rs` 10555–10557), so the `Var` arm lower down is unreachable. The corpus confirmed that: 0 `REFUSE-equatable-var` hits, and the letter arm logged both `:T` and `_` |
| a surface path | false. No edge. Agreed with "membership is an edge", and contradicted by Hole B for an aggregate |
| `Fn` | false. `values_equal` has no function arm and returns `None`. They agree |

Equality's compatibility rule is not a bound (`src/check.rs` 13386–13396):
unify, or either path is a subtype of the other, or both are records, or
both are numeric. Two independent bounds say "both are records" and
"both are numeric". They do not say "A is a subtype of B". That relates
the two variables. `assignable` is not a binder. Measured: two distinct
records type-check under `=` (`255-47-eq-records.wat`, rc 0). The runtime
compares them by class and fields and answers false when the class
differs (`5879`). The check admits it; the runtime returns false rather
than raising. A clause with two `Record` bounds admits the same pair.

## Refuse

Instrumented:

- `is_type_orderable`'s `Var(_) => true`
- `is_type_equatable`'s letter defer, which also catches `Var`
- the unreachable `Var` arm under it (0 hits)
- defclause `None => true` (0 hits in 2285 files)
- a `TypeExpr::Var` accepted by the first defclause clause (p11 at the
  moment of the match)

`None => true` did not fire. No checked call had an argument whose
inference returned `None` and then took a clause.

| reason | hits | what they are |
|---|---|---|
| `defn` parameter `:T`, equality | 3 | `wat/test.wat:62` `assert-eq`, `wat/seq.wat:560` `dedupe-walk`, and `probe-eq-generic-instantiation.wat:32`, which is the test of this defer |
| unification variable, equality | 5 | `wat/doctest.wat:118` (two `eval-ast!` results), `wat/rete/syntax.wat:50` (two `map/get` results), and three scratch examples. The value's type was never pinned |
| free names in a one-line comparison test, ordering | 3 | `tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat` and the `<=` and `>` siblings. The whole file is `(wat.core/< a b)`. `a` and `b` have no type |
| `Result`'s other parameter, ordering | 4 | `tests/types/ord_result_ok_le_same.wat:6`, `ord_result_err_ge_smaller.wat:6`, `ord_result_recursion_deep.wat:6`, `ord_result_recursion_shallow.wat:6`. The call is `(<= (Result.Ok …) (Result.Ok …))` with concrete `i64` payloads. `Ok` does not pin the error parameter. The walk sees that `Var` and returns true |

The four `Result` hits are the one the ruling bites. Refuse, applied
inside the recursive walk, rejects ordering two `Ok` values of `i64`,
which the tests exist to allow. The variable is unresolved because the
other variant was never constructed, not because the author forgot a
bound.

The 15 first-clause hits are `+` (3: `wat/rete/acc.wat:45` and two
destructure probes) and `rete/insert` (7) and `run!` (5). At the match,
one argument was a `Var`. `unify` then binds that variable to the
clause's parameter (`src/check.rs` 16714–16719), so it is not still a
`Var` at the end of the call. Refuse as worded, "still unresolved at the
end", does not see these. They are the p11 event: the first clause
decided the type. Whether that counts as resolving the variable is the
choice below.

## What the rulings did not size

1. Hole B is the documented width rule, and it is also how a featureless
   marker stops being a closed class. `Reason` depends on it. A closed
   `Orderable` cannot be a featureless surface and an `extend-type`
   class at the same time, under this arm.
2. Ordering two `Result.Ok` values depends on the unresolved other
   parameter being accepted. Refuse inside the walk rejects those four
   tests.
3. A `Var` accepted by the first defclause clause is bound by that
   clause before the call ends. "Unresolved at the end" and "the first
   clause won" are not the same set. The corpus has 15 of the second
   and, for `None => true`, 0 of a missing type.
4. `bigint` and `rational` are ordered at runtime and refused by the
   checker's class. A transcription of the checker keeps the refusal.
5. Equality of two different records is admitted and evaluates to false.
   A `Record` bound admits the same pair. Subtype-compatibility is not
   two bounds.
6. The `HashMap` / `HashSet` / `PersistentVector` blanket is an
   unconditional edge. C's conditional shape would be stricter than both
   the checker and `Value`'s `PartialEq`.
