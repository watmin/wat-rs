# REFUTE — the subtree claim over-corrected. The default rule can no longer reach real code.

**Floor GREEN** (`5179/5179`, clippy 0). Idempotence is FIXED and every fix from the previous two
refutations holds. This is the cost of the fix, and it was named honestly in the SCORE:

> *"A claimed form's **whole extent** is off-limits to R11. A half-broken `match` inside a `defn` is
> no longer reformatted by R11; that is R4's. … This is the principle the refute asked for, not a
> regression to hide."*

Correct on both counts — I asked for subtree claiming, and it is not hidden. **But the principle I
asked for was wrong, and here is the measurement that shows it.**

## ⛔ THE CONTROLLED PAIR — same form, same rules, only the nesting differs

`(:wat::core::do …)` has NO specific rule. R11, the default, is the only rule that could lay it out.

```
INSIDE A defn                                          AT TOP LEVEL
(:wat::core::defn :fix::u                              (:wat::core::do
  [x <- :wat::core::i64]                                 (:wat::kernel::println "a")
  -> :wat::core::i64                                     (:wat::kernel::println "b")
  (:wat::core::do (println "a") (println "b") (+ x 1)))  (:wat::kernel::println "c"))
        ↑ 90 columns, one line. R11 INERT.                      ↑ R11 works.
```

**All real code lives inside a top-level definition.** So the default rule — the thing that is
supposed to handle every form nobody has ruled — fires only where nothing real is written.

★ And this is not hypothetical for this arc: the builder's dominating requirement is *"i will never
have all the rules that matter."* **There will ALWAYS be unruled forms, and they will always be
inside a `defn`.** A default that cannot reach them is a default in name only.

## THE ASYMMETRY, and it is the clue to the fix

`ClaimedUnder` gates only R11. **Specific rules are ungated and reach everywhere** — R4 (`match`,
written this session as a new file) lays out a `match` INSIDE a claimed `defn` perfectly:

```
(:wat::core::defn :fix::hb
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::match x
    (n n)
    (_ 0)))          IDEMPOTENT=true
```

So the tree is not the problem. **The gate is too wide.**

## ✅ THE FIX — claim the forms whose CHILDREN you position, not the subtree

The original defect (`let`'s binder values re-breaking on pass 2) was never about descendants in
general. It was one specific node: **R3 positioned the binding VECTOR's children, and nothing claimed
the vector.** The cure is granular, not transitive:

> **A rule claims every form whose children's line-positions it decides.** Nothing more.

```
R1 claims  the defn node  AND the arg-spec vector   (it breaks children of both)
R3 claims  the let node   AND the binding vector    (same shape)
R4 claims  the match node                            (it breaks only the match's own children)
R11 gated on  Claim {?p}  DIRECTLY — not ClaimedUnder
```

Both properties hold at once, and neither is traded for the other:

- **Idempotence survives.** The binding vector IS claimed by R3, so R11 never re-breaks it — which
  was the entire original defect.
- **Reach is restored.** A `do` inside a `defn` body is NOT claimed — R1 never positioned the `do`'s
  children — so R11 fires on it at any depth.

⚠ **Predicted, not measured.** The reasoning is above; the engine and the fixtures get the last word.
The controlled pair is the test to re-run: the `do` must break inside a `defn` **and** the `let`
binders must stay pairs across two passes. **If both cannot hold, that is the finding**, and it means
`Break {id, indent}` is the wrong contract — see below.

★ **`ClaimedUnder` is not wasted work** — the transitive closure is real, it stratifies, and it may
be the right instrument for something else. It is simply the wrong gate for the default rule.

## ⚠⚠ THE DESIGN QUESTION UNDERNEATH, STILL THE BUILDER'S AND STILL UNANSWERED

Three refutations have now all been the same shape: **two rules disagreeing about one node's
position.** The exclusion list, then the node claim, now the subtree claim. Each fix moved the
collision rather than removing it.

That pattern is the symptom of a contract in which **collisions are expressible at all.** A rule
asserts `Break {id, indent}` — an ABSOLUTE column, derived from `parent.col` in the CURRENT source —
so two rules can name different columns for one node, and a form's layout depends on where it
happened to sit before the run.

The alternative, recorded in `[[REFUTE-a-claim-must-cover-a-subtree-not-a-form]]` and still
unanswered: **a rule asserts only WHERE a line begins; the EMITTER computes the column from its own
descent.** Then no two rules can disagree about a column because no rule names one, and indent cannot
drift because it is never read from the input.

**This is the builder's call.** I am not patching it, and I will keep reporting the collisions until
it is made or ruled unnecessary.

## WHAT STANDS

- **Idempotence is genuinely fixed** for the `let` case — pass 1 == pass 2, binders stay pairs.
- **R1's indents are derived now**, not hardcoded — `parent.col + 1` / `+ 2`, matching R11 and R3.
- **`ClaimedUnder` stratifies.** Recursion without an aggregate, exactly as predicted; the engine
  accepted it, which also confirms `[[NOTE-width-is-a-fact-not-a-rule]]`'s diagnosis was about the
  AGGREGATE and not about recursion.
- **The acceptance holds for a FOURTH rule.** `rules/match.wat` (R4) is a new file and nothing else,
  and it produced the builder's ruled shape first run.
- **floor 5179/5179, 0 FAILED, 18 skipped · clippy 0.**

## ★ AND A LANGUAGE FACT, ASKED AND MEASURED

> **Builder:** *"i think underscore is illegal, right?..."*

**No — `_` is legal, and it is the sanctioned way to close a match.** 105 uses across 59 corpus
files, and the checker is BUILT on it: `src/check.rs:6251` (`MatchShape::Open(_) => wildcard_seen`)
drives exhaustiveness from the wildcard, and the non-exhaustive error at `:6257` literally
recommends *"add a fallback `_` arm."*
