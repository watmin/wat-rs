# NOTE — `unquote`/`unquote-splicing` are PUNCTUATION, not verbs

> ⛔ **THE TITLE USED TO END "and the registry should not hold them". THAT HALF IS RETRACTED,
> 2026-09-02.** See the AMENDED section at the foot: the measurement stands, the conclusion drawn
> from it did not.

> Drawing 1a-γ-ii — the last two of the homoiconic eight — the shape refused to fit any of the four
> the campaign has established. **Measured, the reason is that they are not forms at all.**

## The measurement

```
(:wat::core::unquote 1)            outside a quasiquote  →  #wat.runtime/UnknownFunction
(:wat::core::unquote-splicing 1)   outside a quasiquote  →  #wat.runtime/UnknownFunction

(:wat::core::quasiquote (:foo (:wat::core::unquote (:wat::i64::+ 1 2))))   →  (:foo 3)
```

**They are not callable.** They have no dispatch arm, no standalone meaning, and the runtime says
they do not exist. Inside a `quasiquote` template they work — because they are *its grammar*.

★ That is the same relationship `->` has to `fn`, or `<-` to a param spec, or `:-` to a type
application. **No punctuation is registered** — measured: zero registry rows whose leaf is an
operator.

## Why this matters more than two rows

Registering them would have forced two separate defects, and both were about to be improvised:

**① No role names their processors.** Each is consumed by exactly two walkers, and neither is a
dispatch door:

```
macros/expand.rs   walk_template / flatten_template_children   at EXPAND time
runtime.rs         walk_quasiquote                              at RUNTIME, from eval_quasiquote
```

`SpecialFormRole` has `Check`/`Eval`/`Tail`/`Declare`. None names *"recognized inside another form's
walker."* A fifth role would have been minted for two rows that should not exist.

**② `role = eval` cannot stack.** `walk_quasiquote` serves all three FQDNs — `quasiquote`, `unquote`,
`unquote-splicing` — and the shim is keyed on the fn identifier
(`[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]`). Registering both would hit that
wall immediately, and the tempting fix — three identical delegates wrapping one walker — would be
three lies about how the substrate dispatches.

⚠ **Both defects dissolve the moment the right question is asked.** Neither is a problem to solve;
they are the shape telling us the rows are wrong.

## The recommendation

**Do not register them.** Instead:

1. **`quasiquote`'s `@syntax` should show them**, since they are its grammar. It currently reads
   `(:wat::core::quasiquote <template>)` and does not mention unquoting at all — the one place a
   reader would look to learn the form's shape is silent about the only thing that makes it useful.
2. **`special_forms.rs`'s two rows for them are wrong in KIND, not merely unregistered.** They die
   with that table at Phase 4a and need no replacement.
3. The campaign's remaining-18 count becomes **16 rows and 2 non-rows**, and the distinction should
   be recorded rather than left as an unexplained shortfall.

## ⛔ AND A SEPARATE FINDING, measured in passing

`(:wat::core::unquote 1)` outside a quasiquote **type-checks clean** — it falls into `check.rs`'s
shared silent-accept arm (*"declaration forms, not value-producing expressions"*) — and then fails at
runtime with `UnknownFunction`.

★ **The checker accepts a form the runtime says does not exist.** That is the same class as this
arc's founding target (*a nonexistent `:wat::` verb type-checks clean*) and the same class as the
`def`-vs-siblings diagnostic gap already recorded: eight declaration forms in expression position
raise `UnknownFunction` while `def` raises a named error. **A registry that knows `quasiquote`'s
grammar could refuse an unquote outside it, at check time, by name.**

Not this stone's scope; named so it is not rediscovered.

## ⬜ What this NOTE does NOT decide

Whether the two rows are simply dropped from the worklist, or whether `quasiquote`'s `@syntax` gains
the grammar in the same motion. Both are small; the second is the one that leaves the substrate
better documented than it found it. **The builder's call.**


---

## ⛔⛔ AMENDED 2026-09-02 — THE RECOMMENDATION ABOVE IS WRONG, AND THE ARC ALREADY KNEW

> **Builder:** *"the registry must be the authority for what exists and how its used... is it a flaw
> that we cannot call unquote outside of a quasiquote?... is getting unknown function honest?"*

Two questions, two answers, and the second overturns this NOTE's recommendation.

**Is the restriction a flaw? No.** `unquote` outside a template has nothing to substitute into. Every
lisp in the family refuses it. The BEHAVIOUR is correct.

**Is `UnknownFunction` honest? NO — and that is the whole defect.** `:wat::core::unquote` *exists*:
it is in `special_forms.rs`, the checker knows it, two walkers recognise it, the language is
unusable without it. Answering *"unknown function"* is wrong twice over — it is not unknown, and it
is not a function. **The registry returning `None` for a name that exists is the registry lying,
which is the exact thing this arc exists to stop.**

★★★ **So my reasoning above was from DISPATCHABILITY, and that is the wrong test.** The registry's
job is not *"things that dispatch"* — the RULING says it is *"what you query to know what exists…
the properties these names have."* `unquote` exists, and it has a property: **it is legal only
inside `:wat::core::quasiquote`.** That is a fact the registry should hold, not a reason to omit it.

## ★★★ And this arc PLANNED this cure, then deferred it until the registry existed

`[[NOTE-declaration-position-class-guard]]`, dated **2026-06-24**, marked *"LEGIT DEFERRAL to
255-wrap"* on the explicit grounds that *"the proper cure depends on the registry that 255 is
building."* It names the missing property outright:

> *"`SpecialForm` is necessary, not sufficient: `if`/`let`/`do`/`match`/`fn`/`quote` are ALSO
> SpecialForms and are legal at eval. The missing property is finer — **position-class**."*

Its steps are already written: **(1)** declare it once per row, **(2)** query it at the eval seam
before the giant match, **(3)** retire the hand-rolled per-keyword arms — and it names the
anti-pattern to refuse: *"a `const DECLARATION_FORMS: &[&str]` in `runtime.rs` is the
hand-maintained-index-that-drifts."*

**The registry now exists. The deferral's condition is met.**

⚠ `[[feedback_the_refutation_i_brought_was_already_in_the_document]]` — I wrote a recommendation
against a cure this arc had already designed, in a NOTE sitting in the same directory, and found it
only because the builder pushed back on the diagnostic.

## What `unquote` adds to that plan — a THIRD class

The 2026-06-24 note names two: `Declaration` (never legal at eval) and `Expression` (legal at eval).
`unquote`/`unquote-splicing` are neither — they are legal **only inside a `quasiquote` template**.

```
Declaration   def · defmacro · defstruct · …      never at eval          → DeclarationInExpressionPosition
Expression    if · let · do · match · fn · quote  legal anywhere         → dispatches
Template      unquote · unquote-splicing          only inside quasiquote → currently UnknownFunction
```

★ And the same axis explains a defect already on record: **`def` raises a named
`DeclarationInExpressionPosition` while its eight siblings raise `UnknownFunction`** — because two
hand-rolled arms exist and the rest were never added, which is precisely the drift the 2026-06-24
note refused to grow. **One property, declared once per row, answers all three classes and retires
both hand-rolled arms.**

## What the earlier sections still establish

The measurement is unchanged and load-bearing: these two are **not callable**, have **no dispatch
arm**, and are recognised only inside two walkers. That is exactly why their position-class is
`Template` — and it is still true that **no role in `SpecialFormRole` names a walker**, so the
question of what `role` they carry remains open even after `@Position` lands.
