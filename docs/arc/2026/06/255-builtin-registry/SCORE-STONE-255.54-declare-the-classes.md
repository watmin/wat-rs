# SCORE — STONE 255.54: declare Orderable and Equatable, and prove they agree

Struck against draw `5a6d18a7c` (brief drawn at `4d2202cd7`). `<` and `=` still
call `is_type_orderable` and `is_type_equatable`. Membership is proved by
passing a value to a parameter of the class, not by using the operator.

## The two surfaces

`wat/class.wat`, loaded immediately after `wat/core.wat`
(`src/load/stdlib.rs:47`). Both are featureless, and both are
`:nature :wat::core::Struct`. A Record nature would register the surface under
`:wat::core::Record`, and `is_subtype` is transitive, so `i64 <: Orderable`
would become `i64 <: Record`. Struct nature does not register that edge.

Leaves are the predicate's leaf arms. Orderable also has `bigint` and
`rational` (the runtime orders them; the predicate does not). Conditional
Orderable: `Vector`, `Option`, `Result` (both parameters), tuples (`:..`).
Conditional Equatable: `Vector`, `List`, `Option`, `Result`, tuples.
`HashMap`, `HashSet`, and `PersistentVector` are unbounded Equatable edges.
`PersistentMap` is not a member. One edge from `:wat::core::Record`. One edge
from `:wat::enum::Pure`.

A conditional edge is not a string subtype, so `Variant <: Enum` does not
carry it. The ordering predicate widens a variant first. The same widening is
declared for `Option.None`, `Option.Some`, `Result.Ok`, and `Result.Err`.

## Registration

A Pure enum registers `<: :wat::enum::Pure` in the same act as the enum
(`src/types.rs:1671`). A variant already registers `<:` the enum, so it
follows. An Impure enum does not.

A newtype whose inner type is pure registers `<: :wat::core::Record`
(`src/types.rs:1284`). An impure inner does not. A newtype's own Orderable
edge is refused unless `assignable(inner, Orderable)`
(`src/types.rs:4905`): `newtype :u::Box joins Orderable only when its inner
type is Orderable; :u::St is not`.

## Agreement

The corpus was walked once. Operand types that reach `<`/`>`/`<=`/`>=`/`=`/`not=`
are in `tests/types/probe_arc255_54_class_hits.txt`. The unique list, 71 rows,
is `tests/types/probe_arc255_54_class_types.txt`. The test loads
`probe_arc255_54_class_world.wat` so the user records and enums in that list
resolve, then asks `assignable` and the predicate.

| row | declared | predicate | disposition |
|---|---|---|---|
| ord `:wat::core::bigint`, `:wat::core::rational` | true | false | ruled correction |
| eq struct | false | true | Q1. None of the 71 rows is a struct under `=` |
| eq Impure enum | false | true | EN-P. None of the 71 rows is Impure |
| eq newtype of an impure inner | false | true | N-R. None of the 71 rows is a newtype |
| eq `:T` | false | true | finding. Rigid path. The predicate defers |
| eq `:wat::core::Value` | false | true | finding. Value is the universal supertype |
| ord `:wat::core::nil` | true | false | finding. See below |
| the other 66 rows | same | same | agree |

`_` agrees, and that is the code, not the brief's Refuse. `assignable` ends
in `unify` (`src/check.rs:18083`), which binds a variable to the class. A gate
written as `assignable(T, Orderable)` admits `_`.

`:wat::core::nil` is an alias of `TypeExpr::Tuple([])` (`src/types.rs:2102`).
The `:..` edge admits every slot, and there are none, so the class says
Orderable. The predicate refuses the path and also refuses an empty tuple.
255.53 left the empty tuple unpinned. This stone does not add an arity check.
The value `nil` is `Value::Unit` (`src/runtime.rs:1603`). `values_compare` has
no `Unit` arm (`src/runtime.rs:6080`, `_ => None`). The `Value::Tuple` arm
would compare two empty tuples as Equal; that is not the value `nil` is.
STOP-2 describes this row. It was not worked around: no `Unit` arm was added,
and the empty-tuple rule was not special-cased. `<` still uses the predicate,
so `tests/types/ord_unit.wat.bad:3` still fails. 255.55 must not switch `<`
onto the class until this row is decided.

