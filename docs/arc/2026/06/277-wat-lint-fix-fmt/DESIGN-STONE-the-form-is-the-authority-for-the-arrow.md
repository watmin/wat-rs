# DESIGN — STONE: the FORM is the authority for `->`, not the grammar

> **Builder:** *"we just need to mature the rules - we have the full rete expression system to
> manage our complexity."*

## THE DEFECT — the non-negotiable, violated on one shape

```
    (:wat::core::fn :- [:wat::core::i64] [acc <- :wat::core::i64 x <- :wat::core::i64]
      ->
      :wat::core::i64          ⛔ "ret-spec is a single line... i will not accept otherwise"
      (:wat::i64::+ acc x))
```

## THE CAUSE — index arithmetic against a grammar with an OPTIONAL slot

```
grammar   (fn [params] -> :RetType body)              `->` at 2, its type at 3  ->  Slot glued=3
reality   (fn :- [T] [params] -> :RetType body)       the param-spec shifts EVERYTHING BY TWO
                                                      `->` is at 4, its type at 5
```

`glued=3` withholds the **params**, not the return type. **The grammar cannot know which optional
slots a particular form instance actually used.**

## THE RULE

> **Find the `->` CHILD IN THE FORM. Glue the child after it.**

No index. No arithmetic. No registry lookup. Immune to every optional slot, present and future —
because the form being laid out is the thing that knows what it contains.

★ **This is `wat-fix`'s and `wat-grep`'s own discipline**: read the actual tree, not a description
of it.

## ⭐ WHY LEXICAL BEATS THE GRAMMAR *HERE*, on the record

Three measurements, each from a different stone:

```
1  the registry route produces only 3 slots of 36 grammars   (SLOTS=3)
2  it cannot reach type applications at all — no @syntax on HashMap/Vector/Tuple
3  it MIS-INDEXES the one form it was built for               (this defect)
```

⚠ **`->` and `:-` are LANGUAGE, not per-form policy.** A rule about them belongs where syntax lives.
The registry is the authority for what a form *declares*; the form is the authority for what it
*contains*.

## ⛔ AND THIS ORPHANS `Slot` — reported, NOT acted on

`Slot`'s only consumer is the `->` glue. After this stone it has **zero consumers**.

**That is a finding for the builder, not a deletion to slip into this stone.** Two honest readings:

- **Delete it.** A design with no consumer is unfalsifiable
  (`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`), and
  `[[NOTE-the-registry-already-knows-the-slots]]` preserves everything the measurements taught —
  the grammars parse, the head-spelling hazard is real, the refusal discipline works.
- **Keep it.** It is the right home for anything a grammar knows that syntax alone cannot, and such
  a case may well arrive.

⚠ **I argued for keeping it in the last verdict. I now think that was me defending my own earlier
argument** — deletions clear a high bar here, so it is the builder's call either way, and it is not
this stone's business.

## THE ACCEPTANCE — and the wording is deliberate this time

```
1  ★★★ a GENERIC fn renders `-> :wat::core::i64` AS ONE LINE — both tokens, same line
2  ★ a NON-generic fn still does (no regression)
3  ★ defmacro — the third Slot consumer — still does
4  every fixture idempotent; every ruled shape holds; three walls stand
5  Slot's orphaning is REPORTED with a consumer count, not acted on
```

⛔ **Row 1 says "AS ONE LINE — both tokens, same line", not "on its own line".** My previous
EXPECTATIONS said the latter, and `->` and the type each having their own line satisfied it. The
strike read my row and was right to. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## OUT OF SCOPE

- **Deleting `Slot`** — above; the builder's call.
- **`:-`** — already handled by the type-application rule, which is lexical already. Untouched.
- **The 120 lint, compression** — later rulings, unchanged.
