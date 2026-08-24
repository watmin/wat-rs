# `wat-scripts/grep/` — a corpus of encoded questions

Each file here is a **wat-grep program**: it declares rules and returns them from `:user::grep`.
wat-grep supplies the fact base per file (`Node` / `Named` / `Span` / `Source`), owns the one query,
and prints whatever `Match` facts the rules assert. The programs interpret; the tool does not.

```
printf '["path" "path" …]\n' | wat --grep wat-scripts/grep/<program>.wat
```

## Why these exist

The builder's thesis (2026-08-24): *a corpus of proven "encoded thoughts for problem resolution"
buys rete fluency by proximity, not by training.* These are the first entries. Each one is a
question this codebase actually needed answered, written down in the form that answers it.

Every program here is chosen for the same property: **text gives a different answer, and the
difference is not noise.**

| program | the question | measured, `wat/**/*.wat` (54 files) |
|---|---|---|
| `head-position.wat` | where is a partial verb actually CALLED? | 159 calls · text finds 164 occurrences |
| `unwrap-of-lookup.wat` | where does a missing key become a crash? | 30 sites · a one-line regex finds 7, one a comment |
| `bare-variant-constructors.wat` | how big is the Option/Result migration, really? | 211 constructor calls · text finds 212 |

## What the differences were

**`head-position`** — the 5 occurrences that are not calls: three comments, one comment quoting a
panic message, and `wat/fix.wat:1118` — a STRING LITERAL, `":wat::core::first"`, being compared
against an `ast-name`. The codemod's own matcher, reading a name as data. Text renders it
identically to a call.

**`unwrap-of-lookup`** — the pattern is `(Option/expect (HashMap/get m k) "msg")`, three levels of
parentage, not adjacency. **24 of the 30 span multiple lines**, so a line-based regex cannot see
them at all; and its 7th "hit" is `wat/grep.wat:16`, a comment. Misses 80% of the population and
reports a comment as a site.

**`bare-variant-constructors`** — `Some` and `Ok` agree exactly with the text count. `Err` differs
by one: `wat/spawn.wat:603`, a comment describing a MATCH ARM — `((:wat::core::Err _died) acc)` — a
pattern, not a constructor. The one place in 212 where the two instruments disagree, and structure
is right.

## The shape to copy

1. Declare intermediate fact types for the concepts your question needs (`IsHead`, `Unwrap`).
2. Write one rule per concept. A rule that recognises a shape asserts a fact naming it.
3. Write one rule that joins those concepts to `Span` and `Source` and asserts a `:wat::grep::Match`.
4. Return every rule from `:user::grep`.

Two things that will otherwise cost a cycle, both measured:

- The vector constructor in a `:then` is **`:wat::rete::core::PersistentVector`**, not core's.
  Core's fails the fence with *"is not total"*.
- A **record** constructor takes kwargs; a **tagged enum variant** takes positions. They look
  identical at the call site.

## What is NOT here yet

Negation (`a form with no docstring`), transitive depth (`nested more than N deep`), and cross-file
joins. The first two want rules this corpus has not written; the third wants a different lifetime —
wat-grep resets between files BY DESIGN, and a corpus-wide join would use `with-network` rather than
`with-overlay`.
