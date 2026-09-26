# SCORE — STONE 255.46: what `defintrinsic` needs to type every function row

Measurement only. No `src/` or `wat/` edit. Struck against draw `7c53a18c8`.
The weigh of 255.45 left the function rows in class (iv), plus the 12
fresh-return rows. The form-shaped rows stay forms. They are named at the
end, not in the table.

## The syntax that exists

`extend-type` already takes a binder. `wat/spawn.wat` 261–263:

```
(:wat::core::extend-type :- [S R]
  (:wat::kernel::Thread :- [S R])
  (:wat::spawn::Spawned :- [S R]))
```

`extend_type_operands` peels that `:- [P …]` (`src/function/parse.rs` 999–1008).
The child and the target are two type forms. The binder names parameters
of those forms. It does not attach a bound to a variable inside a clause.

A `defn` binder is the same `:- [T …]` peel, and it keeps only symbols
(`src/function/metadata.rs` 48–59). A list such as `(T <- :wat::core::Orderable)`
is dropped. The names that survive become `Function.type_params`
(`src/function/eval.rs` 103) and then `TypeScheme.type_params`
(`src/check.rs` 18286). `TypeScheme` is `type_params: Vec<String>` and
nothing else (`src/check.rs` 81–89). The comment at `src/check.rs` 2496
says the same thing: no bounds field, so a row cannot say "E ranges over
enums".

The illustrative `defintrinsic` bound is not a form either parser accepts.

## Part A

### Ordering

`is_type_orderable` (`src/check.rs` 13634–13687), after widening a variant
to its enum:

| arm | answer |
|---|---|
| `TypeExpr::Var(_)` | true. Unresolved. Deferred to runtime |
| path `i64`, `u8`, `f64`, `String`, `bool`, `keyword`, `Instant`, `Duration`, `holon/Vector` | true |
| any other path | false |
| `(Vector :- [T])`, `(Option :- [T])` | the argument is orderable. An empty argument list is true |
| `(Result :- [T E])` | both arguments are orderable |
| any other parametric head | false. `HashMap`, `HashSet`, records, structs |
| tuple | true only when it is non-empty and every element is orderable |
| `Fn` | false |

`family_extends` answers existence of an `extend-type` edge and ignores
arguments (`src/types.rs` 1708–1722: "existence only, arguments ignored").
An edge

```
(:wat::core::extend-type :- [T]
  (:wat::core::Vector :- [T])
  :wat::core::Orderable)
```

would say every `Vector` is `Orderable`, including a vector of a
function. Nothing in the edge asks whether `T` is. Conditional membership
is not a thing the machinery says.

What is missing is that condition: "`(Vector :- [T])` is orderable when
`T` is", and the same for `Option`, `Result`, and a non-empty tuple.
A flat family bound on the two operands does not state it. The structural
walk is the class.

`infer_ordering` also accepts two different numeric leaves when unify
fails (`src/check.rs` 13738–13744, `both_numeric`). `(T, T)` with `T`
orderable rejects `i64` against `f64`. That exception is a second rule,
not the bound.

The `Var(_) => true` arm means an unresolved variable passes the gate.
A bound that refused an unresolved variable would be stricter than this
arm. A bound that deferred would match it. Which one a bound is, is a
design choice. It is not measured here, because nothing checks a bound.
Consequences are in Part B.

### Equality

255.45 called `=` and `not=` one clause. `infer_equality` does gate on
`is_type_equatable`, after a compatibility test (`src/check.rs` 13386–13408).

Compatibility is unify, or either path is a subtype of the other, or both
are records, or both are numeric (`13386–13396`). Then both sides must be
equatable.

`is_type_equatable` (`13505–13596`) is the same shape of structural walk,
with a wider leaf set (scalars, `Uuid`, `char`, `Instant`, `Duration`,
`holon/Vector`, `HolonAST`, `WatAST`, every aggregate and enum, newtype
and alias and union by recursion). `Vector`, `List`, `Option`, `Result`,
and tuples recurse. `HashMap`, `HashSet`, and `PersistentVector` are
blanket true. A type variable spelled as a param letter, and
`:wat::core::Value`, are true (defer). `TypeExpr::Var(_)` is marked
unreachable under that letter test and still returns true. `Fn` is false.
A surface path is false.

Same gap as ordering: the family edge cannot say "a vector is equatable
when its element is". Equality is not closed by a family bound. It is
closed by keeping this predicate, plus the compatibility rule, which a
`(T, T)` clause does not state (subtype, both-record, both-numeric).

### `select` and `poll`

Unifying `(Vector :- [(Thread :- [S R])])` with `(Vector :- [E])` is the
same-head arm (`src/check.rs` 16808–16814): the single arguments unify,
and a variable binds to the other side (`16714–16719`). `E` becomes
`(Thread :- [S R])`. No covariance. The bound is a check after that bind.
`satisfies_spawned` is `family_extends` of the head to `:wat::spawn::Spawned`
(`11230–11232`), and `Thread` and `Process` have that edge (`wat/spawn.wat`
261–267). A resolved `E` can be asked that question with the machinery
that exists. The check itself does not exist.

