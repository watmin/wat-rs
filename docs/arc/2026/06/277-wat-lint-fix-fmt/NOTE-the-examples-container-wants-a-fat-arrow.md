# NOTE — the `:examples` container wants a fat arrow. Deferred, NOT forgotten. (2026-09-06)

> **Builder, on seeing a rendered `#wat.doc/Row`:** *"i think we'll need to work on the container for
> examples... seeing `[[` at the lead is odd..... maybe the syntax needs something like…*
> ```
> :examples
> [(some-form) :=> some-val
>  (another-form) :=> another-val]
> ```
> *we use this like how we do fn-types ?.... `[A B :-> C]` … the colon-fat-arrow is the declaration
> of a return value for some left hand form?..... i think the change to examples....is something to
> chase later.. but not something to forget...."*

## THE SHAPE TODAY, and why `[[` reads badly

```
:examples
[[(:wat::core::do …)          ← a vector OF PAIRS, so the lead is always `[[`
  (:wat::rete::DerivationStep …)]
 [(:wat::core::char "x")
  (:wat::core::char "x")]]
```

An example is an `[expression expected]` pair, so a list of them opens `[[`. **Nothing marks which
side is which** — the reader infers it from position.

## THE PROPOSAL

```
:examples
[(some-form)    :=> some-val
 (another-form) :=> another-val]
```

★ **The argument is consistency with a marker the language already has.** `:->` separates a function
type's inputs from its output (`[A B :-> C]`); `<-` binds a parameter to its type; `->` introduces a
return type. **`:=>` would be the same idea for "this form yields this value"** — one flat vector,
the relation named rather than positional, and the `[[` gone.

⭐ **And it would compose with rules that already exist.** `:=>` is a two-token slot exactly like
`-> T`, so the arrow-glue rule (*find the arrow child, glue the child after it*) applies unchanged —
and the sibling-table rule would align the `:=>` column across examples for free, which is the very
shape the builder called *"extremely easy to read"*.

## ★ REFINED — VERTICALLY STACKED, one blank line per example (2026-09-06)

> **Builder:** *"i think the examples... should be vertically stacked...."*
> ```
> :examples
> [(some-form)
>  :=> some-val
>
>  (another-form)
>  :=> another-val]
> ```
> *"we have a blank line per example?... the colon-fat-arrow and the ret val get the same line?"*

**Yes to both — and the vertical form is strictly better than the horizontal one first sketched.**

### ⛔ WHY, and it is a reason the first sketch would have failed on

```
[(some-form) :=> some-val]      readable ONLY while the expression fits on a line
```

**71 of 476 `@example` expressions exceed 120 characters in source**, and the worst (`step-payload`,
1,153 chars) formats to **35 lines**. A horizontal `:=>` works for `(char "x") :=> (char "x")` and
collapses for exactly the examples that motivated this work. **The vertical stack is
size-independent.**

### ★ AND IT NEEDS NO NEW CAPABILITY — three existing rules cover it

```
the EXPRESSION      every layout rule already built
`:=> val`           a TWO-TOKEN SLOT — the arrow-glue rule, unchanged.
                    Identical in shape to `-> T` and `:- V`, both already ruled.
the BLANK LINE      BlankBefore, already built — including its "never before the FIRST"
                    semantics, which is exactly what the builder's sketch shows.
```

The arrow-glue rule is *"find the arrow child, glue the child after it."* It needs `:=>` beside
`->` — **one token in one rule.**

### `:=>` ALREADY READS — measured, not assumed

```
read-string ":=>"            ->  kind=keyword  src=:=>
read-string "[(f 1)\n :=> 2]" ->  kind=vector   src=[(f 1) :=> 2]
```

**No lexer change, no language change.** Inside a `#wat.doc/Row` it is ordinary keyword data.

### ⛔ AND AN ARGUMENT OF MINE THAT I WITHDRAW

I argued for the fat arrow partly because *"the sibling-table rule would align the `:=>` column
across examples for free."* **That is dead under the vertical stack** — separate lines mean there is
no column to align. It was an argument for the HORIZONTAL form and must not ride into this one.

The surviving arguments are the stronger ones: **the relation is named rather than positional, the
`[[` lead is gone, and it composes with rules that already exist.**

## ⚠ WHAT IT TOUCHES — this is why it is deferred, not done

```
crates/wat-macros/src/edn_doc.rs    parse_edn_doc_row's :examples decoder
crates/wat-doc/src/print.rs         the printer that emits :examples
the doctest gate                    it RUNS these pairs; it must learn the new shape
the 615 existing @example lines     they are `expr #=> expected`, converted by a sweep
```

⛔ **The doctest gate is the load-bearing one.** It armed at zero this week and immediately caught a
public API example stale through two API changes. **A syntax change to `:examples` must not disarm
it**, and proving that is most of the work.

## STATUS

**Deferred by the builder, recorded so it is not rediscovered.** It is not on the arc-255 migration's
critical path — the current `[[…]]` shape works, the doctest gate runs it, and the formatter renders
it correctly. **This is a readability improvement to make deliberately, not a defect to fix.**

⚠ **If the 255 doc migration converts 575 rows before this lands, they convert again.** That is the
real cost of the ordering, and it should be a conscious choice rather than a discovery. The same
consideration parked the migration once already
(`[[PARKED-the-migration-waits-on-wat-fmt]]`: *"the sweep would … force a re-sweep"*).
