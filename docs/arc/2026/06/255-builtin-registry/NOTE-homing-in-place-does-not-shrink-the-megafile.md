# NOTE — homing IN PLACE does not shrink the megafile. Six waves, 42 verbs, −182 lines.

> **Measured 2026-08-29, orchestrator-side, while W6's rider was in the field. Every number below
> comes from `git show <commit>:src/runtime.rs | wc -l` or an `awk` decomposition of the file at
> HEAD `15d85ca05`. Nothing here is read from a prior doc.**
>
> ⛔ **This is a finding, not a ruling.** It does not say P6-c is wrong. It says P6-c is buying
> something OTHER than what the ROAD's step 1 heading implies, and the builder should know the
> difference before the next wave is drawn.

## The measurement that started it

The builder, 2026-08-29: *"the megafiles are the prime enemy for the near term."* So: is the
campaign shrinking them?

`src/runtime.rs`, at each P6-c commit:

```
P6-c-1   edb06a099   34,388
W1       4e75cecd0   34,307     −81
W2       fa0713722   34,153    −154
W3       2b5a95b6c   34,161      +8     ← up
W4       694ce713e   34,191     +30     ← up
W5a      e01428497   34,165     −26
W5b      2bc1135aa   34,189     +24     ← up
W5c      71f5baaff   34,194      +5     ← up
HEAD     15d85ca05   34,206
```

**Six waves. 42 verbs homed. `34,388 → 34,206 = −182 lines`, and four of the six went UP.**
That is **−4.3 lines per verb**.

## What DID cut the megafile — and the difference is one word

```
HOME-8 strike 1   d43f75887   40,441 → 38,948    −1,493
HOME-8 strike 2   fb0cdb192   38,948 → 34,440    −4,508   (95 verbs)
HOME-13           fcf3c6f57   34,497 → 34,252      −245   (44 arms; dispatch_substrate_impl → 7 lines)
```

HOME-8 strike 2's own commit message: *"runtime.rs is 6,001 lines lighter … the megafile finally has
a line through it that the compiler enforces."* Its diff on runtime.rs was **754 insertions,
5,262 deletions**.

W3's diff on the same file, for ten verbs: **342 insertions, 334 deletions.** Net +8.

**HOME-8 MOVED THE BODIES. P6-c FROM W3 ONWARD REGISTERS THEM IN PLACE.** Both are called "homing"
in the campaign's vocabulary. Only one of them is a megafile lever.

## And the departure was RULED, deliberately — it just was not measured

This is not drift. W3's commit message says it outright:

> *"All ten stayed IN runtime.rs rather than moving to their own file — a deliberate departure from
> W1/W2, justified per-verb: eight depend on module-private helpers (`lookup_form`, `Binding`,
> `function_to_signature_ast`, `require_ast_children`, `peel_param_spec`) and moving them would bump
> visibility on a dozen helpers for no behavioural gain. P6-c-1's two verbs set that precedent."*

The justification is sound **on its own axis** — moving a body that leans on a dozen module-private
helpers costs a dozen `pub(crate)` widenings and buys no behaviour. What was never weighed is that
the axis it was decided on (behavioural gain) is not the axis the ROAD's step 1 is about
(the megafile). The ruling answered the question it was asked.
`[[feedback_a_probe_answers_the_question_you_asked_not_the_one_you_meant]]`

Note also that homing ADDS lines on purpose: a homed verb gains a user-facing doc block
(`@arg`/`@ret`/`@example`/purity), which is the point of P6-a's rule. A wave that deletes a 3-line
dispatch arm and an inline arity guard while adding a 25-line doc block nets positive, and **that is
the mechanism, not a defect.**

## What P6-c IS buying — this is real and should not be dismissed

The waves are not idle. Registering a verb makes three dormant consistency gates fire on it, and the
screams have been substantial:

- **W3 alone exposed SEVEN doc lies**, including `holon_type_ast_to_wat_type_form` — a function
  named in `runtime.rs`, `check.rs`, and three test files, that **has never existed anywhere**.
- **`apply` reachability** — O-iv-a stopped 331 registered verbs being called "unknown function".
- **Real arity published** — 59 hand-rolled arity guards retired across the campaign; verbs that
  reported `:arity -1` now report the truth through `metadata-of`.
- **Purity rulings with disk citations**, per verb, where previously 221 sat unreviewed.

**None of that is megafile work, and all of it is worth having.** The finding is that the campaign
has two products and only one of them is on the ROAD's step-1 line.

## ⛔ THE CEILING — AND A CORRECTION I OWE, MADE THE SAME HOUR

**First published here, then measured properly and found TOO GENEROUS. The corrected number is
below; the wrong one is left visible because the mechanism is the lesson.**

**What I first wrote:** *"171 × `eval_*`/`step_*` bodies = 10,638 lines = 31% = P6-c's addressable
surface,"* and from it, *"everything P6-c could ever move is 12,366 lines — 36% of the file."*

