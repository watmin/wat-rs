# SCORE — STONE 255.50: measure O1, the tuple binder, and what reaches the classes

Measurement only. Drawn against `c87989b11`. Measured on the draw
`d08e09319`, in a worktree. Nothing landed on `main` but this file.

One `wat --check` of each tracked file: 2286 `.wat` and 376 `.wat.bad`
(2662). The first file used `MEASURE_ALL`, so stdlib sites are logged
once; later files logged only their own spans. `select` and `poll`
lines are the type after bottom-up inference of the peers argument.
`infer_select_prime` infers that argument before it looks at the
element (`src/check.rs` 12233–12245). It does not push an expected
element type into the vector.

## 1. O1

15 `select` calls, 177 `poll` calls.

`poll`, every one, element `:wat::kernel::Peer`:

| how the vector is built | sites |
|---|---|
| `foldl` (the service expansion) | 171 |
| a parameter named `peers` | 2 |
| a parameter named `clients` | 2 |
| a parameter named `selectables` | 2 |

No `poll` vector is a `Thread`, a `Process`, a `Spawned`, or a `mapv`.

`select`:

| element the checker inferred | shape | where |
|---|---|---|
| `Spawned` | symbol `peers` | `wat/bracket.wat:603` (stdlib). Already an owner vector |
| `Thread` of `i64`/`i64` | vector literal | `tests/comms/probe_arc214_stone46b_select_prime.wat:32`, and the `.wat.bad` siblings `probe_arc214_stone46b_select_prime_probe2.wat.bad:10`, `probe_arc278_service_event_must_use_wall.wat.bad:15` |
| `Process` of `i64`/`_` | vector literal | `tests/kernel/peer_select_prime_process.wat:26` |
| `Process` of unit/unit | `(:wat::core::Vector …)` call | `tests/comms/probe_select_flood_no_deadlock.wat:34` |
| `Peer` | `Vector` call, or a symbol | the other 9 (6 `wat-tests` timers, `probe_arc278_self_scheduling.wat:68`, `probe-s3a-select-peer.wat:22`, `probe-self-scheduling-loop.wat:28`) |

A parameter typed `(Vector :- [(Spawned :- [i64 i64])])`, probed:

| argument | rc | result |
|---|---|---|
| bare `[t t]`, both `Thread` | 0 | the literal takes `Spawned` from the parameter. `check_vector_literal_against` (`src/check.rs` 16358–16382) checks each element with `assignable` and returns the expected element type |
| bare `[p p]`, both `Process` | 0 | same |
| bare `[t p]`, mixed | 0 | same. Both extend `Spawned` |
| `(:wat::core::Vector :- [(Thread :- [i64 i64])] t t)` | 1 | the constructor infers `Vector` of `Thread`. The parameter then refuses it. Invariance, not an up-cast |
| `mapv` of a function that returns `Thread` | 1 | same refusal. `got (Vector :- [(Thread :- [i64 i64])])` |

So a bare literal does take `Spawned` from the parameter. A vector that
was already built as `Thread` or `Process` — a constructor, or `mapv` —
does not.

Under O1, counted by where the vector is built:

| kind | already `Spawned` or `Peer` | literal of `Thread`/`Process` (the parameter would up-cast it) | constructor of `Process` (would not) |
|---|---|---|---|
| stdlib | 1 (`bracket.wat`) | 0 | 0 |
| test | 1 Peer call | 2 (one `Thread` literal, one `Process` literal) | 1 (`probe_select_flood_no_deadlock.wat:34`) |
| `.wat.bad` | 0 | 2 `Thread` literals | 0 |
| `wat-tests` / scratch | 9 Peer | 0 | 0 |
| `poll` | 177 Peer | 0 | 0 |

No `select` or `poll` argument is a `mapv`. The one constructor site is
the one that would need an annotation or a conversion. The four literals
would not, once the clause parameter is `Vector` of `Spawned`. They are
not `Spawned` where they are built today, because `select` does not
push that expected type.

## 2. The tuple binder

`&` inside a type binder is a parameter name, not rest.

`extend-type :- [&]` of `:wat::core::i64` exited 1:

```
extend-type :wat::core::i64 → :probe::Orderable: binder parameter & does not appear in the child type
```

`:- [& Ts]` on a tuple child rendered the child as `:(Ts,)` and named
the missing parameter `&`. Same door: `extend_type_operands` keeps bare
symbols (`src/types.rs` 5838–5848).

On `defn`, `:- [& T]` with the body producing `T` and the signature
declaring `&` exited 1:

```
:user::f: body produces :T; signature declares :&
```

`:- [T ...]` exited 1:

```
:user::f: body produces :T; signature declares :...
```

`...` is a type parameter whose name is three dots. `peel_type_binder`
(`src/function/metadata.rs` 52–58) keeps every bare symbol and drops
every other entry without an error.

`:<` is not a binder keyword. Used as a return type it is refused as an
angle bracket:

