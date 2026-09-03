# DESIGN — STONE: the hand-rolled arms retire, and every unevaluated form gets the named error

> **Builder, 2026-09-02:** *"retire the hand rolled arms.... all nine get the named error...."*
>
> This is **step 3** of `[[NOTE-declaration-position-class-guard]]` (2026-06-24), which deferred the
> cure *"until the registry can carry/answer this."* It can now.

## ★★★ The measurement — one form is guarded, seven are not

Each of these placed in expression position, run:

```
:wat::core::def            →  DeclarationInExpressionPosition   ← the one hand-rolled arm
:wat::core::defmacro       →  UnknownFunction
:wat::core::defenum        →  UnknownFunction
:wat::core::newtype        →  UnknownFunction
:wat::core::typealias      →  UnknownFunction
:wat::core::defalias       →  UnknownFunction
:wat::core::defsurface     →  UnknownFunction
:wat::load-file!           →  UnknownFunction
```

**Seven forms that exist, are registered, and are known to the checker are told they are unknown
functions.** The 2026-06-24 note predicted exactly this and named the cause: two per-keyword arms
were hand-written and *"the rest were simply never added."*

## THE ONE CONTRACT DECISION — the predicate is `@Purity Unevaluated`

Not `@Category`, and **not a hand-list** — the note's own refused anti-pattern (*"a
`const DECLARATION_FORMS: &[&str]` in `runtime.rs` is the hand-maintained-index-that-drifts"*).

```
registry().lookup_entry(head).purity == Unevaluated   ⇒   refuse, named
```

★ Measured: **11 rows declare `Unevaluated`, and they are exactly the forms that must never reach
eval** — verified by the two gates that already key on it
(`every_special_form_carries_check_and_eval_impls`,
`unevaluated_purity_carries_no_route_to_evaluation`). The axis already means *"this form is never
evaluated"*; this stone makes the evaluator say so.

⚠ **`@Category Declaration` is the WRONG key and the `intueri` cast proved it with a witness:**
`:wat::core::use!` is `@Category Declaration` **and** reaches eval and returns `Unit`
(`use_form.rs:76-77`). A category-keyed guard would refuse a form that legally evaluates.

## The seam

`src/runtime.rs:1990` — immediately after the registry-first handler lookup fails, before
`match head {`. An `Unevaluated` row has no handler by construction (that gate proves it), so it
falls past the door today and lands in the `UnknownFunction` fallback. The check goes in that gap,
in **both** doors (`dispatch_keyword_head`, `dispatch_keyword_head_value`).

## ⛔ The message must name the FACT, not the class — because 3 of the 11 are not declarations

The existing Display reads:

> *"`<head>` is a declaration form, not an expression — declaration forms are top-level registration
> forms and cannot appear in expression position"*

**That is false for `load-file!`/`digest-load!`/`signed-load!`.** They are `@Category Splice`: they
register nothing, they replace themselves with another program's forms. Firing a message that calls
them registration forms would contradict the taxonomy minted two stones ago, in the same week.

The message says what the predicate knows:

> *"`<head>` is consumed before evaluation — it is registered or spliced at freeze time and never
> evaluated — so it cannot appear in expression position."*

★ **The variant NAME is kept.** `DeclarationInExpressionPosition` has **20 sites**, two of them
`.wat` corpus fixtures asserting the EDN tag — so a rename is a user-visible surface change and a
corpus migration, for a name that is historically accurate for 8 of 11 and read by nobody who is not
already reading the message. ⚠ Stated, not hidden: the variant name mumbles for the three splices,
and the message is what carries the truth.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`Unevaluated`-keyed guard at the seam, message rewritten** | YES | YES | YES | YES | ✅ **PICKED** |
| add the seven missing arms by hand | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| key on `@Category Declaration` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| keep the message, fire for all 11 | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| rename the variant too | YES | **NO** | YES | — | ⛔ DISQUALIFIED |

- **hand-arms Honest? NO** — it is the drift-prone index the 2026-06-24 note refused by name, and it
  is *how we got here*: two arms written, seven never added.
- **category-keyed Honest? NO** — `use!` is the live counterexample; it would be refused wrongly.
- **keep-the-message Honest? NO** — it would call three splices registration forms.
- **rename Simple? NO** — 20 sites and a `.wat` corpus migration for no reader benefit.

## Blast radius

`src/runtime.rs` (two seam checks, two hand-rolled arms deleted) · `src/value/signal.rs` (the Display
text). Nothing else. No `.wat` corpus change, no registration, no new axis.

## Acceptance — rows chosen to be unfakeable

| what | expected |
|---|---|
| all 11 get the named error | each in expression position → `DeclarationInExpressionPosition`, none → `UnknownFunction` |
| ⛔ the message is true for a SPLICE | `load-file!`'s text does not call it a registration form |
| ⛔ NON-VACUITY — a made-up head still says unknown | `(:wat::core::zorble 1)` → `UnknownFunction`, unchanged |
| ⛔ a legal form still evaluates | `use!`, `set-redef!`, `if`, `let`, `quote` → unchanged behaviour |
| the hand-rolled arms are gone | `grep -c DeclarationInExpressionPosition src/runtime.rs` drops to the seam's own uses |
| ⛔ the guard is derived | no name list anywhere in the new code |
| floor · clippy | green · 0 |

★ **Row three is the one that matters.** A guard that refuses everything would satisfy row one
perfectly. `zorble` must still be an unknown function, because it genuinely is one.

## Out of scope = REJECTED

- **The `@Position` axis.** Killed by the `intueri` cast: two of its three variants are
  `@Purity Unevaluated` already, with two gates enforcing it.
- **`unquote`/`unquote-splicing`'s containment fact.** The one real gap the cast left standing; it
  wants a FIELD naming the enclosing form, not a variant, and it is its own stone.
- **The checker.** These forms still type-check clean in expression position; making the refusal
  static rather than dynamic is a bigger question and this stone does not touch it.
