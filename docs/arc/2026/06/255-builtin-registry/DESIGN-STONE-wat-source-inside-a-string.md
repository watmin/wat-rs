# DESIGN — wat source inside a string literal: the population the keyword rules cannot see

DRAWN 2026-08-26 against `f77e155eb`. Prerequisite for **Stone C** (the retirement).

## The builder's framing, which corrects mine

> *"wat-fix is meant to provide utilities to enable bespoke programs to satisfy some codemod."*

I had asked *"should wat-fix reach inside a literal?"* as though wat-fix needed a new capability.
It does not. **wat-fix is the utility layer; each codemod is a bespoke program composed from it.**
This stone is such a program, and the question is only whether the utilities compose to reach the
population — measured below, they do.

## ★ THE CLAIM I MADE THAT WAS WRONG, AND THE MEASUREMENT THAT KILLED IT

I told the builder *"wat-fix can't reach inside a literal."* **False, and measured false:**

```
a string node, tests/value/wat_eval_result.wat:24
    :value "(:wat::core::i64::+ 1"      the CONTENT, unquoted — 21 chars
    :col 20  :end-col 43                the SPAN — 23 chars
                                        23 − 21 = 2 = the two quotes
```

The fact base **does** expose string nodes, with content and span. `read-string` is available inside
a codemod (13 uses in `wat/fix.wat`). The edit tuple is `(offset, old-text, new-text)` — **arbitrary
text**, so quotes are perfectly expressible. Nothing structural was in the way.

What was actually in the way: every recorded rule matches **keyword leaves** and emits an *unquoted*
replacement into a span that *includes* quotes — a two-character geometry mismatch.
`rename-core-string-to-string.wat`'s header documents that hazard as a reason to **exclude** strings,
and **I read "excluded" as "impossible."** A hazard note is not an impossibility proof.

## Why this must exist before Stone C

Stone C retires `:wat::core::{i64,f64}::*`. The keyword-leaf codemod (Stone B) correctly does not
touch string literals. So every `.wat` string that carries wat SOURCE naming an old spelling
**survives B and breaks at C** — at runtime, when something calls `read-string` on it.

## THE THREE POPULATIONS — one question, three different answers

| # | what | disposition |
|---|---|---|
| 1 | **wat source parsed at runtime** — the argument of `:wat::core::read-string` / `:wat::eval-edn!` / `:wat::eval-ast!`, or a `:source` kwarg | **MIGRATE** — or it breaks at C |
| 2 | **codemod & grep RULE literals** naming the old spelling *as the thing being matched* | **NEVER** |
| 3 | **prose, comments, diagnostics text** | **NEVER** |

Population 2 is the sharp one. `wat-scripts/fixes/rete-where-per-type-spelling.wat` carries **15**
such literals, `strip-useless-mains.wat` **1**, and `wat-scripts/grep/core-numerics-ops.wat` **4**
(my own census, written today). **A codemod corpus that rewrites its own rule literals starts
hunting for the new name and every recorded migration silently stops matching.** That is the
self-reference hazard, and it is why a blanket string rule is the wrong shape — not a rule with a
carve-out, a rule that never had the reach.

## The discriminator is STRUCTURAL, and I proved that the hard way

Is this string in **argument position of a wat-source-consuming verb?** The fact base answers it:
`Node` carries **`parent`** and **`index`**, so it is a two-hop join — the string's parent's
index-0 child is `Named` with the consuming verb's name.

⚠ **I tried to size population 1 with a regex and got 2**, because `(:wat::core::read-string "…")`
puts a paren between the verb and the string and my pattern demanded adjacency. **I committed the
exact defect this stone exists to fix, inside the act of arguing against it.** The number is
unknown until the encoded question below is run; nothing in this design quotes one.

## ★ THE DESIGN — claim the WHOLE literal, never an inner offset

The obvious approach is to `read-string` the content, rewrite the inner tree, and splice at inner
offsets. **Do not.** Inner spans are offsets into the *parsed content*, while the file holds the
*source text* — and the two diverge the moment the literal contains an escape (`\"`, `\n`). Offset
arithmetic across that boundary is a bug generator.

Instead: **take the literal's own source text, quotes and escapes included, substitute within it,
and claim the whole thing.**

```
old-text  =  fix-text-span-text(span)         the literal AS WRITTEN, including its quotes
new-text  =  same text, prefix substituted
edit      =  (fix-text-offset-of(span), old-text, new-text)
```

No inner offsets exist, so **escapes are irrelevant to the geometry** — they ride along inside the
text on both sides. And `fix-text-apply` **verifies old-text against the source before splicing**
and RAISES on disagreement, naming the offset and the claim (shipped 2026-08-25). A rule that gets
this wrong fails loudly instead of corrupting a literal.

**The one carve-out that stays a STOP, not a guess:** if the literal's content contains an escaped
quote `\"`, the embedded source has a *nested* string, and an occurrence inside THAT is population 2
or 3 wearing population 1's clothes. Report those; do not rewrite them.

## The stone, in two parts

**Part 1 — the encoded question** `wat-scripts/grep/wat-source-in-a-string.wat`. A string node whose
parent's index-0 child names a wat-source-consuming verb, or which follows a `:source` kwarg.
Reports file/line, the content, and whether the literal carries an escape. **This runs first and its
output is the population.** It joins the corpus at `wat-scripts/grep/` as a permanent question.

**Part 2 — the bespoke program** `wat-scripts/fixes/numerics-inside-wat-source-strings.wat`, built
from wat-fix's utilities (`fix-text-span-text`, `fix-text-offset-of`, `fix-text-apply`), rewriting
exactly the population Part 1 names.

**Part 3 — the negative control, as a GATE and not a hope.** After applying, the 20 population-2
literals in `wat-scripts/fixes/` and `wat-scripts/grep/` must be **byte-identical**. Prove it with
`git diff --stat` over those paths reading zero. A migration that quietly rewrote a codemod's own
matcher would still pass every other check in this design.

## The four questions

- **Obvious?** YES — a string that gets parsed as wat is wat, and must migrate with the language.
- **Simple?** YES — one substitution on one literal's own text; no inner offsets, no re-rendering.
- **Honest?** YES — the whole-literal claim is verified before splice, and the population is derived
  structurally rather than by a pattern that cannot tell a call from a comment.
- **Good UX?** YES — after C, a stale spelling inside a fixture is a *loud* failure at migration
  time rather than a silent one at runtime.

## Out of scope, affirmatively cut

- **`.rs` string literals** carrying wat source — a different corpus with a different tool, and the
  `no_inlined_wat_in_tests` lint already governs that ground. Not this stone.
- **Re-rendering via `read-string`** — rejected above, on the escape-geometry argument.
- **A general "rewrite inside any string" rule** — rejected: population 2 makes blanket reach a
  self-reference hazard.
