# DESIGN — STONE: `if`'s measurement rides, and `cond`'s clauses align

> **Builder, 2026-09-06:** *"if's truthy measurement should be on the same line"* · *"i think we
> should do the same cond"* · then, after the disk was read: *"yeah -- let's get if handled
> correctly... cond just needs alignment.. ya"*

Two rules the builder ruled two stones back, unbuilt since. **Two new files and nothing else** —
the arc's acceptance criterion, proven five times.

## THE DEFECT — both forms fall through to `siblings.wat`

There is no `if.wat` and no `cond.wat`; `wat-scripts/fmt/rules/` holds twelve files and neither is
among them. The generic one-child-per-line rule takes both:

```
TODAY                                    RULED
(:wat::core::if                          (:wat::core::if (:wat::i64::> x 0)
  (:wat::i64::> x 0)          ⛔ the        (:wat::i64::+ x 1)
  (:wat::i64::+ x 1)             measure    (:wat::i64::- x 1))
  (:wat::i64::- x 1))            breaks off

(:wat::core::cond                        (:wat::core::cond
  ((:wat::i64::= k 0)                      ((:wat::i64::= k 0) "zero")
    "zero")                   ⛔ the        ((:wat::i64::= k 1) "one")
  ((:wat::i64::= k 1)            body       (:else              "many"))
    "one")                       leaves
  (:else "many"))                its test
```

⚠ **And the current cond output is already INCONSISTENT with itself:** `(:else "many")` stays on one
line — the atoms rule catches it, every value being an atom — while `((:wat::i64::= k 0) "zero")`
explodes. Same construct, two shapes, decided by whether the test happens to be a call.

## ★ THIS IS NOT WHAT UNLOCKS ARC 255 — measured, and said plainly

The builder asked whether this is the 255 unlock. **It is not.** Of the 614 doc `@example` lines:

```
containing :wat::core::if      5
containing :wat::core::cond    1
containing :wat::core::match  17   (already ruled and built)
                             ───
                             6 of 614 touched by this stone — under 1%
```

`[[PARKED-the-migration-waits-on-wat-fmt]]` states 255's order in its own words, and **step 1 is
already met**:

```
1  wat-fmt exists (arc 277)                   ← MET: 614 examples, 0 over 120, idempotent
2  DocSpecialForm gets a metadata-map reader   ← THE UNLOCK — 87 of 576 rows cannot take the
                                                 new form; 52 carry @alias, 36 @syntax, and
                                                 #wat.doc/Alias is designed but NOT WRITABLE
3  the @-form ratchet, shrink-only, day one
4  the sweep — ONE pass, all 576
```

Plus, from the 294 seam: `crates/wat-doc/src/print.rs` still holds no formatter call — only
`use std::fmt::Write`.

★ **Where this stone DOES pay is the corpus**, which is the other thing wat-fmt has been walking
toward: **1822 `if` and 51 `cond` sites**, 118 `if` in the five real files alone.

## THE RULES

**`if`** — the measurement rides the head line; each branch takes its own line one level in. Three
children, and only the first rides.

**`cond`** — one clause per line; within a clause the body **rides its test**; and the bodies are
**ALIGNED across the clauses of one cond**, which is the builder's ruling.

## ⚠ THE ALIGNMENT NEEDS THE FIT TEST, AND THE MEASUREMENT SAYS WHY

Corpus cond clause tests: **34 clauses, min 22, median 41, max 91 characters** — and the spread is
wide *within a single cond*. Aligning bodies pads the short clauses toward the widest test, which is
the same mechanism that produced 240-character padding runs in
`[[DESIGN-STONE-the-table-rule-claimed-a-definition]]`.

**So alignment is subject to the fit test `table.wat` already carries:** a cond whose aligned form
exceeds 120 does not align — each clause formats by its own rules. That is not a re-litigation of the
builder's ruling; it is the builder's own `120` arbitrating between two of their rulings, and
`atoms.wat` and `table.wat` both already work exactly this way.

## THE CONTRACT DECISION

**A NEW STYLE RULE IS A NEW FILE AND NOTHING ELSE.** `wat/fmt.wat` is not edited, no existing rule
file is edited, and no new fact record is minted. If either rule cannot be expressed without touching
the emitter, that is the finding and the stone stops.

## ⛔ AND THIS STONE DELIBERATELY MOVES THE FORMATTER'S OUTPUT

Every stone since the table fix has demanded byte-identity. **This one must not** — it changes
layout on purpose, at 118 `if` sites in the five real files. The acceptance is therefore a **width
non-regression against a captured baseline**, not identity:

```
deporder  over120=0 worst=102     grep  over120=0 worst=98     io  over120=0 worst=104
spawn     over120=9 worst=174     fmt   over120=2 worst=147
```

A row demanding identity here would be a row the stone cannot pass — the failure mode this arc has
now produced twice. `[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## WHAT THIS STONE IS NOT

- **NOT `match`.** Already ruled and built (`match.wat`); the scrutinee already rides.
- **NOT defect E**, still the builder's call, and still a width cost.
- **NOT the 255 unlock**, which is step 2 above and is named here so it is not lost behind this.

## FILES

```
wat-scripts/fmt/rules/if.wat      NEW
wat-scripts/fmt/rules/cond.wat    NEW
wat-scripts/fmt/run-all.wat       + two load-file! lines (a driver must load a rule to see it)
wat-scripts/fmt/fixtures/         the shapes, including a cond whose aligned form overflows
```