```
invalid return type: malformed type expression ":<" angle-bracket parametric types are illegal
```

A nested `[[T :< :wat::core::i64]]` is not a symbol, so that peel drops
it. The spelling `[& [Ts :< X]]` does not declare a rest and does not
declare a bound.

`defclause`'s value rest is a bool `has_rest` (`src/check/env.rs` 181)
plus a rest type that eval requires to be `(Vector :- [T])`
(`src/function/eval.rs` 227–235). `TypeExpr::Tuple` is a `Vec` of slots
(`src/types.rs` 104–109) with no rest flag and no tail element. Reusing
the value rest would mean giving `Tuple` an optional tail type, which
it does not have. The value rest also cannot see a type binder: it
runs on values.

Shapes, not a choice:

| spelling | measured |
|---|---|
| `[& Ts]` | `&` is a type name. The tuple's slots are not collected into it |
| `Ts ...` | `...` is a type name. Same |
| `[& [Ts :< X]]` | `:<` is an illegal angle bracket as a type keyword. A nested vector in a `defn` binder is dropped. No bound is declared |
| the walk that exists | `is_type_orderable` on a non-empty tuple asks each element (`src/check.rs` 13683–13685). No binder. The six ordering sites from 255.49 keep working. A declaration in the finding's syntax does not reach them |

## 3. What reaches the classes

Aggregates whose fields contain a function type, in the 2662 files:

| type | field | line |
|---|---|---|
| `:wat::gen::Gen` | `at <- [:wat::core::i64 :-> T]` | `wat/gen.wat:171` |
| `:wat::spawn::ThreadOpts` | `init-fn`, `post-spawn-fn` | `wat/spawn.wat:64–65` |
| `:wat::spawn::ProcessOpts` | `post-spawn-fn` | `wat/spawn.wat:90` |
| `:probe::apply-it::Kwargs` | a generated kwargs struct in a test | the catalog |

No enum variant payload contains a function. No `=` / `not=` / `<` /
`>` / `<=` / `>=` call reaches any of these four. The one comparison
that reaches a function type at all is the bare function in
`tests/types/probe_arc255_equality_domain_gate.wat.bad:24`, not a
record.

A struct that holds one, probed. `:probe::Box` with
`f <- [:wat::core::i64 :-> :wat::core::i64]`. A record of the same
field is refused first, because a function type is impure:

```
pure aggregate ":probe::Box" may only hold pure fields — field "f" has impure (struct) type "[:wat::core::i64 :-> :wat::core::i64]"
```

The struct checks. `--check` exited 0. Running it exited 1. `=` asks
`values_equal`, the aggregate arm recurses into fields (`src/runtime.rs`
5879–5896), and a function has no arm (`None`). The runtime error:

```
:wat::core::=: expected matching comparable pair, got wat::core::Struct `:probe::Box{#0: <fn>}`
```

`<` on that struct does not get that far. The orderable gate refuses
the struct itself:

```
:wat::core::<: parameter #1 expects an orderable type (...); got :probe::Box
```

Newtypes in the 2662 files. None are in `wat/`. No comparison reaches
any of them (no `REACH-NEW` line).

| name | inner |
|---|---|
| `:b::Price` | `i64` |
| `:diag::UserId` | `i64` |
| `:my::Amount` | `i64` |
| `:my::Price` | `f64` |
| `:my::PriceUsd` | `f64` |
| `:my::trading::Amount` | `f64` |
| `:my::trading::Price` | `f64` |
| `:probe::Count` | `i64` |
| `:demo::edn::NoTag` | `HolonAST` |

`is_type_equatable` recurses into the inner type (`src/check.rs`
13561–13563). `is_type_orderable` does not: a newtype path is not one
of the orderable leaves, so it is refused. The shapes, not a choice:

| shape | consequence |
|---|---|
| one `extend-type` per newtype, to `Orderable` or `Equatable` | membership ignores the inner type. `:demo::edn::NoTag` would be orderable even though its inner is `HolonAST`, which the orderable gate refuses |
| a conditional edge `(Price :- [T])` when `T` is | none of the nine is parametric. The edge would not attach to the types that exist. A monomorphic wrapper has no parameter for the binder to match |
| keep the equatable recursion and add the same recursion to ordering | no new declaration. Equatable already follows the inner type. Ordering would start to, which it does not today |

## What the finding did not foresee

1. Every `poll` vector is already a `Peer`. The owner-vector question
   is `select` only, and the only stdlib owner vector is already
   `Spawned`.
2. A bare literal up-casts to `Spawned`. A `Vector` constructor and a
   `mapv` do not. O1 is true of literals and false of vectors built
   before the call.
3. `&` and `...` are names. The finding's rest spellings declare type
   parameters, and `:<` is an angle bracket.
4. A struct may hold a function, the checker admits `=` on it, and the
   runtime raises. Nothing in the corpus does this. The four real
   function fields are never compared.
