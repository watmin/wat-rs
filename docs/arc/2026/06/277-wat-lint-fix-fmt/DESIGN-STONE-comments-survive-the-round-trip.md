# DESIGN — STONE: comments survive the round trip

## WHY, and the builder's own words closing the loop

> *"didn't you just convince me to go build out tooling for comment preservation?.... why are we
> losing those?.. that was our first move when pivoted to this arc?"*

Right. The first stone (`9a16b68e6`) built `lex_with_comments` and **stopped one line short of the
tree, on purpose**:

```
crates/wat-reader/src/lexer.rs:331
    let (tokens, _comments) = lex_with_comments(src, file)?;
                  ^^^^^^^^^ captured, then DROPPED
parser.rs:240,255            let tokens = lex(src, file_arc)?;    ← parser never sees them
```

The stone's scope note said why: *"ATTACHMENT IS DELIBERATELY NOT DONE — which node owns a comment
is POLICY."* **The builder has now ruled the policy**, and this stone spends it:

> *"whatever is expressed above stays... whatever is expressed within gets shifted"*

## MEASURED — what a round trip loses TODAY

`wat-scripts/scratch-pad/277-what-does-the-reader-lose.wat` (committed; the disconfirming probe,
run before this design was written). Each row is `in` → `(ast->source (read-string in))`:

```
3.0 · 42 · "hi" · :wat::core::defn · [x <- :wat::core::i64]     round-trip EXACTLY
(a  b) → (a b)   ·  (a\n  b) → (a b)      whitespace — the formatter's JOB, not a loss
3.00 → 3.0       ·  1e3 → 1000.0          literal SPELLING normalised; value identical
;; a comment\n(a b) → (a b)               ⛔ LOST
(a b) ;; trailing   → (a b)               ⛔ LOST
```

**Exactly one thing is lost: comments.** Everything else round-trips or is whitespace we are
deliberately deciding. That is the whole gap, and it is this stone.

## ⛔ THE ONE CONTRACT DECISION — comments travel BESIDE the tree, NEVER in it

**No `WatAST::Comment` variant.** Reasons, in order of force:

1. **A comment is not a form.** The checker, the type checker, and the macro expander must never
   see one. A variant puts comments into `ast->children` and every consumer inherits them.
2. **The blast radius is the whole tree.** As of `f0f2e5a` today, `WatAST`'s `PartialEq` is
   exhaustive over the variant set — a 15th variant is a compile error in *at least five* matches
   in `ast.rs` alone, plus every `match` on `WatAST` across `src/`.
3. **The sibling shape is already proven in this arc.** `lex_with_comments` sits beside an
   UNCHANGED `lex` that delegates to it. Copy that exactly one layer up.

So: `parse_all_with_comments(src, file) -> Result<(Vec<WatAST>, Vec<Comment>), ParseError>`, with
`parse_all_with_file` unchanged and delegating.

## THE PLACEMENT RULE — the builder's policy, made computable

Spans became honest this morning (`Span::eq` compares `file`/`line`/`col`/`end`), so placement is
arithmetic on spans, not judgement:

```
a comment whose span is BEFORE the next form, on its own line   →  emitted above that form,
                                                                    at that form's indent
a comment whose span is on the SAME LINE as, and after, a form  →  emitted trailing that form
a comment whose span is CONTAINED within a form's extent        →  travels with that form's contents
```

★ **A line comment PINS A NEWLINE after itself** — it must, or whatever follows on the joined line
is commented out. That is a derived constraint, not a policy choice, and it is the one hard rule the
emitter cannot violate.

## THE ACCEPTANCE — idempotence over (forms, comments), NOT byte identity

Byte identity is the wrong bar and would fail correctly: the printer normalises `(a  b)` → `(a b)`,
which is the entire point of a formatter. The right property is a **fixpoint**:

```
parse_with_comments( print( parse_with_comments(src) ) )  ==  parse_with_comments(src)
```
— same forms, same comment TEXTS, same ORDER. That is exactly the property a canonical formatter
needs, and it is checkable on any corpus file.

## FILES

```
crates/wat-reader/src/parser.rs   ADD parse_all_with_comments beside an unchanged parse_all_with_file
crates/wat-reader/src/lexer.rs    UNCHANGED — lex_with_comments already returns what is needed
src/edn/render.rs:775             ADD a comment-interleaving printer beside write_wat_source
```

## OUT OF SCOPE — affirmatively cut, and where each one goes

- **The wat-level verb.** No new intrinsic, no registry row. wat-fmt is written in wat and will need
  `read-string` to yield comments — that is **the NEXT stone in 277**, and it is bounded here by
  name: *"the wat level can read a comment"*. This stone proves the substrate in Rust first, because
  one contract decision per stone and the Rust half is provable alone.
- **Attachment as ownership.** No comment becomes a child of a node. Placement is computed from
  spans at print time; nothing is stored.
- **Literal-spelling normalisation** (`3.00`→`3.0`, `1e3`→`1000.0`). Real, measured, and NOT this
  stone's business — it is a separate ruling the builder has been asked for and has not given.
  **Change nothing about it.**
- **Comment REFLOW** — moving a comment to a different line than the author put it on. Out. "Above
  stays, within shifts" moves comments with their form's indentation, never across lines.

## ⚠ AND A SIDE FINDING, NOT THIS STONE'S WORK

`"A"` came back **UNREADABLE** from `read-string` in the probe above. A unicode escape inside a
string literal does not read. Unrelated to formatting; not chased; recorded so it is not lost.
