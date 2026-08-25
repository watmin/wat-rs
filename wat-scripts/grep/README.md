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
| `defined-twice.wat` | does one file define the same name twice? | 0 real · 2 false positives, both macro templates |

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
4. `(:wat::rete::collect-rules :your-ns)` — never a hand-list.

The intermediate fact types are what make a rule read like a sentence: `IsHead`, `Unwrap`,
`ArgIsList` are not machinery, they are the NOUNS, and the join is the "whose". A text search has no
way to name a middle step, which is why it can only ever ask about adjacency.

**`:user::grep` returns `(:wat::rete::collect-rules :ns)`, never a hand-written vector.** It
reflects the symbol table for every zero-arg fn in the namespace whose return type is
`:wat::rete::Rule` — the marker `defrule` plants. A hand-list is a second copy of the same
information, and the rule you add next month is the one that silently does not run.

Order does not matter, and this was measured rather than assumed: `collect-rules` sorts by name, so
`head-position.wat` compiles `:hp::calls-first` BEFORE the `:hp::head` rule that feeds it, and the
answer is identical (159 either way). Forward chaining does not care about declaration order.

Four things that will otherwise cost a cycle, all measured:

- The vector constructor in a `:then` is **`:wat::rete::core::PersistentVector`**, not core's.
  Core's fails the fence with *"is not total"*.
- A **record** constructor takes kwargs; a **tagged enum variant** takes positions. They look
  identical at the call site.
- **`cond` does not lower in a `:then`; `if` does.** Measured, and the reason is NOT the one this
  README first gave. `cond` compiles fine in a `where` on the LHS (verified — the rule fires); it is
  the RHS lowerer that has no `cond` arm (`src/rete/expr_ir.rs:314`), so a clause reaches generic
  call lowering and is reported as `malformed :wat::rete::lower form: call head must be a keyword`.
  `--check` passes it either way, so the failure arrives at `compile-all`.
  Filed: `~/work/NOTE-rete-cond-lowers-on-the-lhs-but-not-the-rhs.md`.
  ⚠ Note the clause shape: wat's `cond` is `(cond (test body) … (:else body))` — PARENTHESIZED
  clauses, not Clojure's flat `test expr test expr`. The flat form is refused earlier, by the purity
  classifier, with `'<malformed cond clause>' is not pure` — a shape problem wearing an axis's name.
- **`or` is binary.** Three alternatives need `(or A (or B C))`.

### On collapsing rules — measured, and rejected

`bare-variant-constructors.wat` has three near-identical rules. They collapse into one, using an
`or` guard and a nested `if` in the RHS to pick the replacement — verified, same 211 matches. **It
is not kept.** Three rules that each read as one sentence beat one rule carrying two nested
conditionals, and a conditional inside a `:then` puts branching where a fact assertion belongs.
Fewer lines was the only thing it won.

## The self-join, and the blind spot it exposed

`defined-twice.wat` teaches the pattern no text search can reach even in principle: the same
condition written twice with a shared variable, comparing two occurrences TO EACH OTHER. Every
other program here asks "is there a node like X"; this one asks "are there TWO that agree."

Two things it taught that are worth more than its answer:

**The ordering guard.** A self-join matches both directions — (a,b) and (b,a) — so every duplicate
reports twice, mirrored. `(i64::< ?a ?b)` keeps one of each. Without it the count is exactly
doubled, which is the kind of wrong number that looks plausible enough to publish.

**The fact base has no notion of quote-as-data.** The program's only 2 hits across the stdlib are
both inside a QUASIQUOTE — two mutually-exclusive arms of an `if` in a macro template, of which
exactly one is ever emitted. `facts-of` walks every node including template interiors;
`src/rete/purity.rs:1257` by contrast has an explicit arm skipping `quote`/`quasiquote` because
they are data. **A rule about "definitions" currently cannot tell a definition from a template for
one.** A `:wat::grep::Quoted [id]` fact would close it. Structure beats text, and structure has its
own way of being wrong; naming that is the job.

## What is NOT here yet

Transitive depth (`nested more than N deep`), quote-awareness (above), and cross-file joins. The
last wants a different lifetime — wat-grep resets between files BY DESIGN, and a corpus-wide join
would use `with-network` rather than `with-overlay`.

Available and unused so far, all real: `(:wat::rete::exists (Fact (?k <- :k)))`,
`(:wat::rete::not …)`, and the accumulators — `acc::count` / `sum` / `min` / `max` / `mean` /
`distinct` / `all` / `group-by` / `gather-vals`, bound as
`(?n <- (:wat::rete::acc::count) :from (:Fact))`.

## ⛔ A GUARD MUST BE THE LAST THING IN A `:when`

Measured 2026-08-24, and it cost a rule that read correctly and answered zero: **a
`(:wat::rete::where …)` followed by a FACT condition silently matches nothing.** No error, exit 0,
empty result. The same rule with the guard moved after the last fact condition matches correctly —
11 vs 0 on identical input.

Every program in this directory places its guards last. That was luck before it was knowledge:
`rules-corpus-03-source-to-facts.wat` was written that way and everything copied its shape.

Filed for the rete agent:
`~/work/NOTE-rete-a-where-before-a-fact-condition-silently-matches-nothing.md`.
