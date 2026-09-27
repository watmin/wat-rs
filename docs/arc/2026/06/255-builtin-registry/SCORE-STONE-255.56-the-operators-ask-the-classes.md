# SCORE — STONE 255.56: STOP-2. Two stdlib operands have no type to pin

Struck against draw `56faa2bc7` (brief drawn at `e8646678e`). The gates were
not switched. `<` and `=` still use the predicates. Nothing in `src/` or
`wat/` changed.

## Why this stops

Step 5 says to pin each unresolved stdlib operand to the type the value
actually has, and not to cast it to a class. STOP-2 fires when that type
is not one type.

`wat/doctest.wat:118` is `(:wat::core/= got want)`. `got` is the `:Ok` payload
of `(:wat::eval-ast! (:wat::intrinsic::Example/expr ex))` and `want` is the
`:Ok` payload of a second call on `expected-ast`. `:wat::eval-ast!` returns
`(Result :- [T :wat::core::EvalError])` and `T` is the caller's expected
value (`src/check.rs:20540-20562`). Each call mints its own `T`. An example
evaluates to whatever that example returns. There is no one type to write
on either call. `:wat::core::Value` is the holder the 255.54 finding already
refused to call `Equatable`, and `:wat::WatAST` is the input of `eval-ast!`,
not its result.

`wat/rete/acc.wat:68` and `:87` compare `v` with `cur`. `cur` is the `:value`
of an `(:wat::core::Option :- [:wat::core::i64])` accumulator, so that side
is `i64`. `v` is `(:wat::core::Option/expect (:wat::map::get bindings var) …)`.
`:wat::rete::Element.bindings` is the bare `:wat::core::PersistentMap`
(`wat/rete.wat:40`), and `wat/rete/oracle/accum-pass.wat:26` says that bare
map accepts any accum result. `:wat::map::get` returns `(Option :- [V])`
with `V` fresh (`src/check.rs:23119-23127`). A `let` binding has no type
slot (`src/check.rs:8403`, a flat `[name expr]` vector), so there is no
place to write "this binding is an i64" without claiming the map is
`(:wat::core::PersistentMap :- [:wat::core::String :wat::core::i64])`.
That claim is false for the other accum results that share the map.

Both are the STOP-2 shape: the value is heterogeneous, and the type the
checker sees is a fresh variable because the producer was declared that way.
No gate was added on top of that.

## What a switch would do to the other named sites

Read from the schemes, not from a floor. The work list's order is: widen,
then `unify` or the numeric cross, then refuse an operand that is still a
`TypeExpr::Var`, then the class. A variable unified with a concrete operand
is no longer unresolved.

These stay a concrete type after that `unify`, so they are not STOP-2:

| site | other side |
|---|---|
| `tests/collection/list.wat:46` | the literal `20` (`i64`) |
| `tests/types/probe_arc234_7a_base_record_roundtrip.wat:16` | the record `p` |
| `tests/types/probe_arc234_7b_holon_record_roundtrip.wat:16` | the record `h` |
| `tests/types/uuid_edn_roundtrip_typed.wat:7` | the uuid `u` |
| `tests/value/wat_arc220_char.wat:79` | the char `orig` |

`:wat::edn::read` returns a fresh `T` (`src/check.rs:21610-21618`). The
round-trip's other operand is the value that was written.

These do not unify to a concrete type:

| site | both sides |
|---|---|
| `wat/test.wat:62`, `wat/seq.wat:560`, `wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat:32` | rigid `:T`. Step 5's `[T :< Equatable]` is the fix, and it was not applied |
| `wat-scripts/scratch-pad/255-probe-metadata-of-one-shape.wat:63` | two `:wat::core::Value`s. Step 5 says stop comparing them. Not applied |
| `tests/resolve/probe_arc251_fix_source_local_rules__contract-06a-less-than.wat` and `06b`, `07` | the whole file is `(wat.core/< a b)` (and `<=`, `>`). `a` and `b` have no type. Census `.census/2026-09-27T06-55-26Z.txt` has each at rc 0. After Refuse they would not check. They are not step-5 sites. That flip would be STOP-1, so they were not edited |

## Proof

Pre-census `.census/2026-09-27T06-55-26Z.txt` (2294, the unmodified draw).
No post-census, no floor, no clippy: the tree is the draw.