A newtype of an Orderable inner may declare its own Orderable edge (N-R).
`values_compare` has no `Aggregate` arm, and a newtype value is an aggregate.
No corpus site orders a newtype. The same warning applies before 255.55
switches `<`.

## What 255.55's switch meets

Unchanged by this stone.

Rigid `:T`. The class refuses, the predicate admits. The switch refuses these:

| site | fix |
|---|---|
| `wat/test.wat:62` `assert-eq :- [T]` | `[T :< Equatable]`. Load-bearing |
| `wat/seq.wat:560` `(= p value)` inside `dedupe-walk :- [T]` | the same bound on `T` |
| `wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat:32` | the hole this deferral exists to show. The same bound |

`:wat::core::Value`. `wat-scripts/scratch-pad/255-probe-metadata-of-one-shape.wat:63`
compares two `Value`s. The class refuses. Declaring `Value <: Equatable`
would not be a membership of the payload. The fix is to keep the deferral or
to stop comparing `Value`.

`_`. Both answers are true, because `assignable` unifies the variable. The
brief says Refuse. The code admits them. Sites: `tests/collection/list.wat:46`,
`tests/types/probe_arc234_7a_base_record_roundtrip.wat:16`,
`tests/types/probe_arc234_7b_holon_record_roundtrip.wat:16`,
`tests/types/uuid_edn_roundtrip_typed.wat:7`,
`tests/value/wat_arc220_char.wat:79`, `wat/doctest.wat:118`,
`wat/rete/acc.wat:68` and `:87`, the three
`tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat:1`,
`06b-less-equal.wat:1`, `07-greater-than.wat:1`, and five scratch-pad probes
in the hits file. A real Refuse has to answer before the `unify` fallthrough.

`Pt`/`HPt`. `tests/types/probe_arc237_sC3_macro_split.wat:41`. Each record is
Equatable. The pair is admitted by the both-records arm
(`src/check.rs:13421`), not by the class. Switching the predicate does not
refuse this line. Dropping that arm would. The fix would be to compare one
record type, or to use `Record/same-data?`.

Subtype arm. The only two sites are
`wat-scripts/scratch-pad/277-the-node-kind-boundary.wat:73` and `:74`
(`:user::NodeKind` against `.Map` and `.Set`). Widening each side to its
enclosing enum makes both `:user::NodeKind`, and `unify` then succeeds.
Variant widening covers them. No subtype-only site is left uncovered.

## Proof

`cargo test --test types probe_arc255_54 -- --skip collect`: 6 passed, rc 0.

Floor `.floor/2026-09-27T04-49-53Z` was red and was not re-run.
`Summary [ 358.712s] 6179 tests run: 6178 passed (13 slow), 1 failed, 23 skipped`.
The arm is `tests/lint/tracked_wat_dir_is_stdlib_sources.rs:60`:
`only-tracked: []`, `only-loaded: ["wat/class.wat"]`. `class.wat` was in the
load list and not yet in the git index. `git add -- wat/class.wat`, then a
new floor.

Floor `.floor/2026-09-27T04-57-14Z`: 6179 passed, 23 skipped, rc 0. That is
6173 at `0bc2a4f4c` plus these 6 rows. The extra skip is the ignored
collector. STOP-3 did not fire: the Pure-enum and pure-newtype edges did not
turn any existing check red.

Clippy `--release --all-targets -- -D warnings`: rc 0.

Pre-census `.census/2026-09-27T03-06-10Z.txt` (2290, the unmodified draw).
Post `.census/2026-09-27T05-03-39Z.txt`: `census-diff: no STOP-8`. 0 rc flips.
215 nonzero of 2293. Three new files, each rc 0: `wat/class.wat`,
`tests/types/probe_arc255_54_classes.wat`,
`tests/types/probe_arc255_54_class_world.wat`.

Delta `.delta/2026-09-27T05-04-36Z`: NEW 2 / RECOVERY 0. The two files are
`wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.
