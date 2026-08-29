# NOTE — a parametric literal has THREE spellings, and no authority names all three

> Builder, 2026-08-29, reading a rider's scratch edit: *"the only allowed form for a parametric
> vector is `(:wat::core::Vector :- [:wat::core::i64] 1 2 3)` … right?… this diff /must be/
> illegal, right?"*
>
> **It is not illegal. It is one of three legal spellings, it is used at 1,308 corpus sites, and the
> substrate's own error message recommends it.** Measured on the shipped binary. **No row, nothing
> drawn** — this records the contradiction.

## Measured — all three check clean and produce the same value

```wat
(:wat::core::Vector :wat::core::i64 1 2 3)          ;=> [1 2 3]   BARE type keyword
(:wat::core::Vector :- [:wat::core::i64] 1 2 3)     ;=> [1 2 3]   the `:-` binder
(:wat::core::Vector [:wat::core::i64] 1 2 3)        ;=> [1 2 3]   unmarked bracket
(:wat::core::Vector 1 2 3)                          ;=> MalformedForm — no spec at all is illegal
```

The bare keyword is **consumed as the declared element type**, not passed through as data — proven
by substituting a non-type: `(:wat::core::Vector :not-a-type 1 2 3)` raises
`":wat::core::vec: parameter #2 expects :not-a-type; got :wat::core::i64"`, i.e. the elements are
checked *against* it.

## Corpus counts, across every `.wat` in the tree

```
(Head :- [T …])   2870      the `:-` binder
(Head T …)        1308      the BARE keyword          ← the one the builder expected to be illegal
(Head [T …])        22      the unmarked bracket
```

## ⛔ THE CONTRADICTION — three authorities, three answers, no two agreeing

**1. The design doc names ONE form, and it is the rarest.**
`DESIGN-STONE-all-parametrics-take-a-type-vector.md` (this arc, opened by builder ruling
2026-08-20): *"**Every parametric — type or literal — is `(Head [type…])`.** One shape, nesting
uniformly."* That is the **unmarked bracket — 22 uses**, and it is the one the code says is slated
for deletion.

**2. The implementation's comment names TWO, and neither is the common one.**
`unwrap_type_param_bracket` (`src/check.rs:12152`): *"Arc 109 Stone ②-i-b — BOTH spellings, `[T…]`
and `:- [T…]`."* It goes on to call a form that works on `Tuple` and not on `Vector` *"the
two-ways-to-say-one-thing defect this whole campaign exists to remove"* — and says of the unmarked
arm, *"③ deletes it."* **The bare keyword appears nowhere in that function or its comment.**

**3. The runtime diagnostic names TWO — and recommends the bare form FIRST.**
```
malformed :wat::core::vec form: first argument must be a type keyword (e.g., :i64)
or a `(Head :- [T …])` type form
```
**The unmarked bracket — the design doc's one true shape — is not mentioned.** A user who writes
`(Vector 1 2 3)` is told by the substrate to write the spelling the design never sanctioned.

## Why this is the sharp kind of defect

Nobody wrote a wrong rule. Each authority is locally coherent: the design ruled a shape, ②-i-b added
the `:-` binder and documented the pair it touched, and the diagnostic was written to unblock the
form people actually use. **The defect is that no one of them can be read alone and be right**, and
the one place a user meets the language — the error message — advertises the spelling with no design
behind it. `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`

★ **And it is directly on THE ROAD.** Step 3 is *kill `::` in keywords* and step 5 is *EDN/Clojure-
compliant syntax*. A parametric literal with three spellings is three things to migrate, and the
migration will be written from **one** of these authorities — whichever the author happens to read.
1,308 sites ride on the one that only the error message documents.

## What is NOT claimed here

That the bare form should be removed. It is the second-most-used spelling and may be the right one
to keep — `(Vector :i64 1 2 3)` is arguably the most readable of the three, and `:-` costs two extra
tokens on every literal. **The finding is the disagreement, not the verdict.** Whoever rules this
should decide the spelling first and then make all three authorities say it — the design doc, the
unwrapper's comment, and the diagnostic — because today a reader can consult any one and be wrong.
