# DESIGN — arc 109: a type declaration must CONSUME its param-spec

**Status: DRAWN 2026-08-21, builder-ruled.** Written against `4c3a08ea7`.

> Builder: *"we are in the business of building walls — users may not make mistakes — any type def
> that declares a param-spec must fully consume it."*

```clojure
(wat.core/defrecord ns/R :- [T] [])                      ;; ILLEGAL — T consumed by nothing
(wat.core/defenum   ns/E :- [I O] … [First :- [I] …])    ;; ILLEGAL — O consumed by nothing
(wat.core/defrecord ns/R :- [T] [x :- T])                ;; legal
(wat.core/defrecord ns/R :- [T] [x :- (wat.type/Vector :- [T])])  ;; legal — consumed by NESTING
```

## What is true today — measured

Every one of these is **clean** at HEAD:

```
(defrecord :user::R<T> [])                       zero fields, T unused
(defrecord :user::R<T> [x <- :wat::core::i64])   T unused
(defrecord :user::R<T,U> [x <- T])               U unused
(defenum   :user::E<T> … [f <- :wat::core::i64]) T unused
(defn      :user::f<T> [x <- i64] -> i64 x)      T unused   ← stays legal, see scope
```

So the wall is entirely new.

## ⚠ The justification is NOT "the param is useless" — measured, it is not

An unused param still **discriminates**:

```
(defrecord :user::Phantom<T> [x <- :wat::core::i64])
passing Phantom<String> where Phantom<i64> is wanted  →  REJECTED
```

`Phantom<i64>` and `Phantom<String>` are genuinely different types with no field mentioning `T`.
That is nominal tagging — Rust's `PhantomData` use case, which wat currently allows without
ceremony. **So this wall REMOVES a working capability**, it does not merely forbid a no-op.

★ The UselessMain analogy that suggested this rule does **not** hold: a useless `main` computes
nothing and is observationally identical to absence; an unused type param partitions the type. The
wall is justified on a different axis, and the honest one is READABILITY:

> A param no field mentions makes the declaration's intent unreadable — a reader cannot tell a
> deliberate tag from a leftover edit, and *"the verbosity is our shield"* says the difference must
> be written, not inferred.

Rust reached the same verdict by a different route (variance and drop-check): `struct Foo<T>;` is an
error — *"parameter `T` is never used"* — with `PhantomData` as the explicit opt-in. **If phantom
typing is ever wanted here, it should arrive as a marker, not as silence.**

## Scope: type declarations YES, functions NO

The builder's scoping, and it matches Rust exactly. A function's unused `T` is supplied by the
caller under the call-site type-application rule (*"if a defn declares a parametric, the caller must
declare it too"*), so it is explicit at every use site. An aggregate's is written once and then
invisible forever.

## The blast radius — measured BEFORE drawing, and it is zero

An indicative scan of every parametric type declaration in the corpus:

```
wat/           21 declarations   0 violations
wat-scripts/   13 declarations   0 violations
tests/          4 declarations   0 violations (1 apparent hit was inside a STRING literal —
                                  a migrator fixture feeding source text to a codemod)
```

⚠ **The scan is indicative, not authoritative** — a regex that cannot tell code from string
literals, which is exactly how the one false positive arose. **The wall itself is the instrument**;
this only establishes that it is a RATCHET on existing practice rather than a migration.

## The shape — ONE validator, not seven checks

`TypeDef` has six variants — `Aggregate`, `Enum`, `Newtype`, `Alias`, `Union`, `Surface` — and
**all six carry `type_params`**. So the check runs once over a fully-built `TypeDef`, after
`parse_type_decl` returns, rather than being threaded into each declarator's parser:

```
for each declared param P:
    P must appear in at least one TypeExpr reachable from this TypeDef's members
    (fields · variant fields · newtype inner · alias body · union members · surface fields)
```

★ **Consumption must walk NESTED type expressions.** `[x :- (Vector :- [T])]` consumes `T`, as does
`[x :- (HashMap :- [K (Vector :- [V])])]`. A check that only looks at each member's HEAD would
reject legitimate declarations — and it would do so on exactly the forms this whole arc is
introducing. That is the one way this wall goes wrong quietly.

## The diagnostic

The error must name the param, the declaration, and the remedy — this is `RVINA ERVDIT` territory:

> `type parameter "O" is declared but never used — every param in a type declaration's param-spec
> must be consumed by a field, variant, or body type. Remove it from the param-spec, or use it.`

## Out of scope, affirmatively

- **Functions.** `defn`'s unused params stay legal; the caller declares them.
- **A phantom-typing opt-in.** Not built, because nothing in the corpus wants one. If demand
  arrives it is a marker type, tracked then.
