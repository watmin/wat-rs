# NOTE — `unquote`/`unquote-splicing` are PUNCTUATION, not verbs — and the registry should not hold them

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
