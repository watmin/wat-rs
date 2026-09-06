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
