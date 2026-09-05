# DESIGN — wat-fmt: the RULE SET is the product, not the formatter

> **Builder, 2026-09-05:** *"we must make it extensible over time... i will never have all the
> rules that matter.. but i will absolutely spot stuff i don't like... and when we have those...
> we fix them and the code fixes itself as we do... that's the most important thing."*

> And earlier the same day, on the model: *"we should be able to slurp in any valid form and return
> its 'as we expect it to be written'"* · *"there's zero reason to bind ourselves to limitations of
> others.. but we should absolutely draw inspiration from them."*

## ⛔ THE REQUIREMENT THAT DOMINATES EVERY OTHER DECISION

**Not "get the rules right." Nobody can.** The requirement is that the COST OF ADDING A RULE stays
near zero forever, and that adding one heals the corpus automatically.

That is not a hope; it is gateable, and this design's acceptance is:

> **A new style rule is a NEW FILE AND NOTHING ELSE.** No engine edit, no match arm, no
> recompilation of a layout algorithm. Proven by adding a rule the engine did not know about when
> it was written, and watching the corpus normalise.

⭐ **This is already the arc's own doctrine.** `[[SELF-FIXING-TOOLCHAIN]]`: *"A tool alone is a
suggestion. A tool PLUS a rule that finds every place the old form survives and rewrites it is a
CURE. You do not hand-migrate; the linter finds them all (including the ones you wrote
yesterday)."* wat-fmt is that doctrine extended from anti-patterns to layout.

## THE MODEL — canonical, not normalise-only. RULED.

The formatter takes **any valid form** and emits **the canonical rendering**. It decides line
breaks; it does not merely fix the indentation of breaks the author chose.

⚠ **This is deliberately NOT cljfmt's model**, and the reason matters. cljfmt normalises only —
it never re-wraps — and that is **downstream of a limitation**: Clojure's reader drops comments, so
cljfmt must work on text and cannot re-emit. Its design is a workaround wearing the clothes of a
philosophy. Draw inspiration from its rule vocabulary; do not inherit its constraint.

★ And the same limitation is why `wat/fix.wat` is span-based today: it never re-emits, so comments
survive by not being touched. **Choosing canonical means choosing to fix the reader instead.**

## ⛔ THE BLOCKER — the reader cannot see comments. First stone, and it is not in the formatter.

Proven by the lexer's own test (`crates/wat-reader/src/lexer.rs:1069`):

```rust
lex_tokens("; a comment\n()")  ==  vec![Token::LParen, Token::RParen]
```

**Comments are discarded at lex time.** No `Comment` token, no `Comment` AST node, nothing
downstream can see them. A canonical reprinter emits from the AST — so **every comment in the
corpus would vanish.**

Nothing about "handle comments gracefully" is designable before the reader can see them. The first
stone is a `Comment` token plus an attachment to the node it belongs beside, and it lives in
`crates/wat-reader/`.

## THE ENGINE ALREADY EXISTS — and it is a better fit than "rules happen to be expressible"

```
wat/lint.wat   695 lines    rule walks, Finding + FixEdit records
wat/fix.wat   1291 lines    parse -> locate via ast-span -> splice ORIGINAL text ->
                            fix-text-apply, with a verify-before-splice check
wat/rete/      defrule :when […] :then […] — homoiconic, expands to a zero-arg defn
src/rete/…     fire_fixpoint_delta — forward chaining to a FIXPOINT, derived facts re-fire
wat/rete/acc.wat  count · sum · min · max · mean · distinct · all · group-by · gather-vals
```

A canonical layout engine's core computation maps onto that directly:

```
a node's rendered WIDTH depends on its children's widths   ->  bottom-up derivation to a FIXPOINT
"does this form fit the budget?"                           ->  acc::sum over the matched child set
"which layout rule applies here?"                          ->  :when, on the head symbol
ORDER and NESTING                                          ->  :parent :index :depth ON THE FACT
```

⚠ **Order was raised as an unknown and it is not one.** Layout order is *data you assert*, not a
property the engine must preserve. Recorded because it was raised: the builder's answer — *"we just
derive the knowledge we need such that we have it when we need it"* — is correct, and width
propagation is a textbook forward-chaining derivation, which is the thing rete is FOR.

## ★ THE ONE STRUCTURAL CONSTRAINT EXTENSIBILITY IMPOSES — exclusivity by shape

Measured: this rete has **no salience, no priority, no agenda** — rules fire when they match. (Found
nothing; not proven absent.) With an open-ended rule set that is a hazard: two layout rules matching
one node, when layout needs exactly ONE decision per node. It bites at rule #15, not rule #2.

**The cure is the rule SHAPE, not an engine feature: a layout rule dispatches on HEAD SYMBOL.** Two
rules for `defn` cannot both exist; a rule for `let` cannot fire on a `defn`. Exclusivity becomes
structural, and no conflict resolution is ever needed.

⚠ The one place ordering is real: the **default** rule for a form no rule names. It must be a
fallback consulted when head dispatch misses, never a competing rule — otherwise it races every
specific rule and the exclusivity argument collapses.

## THE SELF-HEALING LOOP — the acceptance made mechanical

```
1  spot something you do not like
2  write a defrule                       ← a NEW FILE. nothing else.
3  the floor goes RED everywhere the old form lives     ← wat fmt --check over the corpus
4  run wat fmt                           ← the corpus heals
5  the gate is green, and stays green
```

Step 3 is what makes step 1 cheap: **you never have to find the instances.** That is the
SELF-FIXING-TOOLCHAIN's whole claim, and here it is the difference between a style rule and a
style *opinion*.

## SEQUENCING

```
1  THE READER      a Comment token + attachment. Nothing downstream is designable without it.
2  A RETE-SHAPE PROBE  one layout rule (defn — 277's NOTE already specifies it exactly) expressed
                       as a defrule, driven by fixpoint width derivation, over one file.
                       Its job is to fail cheaply if the shape is wrong.
3  THE STYLE TABLE  argued, not guessed. The May scratch draft is superseded: written for a
                    surface that changed, and for a normalise-only model now overruled.
4  THE CORPUS GATE  wat fmt --check joins the floor — at zero, after one reformatting commit.
```

⛔ **Step 2 before step 3, and the reason is this campaign's most expensive repeated lesson:** do
not write the table against a shape nobody has driven. `[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

## WHAT THIS IS NOT

- **Not the May scratch design** (`scratch/2026/05/003-wat-fmt`). Its architecture (own crate, own
  parser, parse→AST→emit) was superseded six weeks later by arc 264's *"never `write-forms` the
  tree — comments die"*, and its premise — *"wat code is HolonAST"* — is false as of arc 294. Its
  ~20 STRUCTURAL rules (indent, parens, line length, symbols, literals, quasiquote) survive and are
  worth mining; its six special-form rules name retired spellings (`let*`, `define`, `lambda`,
  `try`, `expect`, `vec`).
- **Not a second formatter for doc rows.** The doc-row printer is wat-fmt's first real consumer:
  609 examples, median 67 columns, p90 188, max **1515** — all one line, because the `@example`
  grammar forbids breaking them. Canonical layout is what makes them readable, and one canonical
  renderer means the fence and the corpus cannot drift.
