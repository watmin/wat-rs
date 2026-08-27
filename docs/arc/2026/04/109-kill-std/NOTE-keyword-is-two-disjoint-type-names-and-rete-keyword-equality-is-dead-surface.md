# NOTE — `Keyword` is two disjoint type names, and rete's keyword equality is dead surface

**Found** 2026-08-27, from arc 278, while widening the rete differential fuzzer to the scalar-type
surface. **Handed to arc 109** because the root is a type-NAME split, which is this arc's ground.
Not fixed here; nothing in arc 278 depends on it.

**Builder's read on hearing it:** *"huh.... feels like a miss...."* — it is.

## The one-line version

`:wat::core::Keyword` and `:wat::core::keyword` are **two different type names**, and each is
missing exactly what the other has. A field declared with the capitalised one is recognised by
rete but can never hold a value; a field declared with the lower-case one can hold a value but
rete refuses to compare it. The consequence is that the `keyword::=` and `keyword::not=` rows in
`RETE_OPS` are **unreachable from any user record** — minted, gated, documented, and dead.

## Measured, three probes, on `2361bf8b3`

**1. The capitalised name is an accepted field type with NO INHABITANTS.**

```wat
(:wat::core::defrecord :kw::R [v <- :wat::core::Keyword])   ;; declares clean, exit 0
(:kw::R :alpha)
;; #wat.check/TypeMismatch — :kw::R: parameter #1 expects :wat::core::Keyword;
;;                          got :wat::core::keyword
```

The declaration type-checks. Every construction of it cannot. That is a shape whose only possible
use is a compile error — the arc's own standard says the wrong thing should have no form, and here
it has a form that looks right.

**2. The lower-case name is constructible but NOT COMPARABLE.**

```wat
(:wat::core::defrecord :kw::R [v <- :wat::core::keyword])
:when [(:kw::R (?v <- :v) (:wat::rete::core::keyword::= ?v :alpha))]
;; #wat.rete/ConstraintTypeNotComparable — `:wat::rete::core::keyword::=` compares operand `?v`,
;; declared `:wat::core::keyword`, for which rete has NO comparator — the rete equality surface
;; is i64/f64/string/bool/keyword/enum. Compare a scalar FIELD of it instead
```

**The diagnostic contradicts itself**: it lists `keyword` as part of the surface in the same
sentence that refuses a keyword. The cause is one line — `rete_type_segment_of`
(`src/rete/validate.rs`) matches `"wat::core::Keyword"` and nothing maps the lower-case spelling,
so it falls through to the enum-registry lookup, misses, and returns `None`.

**3. Even with the types aligned, a keyword LITERAL cannot be written as an operand.**

The same probe also reports `UnknownField { field: "alpha" }` — `:alpha` in operand position is
read as a FIELD REFERENCE. That is deliberate and documented at `src/rete/matcher.rs`'s
`ast_literal_value`: *"Keyword-as-field stays out: in operand position a keyword is a field
reference, never a keyword value."*

So there are three independent blocks, and removing any one or two of them still leaves keyword
equality unreachable.

## Why this is arc 109's

The capital/lower-case split is a type-NAME normalisation defect, which is what this arc annihilates
(`:wat::core::String` vs `String`, the angle-bracket sweep, `a-type-reference-must-resolve`). Every
other rete scalar has ONE spelling that is both inhabitable and recognised: `wat::core::i64`,
`wat::core::f64`, `wat::core::String`, `wat::core::bool`. Keyword is the only one that forked.

## What a fix has to decide (NOT decided here)

1. **Which spelling survives.** The value's own type is `:wat::core::keyword`, and values are the
   thing that cannot be renamed by fiat, so the lower-case one looks like the survivor — but
   `String` is capitalised and also a value type, so the convention is not self-evident and this is
   a ruling, not a lookup.
2. **Whether the loser is REFUSED or ALIASED.** Aliasing keeps existing source working; refusing
   makes the uninhabitable declaration impossible. The arc's own ladder argues for refusing the
   name outright, since a declaration that can never be constructed is exactly a form the mistake
   can be written in.
3. **Whether a keyword literal becomes writable in operand position.** Without this, fixing (1)
   and (2) still leaves `keyword::=` usable only for comparing two keyword FIELDS to each other,
   never a field to a constant — which is the shape a rule actually wants. Note this one is a
   deliberate grammar choice, not an oversight, so changing it needs its own justification.

## A gate this would have failed, if one existed

Nothing asserts that every `RETE_OPS` row is REACHABLE from a user record. The rows are gated for
purity, totality, arity and type — but not for "can a user actually get here". `keyword::=` has
passed every one of those gates since it was minted. A reachability ledger over `RETE_OPS`
(*for each row, one user-authored rule that exercises it*) would have caught this the day the row
landed, and would be the durable cure rather than fixing this one instance. Arc 278's rete fuzzer
now covers i64/f64/string/bool/enum and would host such a ledger naturally — it found this gap by
trying to add the sixth type and failing.

## Where this came from

`wat-tests/rete/differential-fuzz-scalars.wat` — the scalar-type differential fuzzer, which covers
five of the six comparator modules. Its header states keyword's absence and why. That header
originally claimed "there is no `:wat::core::Keyword` record-field type", which is FALSE and was
corrected when these probes were run: the type name exists and is accepted; it is the inhabitants
that do not.
