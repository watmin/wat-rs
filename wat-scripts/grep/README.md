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
| `can-raise.wat` | which functions contain a call that can panic? | 207 call sites across 121 functions · 2.6s |
| `core-numerics-ops.wat` | how big is the numerics rehome, really? | **1495** keyword leaves · text finds 1571 · *(whole corpus, 1577 files)* |

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

**`core-numerics-ops`** — the arc 255 numerics rehome (`:wat::core::{i64,f64}::*` →
`:wat::{i64,f64}::*`). Text finds **1571**; the migration is **1495**. The 76-site gap is **39
comments** (`;; metadata-of on a rust builtin (:wat::core::i64::+) — RED at HEAD returns None`) and
**42 string literals** (`"(:wat::core::i64::+ 1 2)"` — wat source embedded in test fixtures). The
string literals are the dangerous half: a literal's span covers its surrounding quotes while its
`name` does not, so splicing a replacement into that span corrupts the literal into unquoted keyword
syntax. That is what the KEYWORD-ONLY guard exists for.

There is a second cut this program makes that text cannot make safely at all. `:wat::core::i64`
**without** a trailing `::` is the TYPE — 6,670 `.wat` occurrences bound for arc 251's `wat.type/`,
a different destination in a different arc. `grep -oF ':wat::core::i64'` returns **8111**, one number
covering both populations. The trailing `::` separates them, and the rule's `starts-with?` cannot
match the shorter bare type by construction — but only the structural instrument can then also drop
the comments and strings, and it is the combination that makes the number the migration's size.

⚠ **Written 2026-08-26, mid-migration, because the orchestrator had already briefed the wrong
number.** A text census of 1613 went into a rider's acceptance row as a bar to reconcile against.
The rider was told to explain any difference before applying — which would have made a correct tool
look like a discrepancy. The census that should precede the brief was written after it.

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

Order does not matter — not the rules', and not the conditions'. Measured rather than assumed:
`collect-rules` sorts by name, so `head-position.wat` compiles `:hp::calls-first` BEFORE the
`:hp::head` rule that feeds it, and the answer is identical (159 either way). Within a `:when`,
guards are positionally free — put a `(:wat::rete::where …)` next to the conditions whose variables
it relates, which is where it reads best.

Four things that will otherwise cost a cycle, all measured:

- The vector constructor in a `:then` is **`:wat::rete::core::PersistentVector`**, not core's.
  Core's fails the fence with *"is not total"*.
- A **record** constructor takes kwargs; a **tagged enum variant** takes positions. They look
  identical at the call site.
- **`cond`'s clauses are PARENTHESIZED**: `(cond (test body) … (:else body))`, not Clojure's flat
  `test expr test expr`. The flat form is refused by the purity classifier with
  `'<malformed cond clause>' is not pure` — a shape problem wearing an axis's name, so the message
  sends you to think about purity when the problem is the clause shape.
  (`cond` in a `:then` used to fail in the RHS lowerer; fixed 2026-08-24 by making `:then` an
  expansion boundary. It works now — this line is about the clause shape only.)
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

## Recursion — a rule that feeds itself

`can-raise.wat` asks a question with no bounded shape: *does this function contain, at ANY depth, a
call that can raise?* "At any depth" is transitive closure, and a rete rule computes it by matching
its own output:

```clojure
direct : a node is Under its own parent
step   : if X is Under A, and N's parent is X, then N is Under A     ;; matches its own :then
```

Forward chaining runs that to a fixed point. Verified on `(a (b (c (d))))`: **24** `Under` facts,
exactly the sum over nodes of each node's ancestor count — counted, not eyeballed. Cost on the real
corpus: **2.6s for all 54 stdlib files**, closure included.

Three controls, because a new rule's first answer is never evidence:

```
:ctl::risky   partial call at depth 1           matched
:ctl::deep    same call nested inside two ifs   matched     <- transitivity, not adjacency
:ctl::safe    no partial call at all            silent      <- non-vacuous in both directions
```

And one result checked against the disk: `bracket.wat:338` really is
`(:wat::core::first (:wat::core::ast->children node))`, and the enclosing top-level `defn` at
`:335` really is `:wat::bracket::-type-slot-name` — the exact pair reported.

**Report at the site, not at the container.** The first draft reported each match at the *defn's*
span, which produced one Match per (defn, call) PAIR — the same function repeated with identical
coordinates, reading as several findings when it is one. Reporting at the call site makes every
Match a distinct place and carries the containing function as a capture. Same facts, honest
granularity.

## What is NOT here yet

Transitive depth (`nested more than N deep`), quote-awareness (above), and cross-file joins. The
last wants a different lifetime — wat-grep resets between files BY DESIGN, and a corpus-wide join
would use `with-network` rather than `with-overlay`.

Available and unused so far, all real: `(:wat::rete::exists (Fact (?k <- :k)))`,
`(:wat::rete::not …)`, and the accumulators — `acc::count` / `sum` / `min` / `max` / `mean` /
`distinct` / `all` / `group-by` / `gather-vals`, bound as
`(?n <- (:wat::rete::acc::count) :from (:Fact))`.

## ⛔ A NEW RULE'S FIRST ANSWER IS NEVER EVIDENCE

`defined-twice.wat` returned **0 across all 54 stdlib files** — a completely plausible answer to
"are there duplicate definitions", and one most readers would accept without checking. It was wrong.
A positive-control fixture that defines the same name twice **also** returned 0, which is the only
thing that separated *"nothing matched"* from *"the question was never asked"*.

(The cause was a rete bug — a guard placed before a fact condition matched nothing, silently. Fixed
2026-08-24; guards are positionally free and that is verified on this exact program. The bug is
gone; the discipline it taught is not.)

**So: before believing any count, including zero — especially zero — build a fixture the rule MUST
match and one it must NOT.** Every program here has both.