A bare vector literal infers one element type and unifies each element
into it (`src/check.rs` 16210–16278). `Thread` and `Process` do not unify,
so an unannotated literal cannot hold both. When the declared element
type is a surface, that same constructor uses `assignable` instead of
unify (`16265–16272`). `Spawned` is a surface, so an annotated
`(Vector :- [(Spawned :- [S R])])` can hold a `Thread` and a `Process`.
The live `select` in `wat/bracket.wat` 593–603 is the annotated
homogeneous case: one `Spawned` instantiation, not a mixed literal. A
walk of every `select` call was not done. The constructor rule is the
measurement.

`poll`'s listener is inferred and not constrained (`src/check.rs` 12323).
The clause states it as a type variable with no bound.

`select`'s admin parameter is a fresh variable because `select` has no
admin channel (`12291–12307`). A result variable that does not occur in
the parameters is how a clause says that, once the clause's free variables
are quantified. That part is the dispatch that already collects free
variables (`5645–5650`). It does not need a bound.

### `first`, `second`, `third`, `nth`

A bound does not name an element. The homogeneous containers are a finite
set. `infer_positional_accessor` names them in the mismatch: tuple,
`(Vector :- [T])`, `(List :- [T])`, `(PersistentVector :- [T])`, `WatAST`
(`src/check.rs` 10121). One clause each, returning the element type,
closes those. A tuple's slot type is the type at a fixed index
(`10071–10075`). That is an index-literal rule for the three accessors,
not a bound and not one more container clause. Tuple arity is open, so
the rule is "the type at position n", which a clause per arity would only
approximate.

`nth` unifies the index with `i64` (`10173–10184`). On a tuple it
refuses: a runtime index has no per-slot type (`10234–10242`). On an
unresolved receiver it returns a fresh variable (`10197–10198`). `nth`
is closed by the homogeneous clauses and by not having a tuple clause.
It is not a form.

## Part B — where a bound would live

Nothing stores one.

| place | what is there |
|---|---|
| `TypeScheme` | `type_params: Vec<String>` (`src/check.rs` 81–89). No bound |
| `InferCtx` | `fresh()` mints a `TypeExpr::Var`. No annotation on it |
| `Subst` | `unify` inserts `var → type` after the occurs check (`16714–16719`). No hook beside that insert |

`instantiate` (`17937`) and `instantiate_with_args` (`17969`) replace each
name with a fresh variable or a supplied type argument. They do not
consult a bound.

When a variable is still unresolved at the end of the call, ordering
accepts it (`Var(_) => true`). Defclause dispatch treats a missing
argument type as a match (`None => true`, `5668`) and unifies a fresh
variable into the first clause. A bound that refuses an unresolved
variable makes the p11 refusal the bound's own answer, for every variable
that carries the bound. A bound that defers, as ordering does, leaves
p11 as a separate rule for the rows that must see a concrete family.
Both are consistent with something that was measured. Choosing one is
the design, and this stone does not choose it.

`defn` binders are enforced as names: they are freshened for the body
(`src/function/infer.rs` 142–159) and again at each scheme instantiation.
They are not bounds. `defclause`'s parsed `type_params` is an empty vec
(`src/function/parse.rs` 863). Its check dispatch collects free names off
the clause (`src/check.rs` 5650) and stores `(arg types, return, has_rest)`
with no bound (`src/check/env.rs` 176–183). One mechanism has to grow
both stores. The binder parser has to grow for both `defn` and
`defclause`, because today it cannot see a bound.

## Part C — the three gaps, after a bound

| gap | still needed? | where |
|---|---|---|
| Rest elements are not typechecked at check-time dispatch | yes. A bound does not look at extra arguments. Eval already checks them | `src/check.rs` 5641 and 5665; `src/check/env.rs` 181; the check that exists is `src/function/eval.rs` 227 |
| `None => true` | yes, until a design says an unresolved variable fails its bound, and until a missing argument type is no longer treated as a match. Those are two holes. The bound closes the first only if it refuses | `src/check.rs` 5668 |
| `Vector` covariance under surface satisfaction | not for these function rows. `select` and `poll` are the two rows 255.45 named, and `(Vector :- [E])` plus a bound on `E` does not need the element slot to be covariant | the invariance is `src/check.rs` 17549–17589 |

## Part D — the 12

Each clause is the type the runtime function returns, read from its body
and its `@ret`. All twelve can be written except `struct-field`.

