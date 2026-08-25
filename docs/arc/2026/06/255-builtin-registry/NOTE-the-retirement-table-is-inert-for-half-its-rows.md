# NOTE — the retirement table is INERT for half its rows

> Surfaced by the rider on the four-homes stone, 2026-08-25. It reported that feeding
> `RETIREMENT_TABLE` is *necessary but not sufficient*. That is right. Its scope claim — *"every one
> of the 25 rows already has a hand-written arm"* — is wrong, and the truth is worse.

## What I measured, and how

The honest instrument is not a grep over the arms; it is **running every entry**. Each of the 35
`retired:` names was invoked in head position against the built binary and the outcome classified:

```
13   RETIREMENT MESSAGE      Char · def-restricted · define · define-dispatch · defn-restricted
                             enum · foldr · option::expect · result::expect · struct
                             struct-restricted · try · runtime::define-alias

13   ⛔ bare UnknownFunction  List/of · char/of · regex::matches? · Uuid/{v4,v5,nil,version,
                             from-string,to-string,rfc4122-variant?} · Record::def · to-struct
                             holon::Record::def

 7   other (not head-callable in this probe)   :None · list · tuple · vec · Ok · Err · Some
```

**Ten of the thirteen inert rows are this stone's own** — appended today, verified present
(`grep -c RetirementEntry` = 35), and producing nothing. **Three were inert before today:**
`:wat::core::Record::def`, `:wat::core::to-struct`, `:wat::holon::Record::def`.

## ★ THE DISCRIMINATOR, and it is a derivation rather than a list

Sort the two columns and the rule falls out:

> **A retirement row fires iff its name is a BARE `:wat::core::<word>`. Every row whose leaf carries
> a `/` or a further `::` is inert.**

Because they take different paths. A bare name is caught by a hand-written arm — `check.rs:955`'s
`if s == ":wat::core::Char"` in `walk_for_bare_primitives`, or one of the arms in `infer_list`'s
`match k` — and those arms call `remedies_for`, which is the only production caller of
`retirement_lookup`. A slash-form or nested name never reaches one; it falls through `infer_list`'s
callee lookup to `check.rs:5628`, which **silently accepts** (its own comment: *"HARVEST (236.2):
silent-by-intent"*), and then dies at runtime as a bare `UnknownFunction` on a path that never
consults the table either.

So the table is not a lookup the substrate performs. **It is a lookup thirteen hand-written arms
perform, and the table is the data they happen to share.** Adding a row without an arm adds nothing.
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]` — with the sharper edge that here the
hand-list is *invisible*, because the table LOOKS like the mechanism.

## ⚠ AND IT IS WHY MY OWN ACCEPTANCE ROW WAS UNMEETABLE

The four-homes brief's row 1b said the old name must be a `MalformedForm` naming its replacement. I
**derived** that bar — I ran `(:wat::core::Char "x")` and read the retirement message off the
screen. What I never checked was **which mechanism produced it.** I measured the output and inferred
the cause, then wrote a bar that "append exactly ten rows" could not reach.

That is a new variant of this session's recurring defect and worth naming separately: not *a bar
written from what I expected* — the command was right and the output was real — but **a bar written
from a correct measurement whose mechanism I assumed.** Deriving the SHAPE is not deriving the CAUSE.
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`

## THE FIX — one door, not thirteen more arms

The shape is already visible in the failure: `remedies_for` is the door, and only hand-written arms
knock. The candidates:

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **(a)** add ten more hand-written arms | NO — the next hand forgets, and the table still is not the mechanism | YES | **NO** — the table keeps looking like a lookup it is not | NO |
| **(b)** consult `retirement_lookup` where `UnknownFunction` is raised | YES | YES — one site | YES — the table becomes the mechanism it appears to be | YES — every row, past and future, fires |
| **(c)** (b) plus the check-time silent-accept at `check.rs:5628` | YES | one more site | YES — and the error arrives at CHECK time, where the working thirteen already deliver it | **YES** — a retired name should not need to be *reached* to be diagnosed |

**(c) is the answer**, and the 13/13/7 census above is its acceptance row: after it, the inert column
is empty.

⚠ **CORRECTED 2026-08-25, same day.** The sentence that stood here called the seven "an artifact of
my probe's shape" and expected them to need a disposition. Measured properly: they produce
`TypeMismatch`, and at least `tuple`'s message already names its own retirement (*"the comma dies in
the reader"*). **They are diagnosed by a third path and are not a gap.** 22 of 35 rows diagnose; 13
do not. I wrote a guess into a note whose whole subject is guessing about mechanism — recorded
rather than silently edited.

⚠ **The wall this needs** is not a count. It is a test that walks `RETIREMENT_TABLE` itself and
asserts every row produces a retirement diagnostic — a gate over the *table*, not over a hand-list
copy of it. Without that, the thirteenth inert row is one commit away from being the fourteenth.

## STATUS

**Not blocking the four-homes rename**, which is complete and correct on every other axis: the ten
verbs answer under their new names, the corpus finder is 190 → 0, the floor is 5056/5056. This is a
substrate defect the rename EXPOSED, three rows of which predate it. Drawn here with its
measurement, for the builder's ruling on when it ships.
