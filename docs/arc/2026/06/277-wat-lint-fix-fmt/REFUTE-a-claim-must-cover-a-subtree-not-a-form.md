# REFUTE — the rule set is NOT idempotent. `Claim` covers a form; the fight is one level below it.

**The floor is GREEN** (`5179/5179`, clippy 0) and the previous refutation's fixes all hold. This is
a different defect, found by a probe I wrote to test the acceptance's own claim.

## HOW IT WAS FOUND — the acceptance asserted something about a rule nobody had written

The SCORE said: *"A later `let`/`match` rule asserts its own Claim; R11 is not edited."* True as far as
it goes — but it is a claim about a rule that did not exist. So I wrote the third rule myself
(`wat-scripts/fmt/rules/let.wat`), to the shape the builder ruled mid-session:

```
(:wat::core::let           ;; open the block  — NOTHING rides the head line
  [y (:wat::core::+ x 1)]  ;; one binder per line
  y)                       ;; body after binders
```

**Adding it as a NEW FILE worked** — no edit to `fmt.wat`, `defn.wat` or `siblings.wat`, and the
layout came out exactly as ruled. **And then:**

```
FORMS=1 COMMENTS=0 IDEMPOTENT=false
```

## ⛔ THE ARM — both passes, verbatim

```
PASS 1                                PASS 2
(:wat::core::let                      (:wat::core::let
  [y (:wat::core::+ x 1)                [y
   z (:wat::core::+ x 2)]                 (:wat::core::+ x 1)
  (:wat::core::+ y z))                   z
                                          (:wat::core::+ x 2)]
                                        (:wat::core::+ y z))
```

Pass 1 is correct. **Pass 2 breaks each binder's VALUE onto its own line**, and pass 3 would differ
again. `wat fmt --check` (which is `fmt(x) == x`) can never go green on this.

## THE MECHANISM — precise, not theorised

`Claim {form}` is asserted on the `let` NODE. R11 fires on any form whose children sit on different
lines, gated by `(:wat::rete::not (:wat::fmt::Claim (?p <- :form)))`.

**The binding VECTOR is a different node, and nothing claims it.** After pass 1 puts `y …` and
`z …` on two lines, the vector's own children are on different lines — so on pass 2 R11 fires **on
the vector** and breaks every child of it, undoing exactly what R3 laid out.

⭐ **This is the DESIGN's predicted collapse, one level lower than the fix reached.**

> *"It must be a fallback consulted when head dispatch misses, **never a competing rule** — otherwise
> it races every specific rule and the exclusivity argument collapses."*

The first refutation moved R11 from an exclusion list to a `Claim` gate — correct, and it fixed the
race at the claimed node. **The race simply moved into the claimed node's children.** A specific rule
lays out a whole SUBTREE; a claim over a single node protects only its top.

## THE FIX — claim the SUBTREE, and it is stratifiable

A node is off-limits to the default rule if **any ancestor is claimed**, not just itself:

```
ClaimedUnder {node}  :-  Claim {form}                          … the claimed form itself
ClaimedUnder {node}  :-  ClaimedUnder {p} AND Node {node, parent: p}   … and everything beneath it

R11  :when [ … (:wat::rete::not (:wat::fmt::ClaimedUnder (?p <- :node))) … ]
```

**No stratification problem.** `ClaimedUnder` is recursive over ITSELF, which forward chaining does
natively — this is not the refused shape from `[[NOTE-width-is-a-fact-not-a-rule]]`, where a rule
AGGREGATED over its own output. There is no aggregate here, only transitive closure, and R11 reads
`ClaimedUnder` without producing it. ⚠ **Predicted, not measured** — the engine gets the last word,
and if it refuses, that refusal is the finding.

★ It also states the right principle: **a rule that lays out a form owns that form's whole extent.**
The default rule handles what no rule has claimed — which is what "fallback" meant all along.

## ⚠ SECOND FINDING — R1's indents are hardcoded and R11's are derived

```
wat-scripts/fmt/rules/defn.wat      :indent 2 · 2 · 2 · 3          ← absolute constants
wat-scripts/fmt/rules/siblings.wat  :indent (i64::+ ?pc 1 …)       ← derived from the parent's span
```

`Break {id, indent}` is an **absolute column**. R1's constants are correct **only because a top-level
`defn` sits at column 0** — they are latent-wrong for any nested form. My own first draft of R3
hardcoded `2` and produced a `let` binding vector at column 2 when it belonged at 4; deriving it from
`?pc` fixed it in one run.

**The DESIGN's contract line is the cause** — *"`indent` — its column, in spaces"* does not say
absolute or relative, and the two rule files answered differently. Whichever wins, **say it in the
DESIGN**, because every future rule inherits the ambiguity.

## ⚠⚠ AND THE DEEPER QUESTION THIS RAISES — is an absolute column the right contract at all?

A rule computes `indent` from `?pc`, **the parent's column in the CURRENT SOURCE**. But formatting
MOVES forms. So every rule's output depends on a coordinate the previous pass may have changed —
which is a standing invitation to exactly the non-idempotence above, even after `ClaimedUnder` lands.

The alternative: **a rule asserts only WHERE a line begins, and the emitter computes the column from
its own descent** (`parent's emitted indent + 2`). Then indent is structural, no rule can disagree
with another about it, and a form's layout cannot depend on where it happened to sit before.

**This is a DESIGN question, not a defect to patch, and it is the builder's.** Recorded here because
the probe made it visible and it will decide how every later rule is written.

## WHAT STANDS — the previous refutation's fixes are all verified

- `:wat::core::ReadWithCommentsOutcome` registered in Rust (`types.rs:1290`), `TypeScheme` in
  `check.rs:19508`. **`src/intrinsic/mod.rs` was not touched — the debt ledger took no new entry.**
- The dangling `@see` is gone; `wat/fmt.wat` no longer defines `Parsed`.
- The hand-list is gone: `siblings.wat` uses `(:wat::rete::not (:wat::fmt::Claim …))` and
  `defn.wat` asserts `Claim`.
- **floor 5179/5179, 0 FAILED, 18 skipped. clippy 0** under `-D warnings --all-targets`.
- **The acceptance holds for a THIRD rule** — `let.wat` is a new file and nothing else, and the
  output moved. That is the extensibility requirement genuinely met.
