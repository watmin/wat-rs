# NOTE — `Keyword` is two type names, and only one of them can be compared inline

**Found** 2026-08-27 from arc 278, widening the rete differential fuzzer to the scalar surface.
**Handed to arc 109** because the root is a type-NAME split, this arc's ground. Not fixed here.

**Builder's read on hearing keyword was missing:** *"huh.... feels like a miss...."* — it is, though
not the miss either of us first thought.

> ⚠ **THIS NOTE WAS WRONG ON ITS FIRST WRITING AND IS KEPT AT THE SAME PATH DELIBERATELY.** Its
> first version claimed rete's keyword equality was "dead surface, unreachable from any user
> record". That is FALSE, and the disk disproved it within the hour:
> `wat-scripts/perf/grid/../scratch-pad/probe-cond-rete-where.wat` is a live, compiling, FIRING
> rule that declares `[tier <- :wat::core::keyword]` and compares it with
> `:wat::rete::core::keyword::=`. The filename still says "dead surface" because renaming it would
> break the citation in `differential-fuzz-scalars.wat` and in arc 278's RETE-OPEN-WORK; the
> headline above is the correct claim. **The lesson is the useful part: "I grepped and found
> nothing" and "this cannot be written" are different statements, and the first was reported as
> the second.**

## What is actually true

**`:wat::core::keyword` (lower-case) is a real, working field type.** Declared, constructed,
bound, and compared — proven twice, once by the pre-existing probe above and once by an
independent one written for this NOTE:

```
all=2  where-eq=1        ;; two keyword facts inserted; `keyword::=` in a `where` selects one
```

**`:wat::core::Keyword` (capital) is an accepted field type with NO INHABITANTS.**

```wat
(:wat::core::defrecord :kw::R [v <- :wat::core::Keyword])   ;; declares clean, exit 0
(:kw::R :alpha)
;; #wat.check/TypeMismatch — expects :wat::core::Keyword; got :wat::core::keyword
```

The declaration type-checks; every construction of it cannot. A form whose only possible use is a
compile error.

**And the two facts collide in one place, which is the actual defect:**

`rete_type_segment_of` (`src/rete/validate.rs`) maps `"wat::core::Keyword"` — the capital,
uninhabitable spelling — and nothing maps the lower-case one. So the lower-case type falls through
to the enum-registry lookup, misses, and returns `None`:

```wat
;; INLINE ALPHA CONSTRAINT on a keyword field — REFUSED
:when [(:kw::R (?v <- :v) (:wat::rete::core::keyword::= ?v :alpha))]
;; #wat.rete/ConstraintTypeNotComparable — ... declared `:wat::core::keyword`, for which rete has
;; NO comparator — the rete equality surface is i64/f64/string/bool/keyword/enum

;; THE SAME COMPARISON IN A `where` FENCE — WORKS
:when [(:kw::R (?v <- :v))
       (:wat::rete::where (:wat::rete::core::keyword::= ?v :alpha))]
```

Same record, same field, same op, two spellings of the same rule — one refused, one fires. The
`where` path works because a fence's interior is deliberately OUT of this wall's scope (design
call 3 in `validate.rs`), so it never consults `rete_type_segment_of` at all.

**The diagnostic is also self-contradicting**: it lists `keyword` as part of the equality surface
in the same sentence that refuses a keyword, and says rete has "NO comparator" for a type that
demonstrably has a working one one line away.

## A third, independent wrinkle

Even with the type mapping fixed, `:alpha` in INLINE OPERAND position is read as a FIELD
REFERENCE, not a keyword literal — deliberate and documented at `src/rete/matcher.rs`'s
`ast_literal_value`: *"in operand position a keyword is a field reference, never a keyword value."*
(Inside a `where` fence there is no such grammar, which is the other half of why that path works.)
So fixing the type map alone makes `keyword::=` usable inline only for comparing two keyword
FIELDS to each other, never a field to a constant.

## Why this is arc 109's

Every other rete scalar has ONE spelling that is both inhabitable and recognised —
`wat::core::i64`, `wat::core::f64`, `wat::core::String`, `wat::core::bool`. Keyword is the only one
that forked, and the recognised half is the uninhabitable half. That is a type-NAME normalisation
defect, which is what this arc annihilates.

## What a fix has to decide (NOT decided here)

1. **Which spelling survives**, or whether the loser is aliased or REFUSED. A declaration that can
   never be constructed is exactly a form a mistake can be written in, which argues for refusing
   the capital name outright rather than aliasing it.
2. **Whether `rete_type_segment_of` should map the lower-case name** so the inline path matches the
   `where` path. This is the smallest possible fix and would remove the two-paths-one-works
   asymmetry — but see (3) before assuming it is sufficient.
3. **Whether a keyword literal becomes writable in inline operand position.** Without it, (2) buys
   only field-to-field comparison. This one is a deliberate grammar choice, so changing it needs
   its own justification, not just "it would be consistent".

## The durable cure is not this row

Nothing asserts that every `RETE_OPS` row is REACHABLE. Rows are gated for purity, totality, arity
and type, never for "can a user get here". A reachability ledger is proposed in arc 278's
`RETE-OPEN-WORK.md` § 4.1 — and **this NOTE's own error is the strongest argument for what such a
ledger must be**: a grep-based one would have called `keyword::=` dead (it appears in only two
scratch-pad files), and a compile-based one would have called it fine. Reachability has to be
asked PER CALL SITE KIND — inline constraint vs `where` fence are different reachability questions
about the same row, and this row is reachable in one and not the other.
