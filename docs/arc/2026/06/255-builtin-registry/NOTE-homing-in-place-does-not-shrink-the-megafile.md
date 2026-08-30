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

## The ceiling, so the next decision is made against a bound

`src/runtime.rs` at HEAD, decomposed (`awk` over top-level items):

```
                                  lines    share
  171 × eval_* / step_* bodies   10,638     31%   ← P6-c's addressable surface
    4 × dispatch_*                1,728      5%   ← incl. dispatch_keyword_head_value (1,539)
  250 × other top-level fns       9,627     28%
  non-fn (impl/struct/static)    12,213     36%
                                 ------
                                 34,206
```

**Everything P6-c could ever move is 12,366 lines — 36% of the file.** A total victory, every
remaining verb body relocated, leaves `runtime.rs` at **~21,800 lines**. It would still be the
second-largest file in the tree.

The other giants, for scale (`>3,000` lines, `src/` + `crates/`, 203,694 total):

```
  34,206  src/runtime.rs          7,250  src/types.rs
  22,505  src/check.rs            5,018  src/edn/render.rs
  10,123  src/rete/kernel/tests.rs  3,209  src/closure_extract.rs
```

Two files hold **56,711 lines — 28% of the tree.**

## The question for the builder — NOT drawn, NOT decided

If the near-term enemy is the megafile, three things are true at once and they pull apart:

1. P6-c's remaining ~106 verbs, homed at the current in-place rate, will move runtime.rs by
   roughly **−450 lines**. That is not a megafile campaign; it is a correctness campaign that
   happens to touch the megafile.
2. The in-place ruling is *correct on its own terms* — the helper-visibility cost is real. Reversing
   it wholesale would trade 12,000 lines of megafile for a dozen widened helpers and a lot of churn.
3. The lever that actually worked was **HOME-8's shape: move a whole namespace's bodies at once**,
   where the helpers travel WITH the verbs instead of being left behind. `:wat::holon::` went out as
   a unit for −6,001. The remaining population may or may not contain another such unit.

So the question is not "should P6-c continue" — its correctness yield alone justifies it. The
question is whether a **separate, differently-shaped strike** should run against the megafile:
one that asks *"which cluster of runtime.rs bodies shares enough private helpers to move as a
unit?"* rather than *"which verb is next in the ledger?"*

**That is a measurement nobody has taken, and it is the one that would tell us whether the megafile
has another HOME-8 in it or only a long tail.** It is cheap — cluster the 171 bodies by which
module-private helpers they call, and read the sizes.

⛔ Not drawn. The builder rules whether that measurement happens before, after, or instead of the
remaining waves.

---

`DERIVAMVS NE MENTIAMVR.`
