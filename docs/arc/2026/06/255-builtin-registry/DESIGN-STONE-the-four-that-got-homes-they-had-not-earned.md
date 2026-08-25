# DESIGN — STONE: the four that got homes they had not earned

> Home #4 phase 2 (`56eb6ab3a`) killed `string_ops.rs` by giving every family it held a file. Four of
> those families got a **home** without getting a **right name**. The builder caught it:
> *"i don't know if i fully agree to them… this was more than i asked for."*

## The ruling

> *"i wanted to move uuid to `:wat::uuid::Uuid` for class and `:wat::uuid::*` for fns"* ·
> *"`/of` are meant to die as the ctor for a type is just itself invoked on its argument"* ·
> *"i say these are our next targets — `:wat::regex::*` and `src/regex/*.rs` feels fine?… we can grow
> it as we go"*

## ★ ALL FOUR ARE ONE CLASS — a name migration where the HANDLER DOES NOT CHANGE

That is the finding, and it is what makes this one stone instead of four. Every target already has a
working implementation; what is wrong is only what it is called.

| # | today | target | verbs | corpus sites |
|---|---|---|---|---|
| 1 | `:wat::core::Uuid/*` | `:wat::uuid::*` (+ `:wat::uuid::Uuid` the type) | 7 | **101** |
| 2 | `:wat::core::regex::matches?` | `:wat::regex::matches?` | 1 | 13 |
| 3 | `:wat::core::List/of` | `:wat::core::List` | 1 | 62 |
| 4 | `:wat::core::char/of` | `:wat::core::char` | 1 | 17 |

**193 sites, 10 verbs, zero behaviour change.** Re-register each intrinsic under its new name,
codemod the corpus, delete the old registration — the mechanism stone E proved, now with a working
rules-based codemod (`wat-scripts/fixes/rename-core-string-to-string.wat` is the shape).

## Why `/of` is FINISHING a migration, not starting one

This is not taste. **Every other collection type is already its own constructor**, measured:

```
(:wat::core::PersistentVector 1 2 3)   → #wat.core/PersistentVector [1 2 3]
(:wat::core::HashSet …)                → #{"a" "b"}
(:wat::core::Vector …) (:wat::core::Tuple …) (:wat::core::HashMap …) (:wat::core::PersistentMap …)

(:wat::core::List 1 2 3)               → UnknownFunction     ← the holdout
(:wat::core::char "x")                 → UnknownFunction     ← the holdout
```

Each working constructor is a thin match arm delegating to `crate::collection::eval::eval_*_ctor`.
`:wat::core::List` **already exists as a TYPE** (`types.rs`, `check.rs`, and
`runtime.rs:9266`'s `declared_type_name`) — it simply has no constructor arm. The body it would call
already exists: `eval_list_of`, now at `src/intrinsic/list.rs:33`.

So `List/of → List` is not new machinery. It is registering an existing handler under the name the
language already uses for every one of its siblings. `keyword/of` is already a kept gravestone;
these are the last two.

★ **And the type-name/head-position question answers itself** — `PersistentVector` is simultaneously
a type in annotation position and a constructor in head position, and has been for the whole corpus.
`List` and `char` inherit that, unchanged.

## ⚠ TWO CASINGS FOR ONE THING — resolve it here, since we are touching it

`src/` holds **both** `:wat::core::Char/of` and `:wat::core::char/of`. That is the casing question
from `109/NOTE-the-type-names-go-short-and-lowercase.md` showing up as an actual duplicate rather
than a preference. Whichever survives, only one should — and this stone is the moment it costs
nothing to settle.

## The file homes

```
src/uuid/       the namespace home; src/intrinsic/uuid.rs stays the registry home   (E's two-home split)
src/regex/      builder-approved: "src/regex/*.rs feels fine — we can grow it as we go"
```

`List` and `char` are single verbs and stay where phase 2 put them. ⚠ **One open question worth a
sentence, not a stone:** every other collection constructor's body lives in `crate::collection::eval`.
`List`'s lives in `src/intrinsic/list.rs`. Consistency argues for `collection::eval`; the registry
argues for where it is. Name the choice in the brief rather than letting a rider guess.

## ACCEPTANCE

1. **Each of the 10 verbs answers `metadata-of` under its NEW name, and the old name is
   `UnknownFunction`.** Both directions per verb — a rename that leaves the old name working is a
   bridge, and R9 is about bridges nobody demolishes.
2. **`(:wat::core::List 1 2 3)` and `(:wat::core::char "x")` evaluate** — the holdout closes.
3. **193 corpus sites migrated by codemod**, dry-run diffed byte-level first. No hand edits (R21).
4. **Idempotent AS A QUERY** — re-run the finder, get 0. The property the rules-based mechanism has
   and the char-walk never did.
5. **The doctest count is STILL 5.** Ten verbs change names; their `@example` lines change with
   them. If the count rises, an example is naming a verb that no longer exists.
6. **Only one of `Char`/`char` remains.**
7. Floor green accounted BY NAME; clippy 0.

## OUT OF SCOPE — affirmatively cut

- **A regex ENGINE.** `matches?` is one predicate with 13 call sites. `src/regex/` is a home to grow
  into, not a promise to fill this stone.
- **`:wat::core::Uuid` the TYPE's own spelling beyond the namespace move.** `:wat::uuid::Uuid` is the
  target; whether it later becomes `wat.type/uuid` belongs to the type-names note, not here.
- **The other 33 loose files at `src/` root.** This stone is four families that a carve relocated
  without asking. The rest of that population is its own arc.
