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

## ⛔⛔ RULED, 2026-08-29 — `:- [...]` IS THE ONE FORM. THE OTHER TWO DIE.

> Builder: *"we spent weeks imposing param-spec… param-spec is `:- [...]`. we have heresy in our
> code. this is unacceptable. there is exactly one way to confer a parametric type. it is
> `:- [...]`. all others must die."*

The section that stood here declined to pick a spelling and said "the finding is the disagreement,
not the verdict." **The verdict is now in and it is `:- [...]`.** The design doc's `(Head [type…])`
is retired by the same ruling — one form, no exceptions, in every position and every file.

### ★ THE ROOT CAUSE — and the weeks were not wasted, they answered a different question

Two recorded migrations imposed the param-spec, and **both were sourced from the ANGLE spelling**:

```
wat-scripts/fixes/parametrics-take-a-type-vector.wat  (Stone ②-ii)  Head<args> -> (Head [args])
wat-scripts/fixes/angle-brackets-to-binder.wat        (Stone ③)     Head<args> -> (Head :- [args])
```

Each rewrote `Head<args>`. **A site already written `(Vector :wat::core::i64 1 2 3)` has no angle
brackets, so no codemod ever looked at it.** The work was complete for the question it asked —
*where are the angle brackets* — and that question could not see a second spelling of the same
concept sitting beside it. The 22 surviving unmarked brackets are ②-ii's own output, left behind
when ③ moved the target to `:-`.
`[[feedback_a_census_of_a_name_must_ask_every_rendering]]`

### The population, measured 2026-08-29 at `27923cb2c`

```
.wat corpus     1474 bare keyword  +   23 unmarked bracket   = 1497
.rs doc/@example 211 bare keyword  +    7 unmarked bracket   =  218
                                                        TOTAL ~1715 sites
```
⚠ `:wat::core::fn`'s `[...]` is its PARAMETER LIST, not a param-spec — 1053 sites excluded from the
unmarked count. A census that misses that exclusion overstates the work by 40×.

### The shape this wants — three stones, in this order

1. **The `.wat` corpus codemod.** R21: wat rewriting wat, `wat-scripts/fixes/`, dry-run on a `/tmp`
   copy + `diff`, then apply. Its two siblings above are the shape to copy — and note that
   `positional-to-kwargs.wat` *itself* contains the bare form, so **the fix corpus is in the
   population it fixes.**
2. **The `.rs` doc-comment and `@example` sites.** Not reachable by a `.wat` codemod, and now
   user-facing (P6-a: a fn named by `#[wat_intrinsic]` has published documentation).
3. **The wall, LAST.** `unwrap_type_param_bracket`'s unmarked arm (its own comment already says
   "③ deletes it"), whatever accepts the bare keyword, and — the part that matters most — **the
   diagnostic, which today recommends the bare form to every user who gets it wrong.**

⛔ **Order is load-bearing.** The wall before the sweep turns the whole corpus red; the sweep before
the wall leaves nothing stopping the third spelling from coming back.