| row | clause | where |
|---|---|---|
| `fresh-symbol` | `(base <- String) -> WatAST` | `src/intrinsic/ast.rs` 387–407 |
| `macro-error` | `(msg <- String) -> Never`. The `@ret` says `nil`. The body always returns `Err` (`src/intrinsic/macro_error.rs` 78 and 82–93). `nil` is the wrong clause. `Never` is the type the timer already uses for an uninhabited result | |
| `str` | `(x <- Value) -> String` | `src/runtime.rs` 10938–10941 |
| `type-equal?` | `(a <- WatAST, b <- WatAST) -> bool` | `src/intrinsic/reflect.rs` 942–944 |
| `type-params-used-in` | `(params <- (Vector :- [WatAST]), node <- WatAST) -> (Vector :- [WatAST])` | `src/intrinsic/reflect.rs` 826–828 |
| `peer-pid` | `(peer <- (Peer :- [I O])) -> (Option :- [i64])`. The body returns `Some` pid for a spawned process peer and the doc says `None` for a thread peer (`src/kernel/identity.rs` 22–26). A closed peer and a timer raise | `src/intrinsic/kernel/identity.rs` 294–295 |
| `linkedlist/length` | `(l <- (List :- [T])) -> i64` | `src/intrinsic/linkedlist.rs` 67–68 |
| `linkedlist/empty?` | `(l <- (List :- [T])) -> bool` | 85–86 |
| `linkedlist/contains?` | `(l <- (List :- [T]), item <- T) -> bool` | 104–106 |
| `linkedlist/get` | `(l <- (List :- [T]), i <- i64) -> (Option :- [T])` | 124–127 |
| `metadata-of` | `(name <- keyword) -> (Option :- [(HashMap :- [keyword Value])])` | `src/runtime.rs` 7615–7616 |
| `struct-field` | no clause says the result. The body returns `fields[index]` of whatever aggregate it was given (`src/intrinsic/record.rs` 290–320). The index is an `i64`, not a literal, and the receiver is any aggregate. `@ret` says `:T`, and `T` is not determined by the arguments. A clause `(record, i64) -> T` with `T` free is the fresh variable the checker already returns. It is not the field's type | |

## The table

Every function row 255.45 left uncovered, and the 12. "Closed by a bound"
means a flat family bound, the kind `family_extends` can answer.

| row | what closes it | |
|---|---|---|
| `<` `>` `<=` `>=` | not a flat bound. The structural walk, plus the numeric-pair exception. Conditional container membership does not exist | measured |
| `=` `not=` | not a flat bound. `is_type_equatable` plus the compatibility rule (unify, subtype, both records, both numeric) | measured |
| `select` | a flat bound on the vector's element (`Spawned`, and a second clause for `Peer`). Unify binds the element with no covariance. The admin result variable is free | measured |
| `poll` | the same bound on the peers element. The listener is an unbound type variable | measured |
| `first` `second` `third` | one clause per homogeneous container, and a tuple rule for a fixed index | measured |
| `nth` | the homogeneous clauses. A tuple is refused today, so it gets no clause. An unresolved receiver returns fresh | measured |
| `fresh-symbol`, `str`, `type-equal?`, `type-params-used-in`, `metadata-of`, the four `linkedlist` rows, `peer-pid` | one clause, as in Part D | measured |
| `macro-error` | one clause to `Never`, not to `nil` | measured |
| `struct-field` | not closed. The result is the field at a runtime index | measured |

## What `defintrinsic` needs, in order

1. A bound stored beside `type_params`, on `TypeScheme` and on the
   defclause clause triple, and a binder parse that keeps it. Today both
   stores lack it, and the `defn` peel drops anything that is not a symbol.
2. A check of a flat family bound after unify has bound the variable, using
   `family_extends`. That closes `select`, `poll`, and any owner-or-peer
   clause written as one head plus a bound. It does not close ordering or
   equality.
3. The open choice for an unresolved variable: defer, as `Var(_) => true`
   does, or refuse, which is p11 for every bounded variable. Not chosen
   here.
4. For ordering and equality, either the two structural predicates stay as
   the meaning of those two bounds, including conditional containers and
   the numeric exception, or conditional family membership is built. A
   flat edge does not do it. Not chosen here.
5. Rest-element checking in the check-time dispatch. Independent of the
   bound. Still required for the six variadic rows.
6. `Vector` covariance is not required for the function rows, once (2)
   exists.

## Forms that stayed forms

255.45's other class-(iv) rows are forms, and this measurement did not
move them: `declare-acronyms`, `edn/validate`, `type-of`,
`field-names-of`, `field-types-of`, `aggregate-new`, `kwargs-construct`,
`to-record`, `holon/to-record`, `variant`, `retag-op`, `listener`,
`struct-new`, and the `Ok` / `Err` / `Some` refusal. Their arguments are
type keywords, syntax, or a refusal. `interpolate`'s tail is
keyword/value pairs, which is syntax, not a rest element type.
`struct-field` is the one that looks like a function and is not closeable
by a clause. `macro-error` looks like a form because it never returns; the
runtime type is `Never`, so it is a function row.