**What is actually true.** I used a NAME PATTERN as a proxy for a POPULATION and never validated
it. Of those 171 fns, only **48** are the body of a verb still in the dispatch. The other **123**
are a mix I had folded in without looking: already-homed verbs whose bodies stayed behind
(`eval_bigint_arith`, `eval_body_of`, `eval_edn_validate` — registered elsewhere, calling back in)
and pure evaluator machinery that was never a verb at all (`eval_do_tail`, `eval_and_tail`,
`eval_call_to_defclause`). `eval_*` is what the evaluator names its own internals, not a verb marker.
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]` — re-read this morning, recurred
by lunchtime.

**The measured surface, resolved arm-by-arm instead of by name:**

```
  90  FQDN arms left in dispatch_keyword_head_value  (63 :wat::core:: + 9 bare core + 18 other)
  81  resolved to a target (62 single-expression, 19 block-form)
  61  whose target fn is DEFINED IN runtime.rs        ~2,669 lines   ← the remaining waves' mass
  20  already delegating out (collection::transform 13 · rete::kernel 4 · eval 1 · function 1 · io 1)
```

```
                                       lines   share of runtime.rs
  61 resident bodies of REMAINING arms  2,669       7.8%
  15 verbs homed IN PLACE (attribute
     on the fn, body never moved)         844       2.5%
  dispatch_keyword_head_value           1,539       4.5%   (collapses to ~7 lines when the
                                        -----               last arm goes — HOME-13's precedent)
  realistic P6-c-addressable total     ~5,050        15%
                                                    ----
  runtime.rs                           34,206
```

**So the ceiling is ~15%, not 36%.** A total P6-c victory — every remaining verb homed AND its body
moved AND the dispatch collapsed — leaves `runtime.rs` at roughly **29,150 lines**. It stays the
largest file in the tree by a wide margin (`check.rs` is 22,505).

That is the number the megafile decision should be made against, and it is far worse for the
campaign-as-megafile-lever than the one I published an hour earlier. The remaining **19,000+ lines**
of `runtime.rs` are the evaluator itself — `Environment`, `SymbolTable`, `apply_function`,
defclause parsing and selection, pattern matching, quasiquote walking, shutdown/fault machinery —
and **no amount of verb homing touches any of it.**

★ Only **15 of 422** registrations live in `runtime.rs`. The campaign has been moving code out for
real; it is simply that the mass it can still reach is small, and the file's bulk was never verbs.

## The question for the builder — NOT drawn, NOT decided

The measurement the first draft of this NOTE asked for has now been TAKEN (it was cheap, and the
rider was in the field). It changes the question rather than answering it:

1. **`:wat::core::` IS the last big unit — 63 of the 90 remaining arms, plus 9 bare.** That is
   HOME-8 scale by verb count (HOME-8 moved 95). So the "is there another HOME-8 in here" question
   has a YES on the count axis. **W6 is the first wave into exactly this namespace.**
2. **But the LINE mass is not there.** Those 90 arms' resident bodies are ~2,669 lines, against
   HOME-8's −6,001. `:wat::holon::` was 95 verbs of dense algebra; `:wat::core::` is 90 arms of
   mostly-small readers, and a third of them already delegate out. Same verb count, a quarter of
   the mass.
3. **So the ceiling binds before the shape does.** Even run at HOME-8's shape — bodies moved,
   helpers travelling with them, dispatch collapsed — `runtime.rs` lands near **29,150**. The
   megafile is not made of verbs. It is made of the evaluator.

**Therefore the question is no longer "which cluster moves as a unit."** It is:

> ⬜ **If `runtime.rs` cannot go below ~29,000 by homing, what is the ACTUAL decomposition of the
> evaluator?** `Environment` + `SymbolTable`, `apply_function` + the call path, defclause
> parse/select, pattern matching, quasiquote, shutdown/fault — those are the 19,000+ lines, and
> they are six or seven genuinely different concerns sharing one file. That is a `partire` question
> (does this file have one reason to change, or several?), not a homing question, and it is the
> only lever left that can move the number the builder cares about.

And the honest counterweight, so this is not read as "stop":

- P6-c should still finish. Its yield is **correctness, not lines** — doc lies, arity truth, `apply`
  reachability, purity rulings. Seven lies in W3 alone. Killing the dispatch match also removes the
  last unregistered surface, which is the thesis of arc 255 (the blanket-accept at
  `resolve/walk.rs:268`) and a hard prerequisite for ROAD steps 2–4.
- Nothing here says the waves are wasted. It says **the megafile campaign and the registry campaign
  are two campaigns wearing one name**, and only the second one is being fought.

⛔ Not drawn. Which of the two the builder wants swung next is the builder's ruling.

---

`DERIVAMVS NE MENTIAMVR.`
