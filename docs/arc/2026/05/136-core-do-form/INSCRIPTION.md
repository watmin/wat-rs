# Arc 136 — `:wat::core::do` form — INSCRIPTION

## How we got here

The crutch surfaced 2026-05-03 mid arc 135 slice 1, when the user
noted:

> *"i think we need a (do ...) form?... [showing the let*-with-
> ((_ :unit)) pattern]... this pattern is a crutch we keep leaning
> on?"*

Every wat test file used the let*-with-anonymous-binding pattern
as a poor-man's `progn` / `begin` / `do`:

```scheme
;; The crutch — let* with anonymous unit bindings
(:wat::core::let*
  (((_ :wat::core::unit) (:wat::test::assert-eq v1 e1))
   ((_ :wat::core::unit) (:wat::test::assert-eq v2 e2)))
  (:wat::test::assert-eq v3 e3))
```

The four questions all degraded against this pattern: `_` LIES
about what's being declared (it's not a binding; it's a sequencing
artifact). Five lines of binding ceremony for what should be three.

Arc 136 minted `:wat::core::do` as the clean replacement.

## The shape we landed on

```scheme
(:wat::core::do form_1 form_2 form_3 ... form_N)
```

- Variadic; one-or-more forms
- Non-final forms are evaluated for side effects; their results
  are discarded; their types are UNCONSTRAINED (Clojure-faithful)
- Final form's value is returned
- Final form's inferred type IS the do's type — substrate infers;
  recipient unification verifies
- No `-> :T` declaration

## The path through the four questions

The DESIGN for arc 136 went through four amendments before
landing. The shape that shipped is NOT the shape we drafted.

### First draft (2026-05-03, Option A — pure macro)

Original DESIGN: a defmacro expanding to let*-with-unit-bindings.
~10 LOC of wat. No substrate change. The four questions all
answered YES; recommendation was "ship Option A first; promote to
substrate special form later if diagnostic friction surfaces."

### Second amendment (2026-05-06 — REQUIRED `-> :T`)

When arc 145 (typed let) opened with REQUIRED `-> :T` on every
value-bearing form, arc 136 inherited the constraint:

```
(:wat::core::do -> :T form_1 ... form_N)
```

The DESIGN was amended forward; Option A-revised threaded `-> :T`
through the macro expansion. The four questions still answered YES
(Honest = YES because macroexpand surfaces the underlying shape).

### Third amendment (2026-05-06 — Honest = NO)

The user pushed back on Option A-revised:

> *"i don't think do is a macro... i think its just a form the
> runtime provides... the exception case would be confusing having
> it be a long let chain"*

The Honest test failed. When type-check fails on
`(:wat::core::do -> :T f_1 f_2 f_3)`, the diagnostic surfaces from
the macroexpanded `let*`-with-anonymous-unit-bindings — a form the
user didn't write. "Macroexpand surfaces the underlying shape if
asked" was papering over the lie; from the caller's perspective,
the diagnostic doesn't tell the truth about what they typed.

Stop at Honest. **Option A-revised dead.**

Locked **Option B** (substrate special form alongside `if` /
`match` / `let` / `let*` / `try`). ~150 LOC across check + runtime
+ special_forms registry + tests. More LOC; same pattern as every
other value-bearing form in the substrate. Diagnostics surface
against what the user wrote.

### Fourth amendment (2026-05-06 — drop `-> :T`)

Arc 145 backed out as foundation-correction-non-shipping (the
typed-let realization: substrate is already type-checking via
inference + recipient unification; REQUIRED `-> :T` provided only
error-locality, not new safety; the print-then-return idiom failed
Good UX). Arc 136 inherited the realization.

The user re-grounded with the Clojure reference:

> *"do is value bearing — same concern as let"*
>
> https://clojuredocs.org/clojure.core/do

Clojure's do discards each non-final's return value; non-final
types are unconstrained. The form's contract is "evaluate
sequence, return last." Non-finals don't owe anything to the
contract.

The DESIGN amended forward one last time:
- Drop `-> :T` slot entirely
- Non-finals: type-checked normally for internal consistency, but
  their types are unconstrained (Clojure-faithful)
- Final form: substrate infers; recipient unifies

Same shape, fewer slots, more ergonomic for the print-then-return
idiom that started this whole arc-145-and-arc-136 conversation.

## What shipped

### Slice 1a — substrate (`ff45f38`)

Substrate special form alongside `if` / `match` / `let` / `let*` /
`try` / `option::expect` / `result::expect`. ~17 min wall-clock
for sonnet's slice 1a; 154 LOC substrate + 289 LOC test file.

Four files:
- `src/check.rs` (+46): `infer_do(args, ...)` — empty form parses
  as MalformedForm; non-finals' types intentionally discarded;
  final's type returned; wired into `infer_list` keyword dispatch
- `src/runtime.rs` (+101): `eval_do` (non-tail) + `eval_do_tail`
  (tail-position twin threading final through `eval_tail`) +
  `step_do` (incremental peel-one-head-per-step); wired into
  `dispatch_keyword_head`, `eval_tail`, `step_form`
- `src/special_forms.rs` (+7): `:wat::core::do` registered with
  `<form>+` variadic sketch (reflectable via arc 144's lookup-form
  trio)
- NEW `tests/wat_arc136_do_form.rs` (+289): 10 tests covering
  empty/single/multi/recipient-unify/non-final-type-unconstrained/
  reflection/tail-call/nested/mixed-with-let*. All passed
  first-run.

Mode A clean. Workspace stayed at 1978 passed / 0 failed.

### Slice 1b — consumer sweep (`f50edf7`)

~45 transforms across 14 files; ~21 min wall-clock for sonnet.

Pure unit-binding chains transformed; mixed sites (real bindings
interspersed with unit-discards) preserved as let*. The grep count
for `((_ :wat::core::unit)` dropped from 197 → 77 (-60.9%, lower
than predicted 70-90% because many files had a single
unit-discard amid real bindings — those legitimately stay mixed).

No latent bugs surfaced. Every transformed site already had
`:unit`-returning non-finals; the migration was clean.

### Arc 153 intersection

After arc 136 slice 1b shipped, arc 153 ran (rename
`:wat::core::unit` → `:wat::core::nil`). Arc 153's sweep migrated
the do form's signatures + return positions: every
`(:wat::core::do ... ())` became `(:wat::core::do ... :wat::core::nil)`,
and every annotation `-> :wat::core::unit` became
`-> :wat::core::nil`. The do form inherits the renamed singleton.

### Slice 2 — closure (this commit)

INSCRIPTION + 058 row + USER-GUIDE entry + WAT-CHEATSHEET entry.
Pre-INSCRIPTION grep mandatory per FM 11 (zero matches verified).

No substrate retirement — the do form has no transitional
scaffolding. It's permanent.

## Why

The crutch advertised bad practices wherever it shows. Wat is a
Lisp; minting a clean `do` form is one line of new vocabulary
that retires hundreds of lines of awkward `let*` boilerplate. The
four questions answer YES at every test site after this arc.

Sequence-of-side-effects-then-return is the daily verb of any
Lisp:

```scheme
(:wat::core::do
  (:println "computing...")
  (:wat::core::+ 1 1))
```

Three forms; the print runs; the sum returns. Nothing more, nothing less.

## The four questions — what landed

Run on the shipped shape (no `-> :T`; non-finals unconstrained;
final's type IS the do's type):

1. **Obvious?** YES. Three forms, no declaration tax. Reads as
   "do these things, return the last."
2. **Simple?** YES. Substrate special form mirrors infer/eval
   patterns of `if`/`let*`/`try` minus the `-> :T` slot. Each
   piece atomic; composition follows the established pattern.
3. **Honest?** YES. Substrate infers final's type from final
   form's value; recipient slot is the contract; substrate
   diagnostics surface against what the user wrote (no
   macroexpansion lies).
4. **Good UX?** YES. The casual print-then-return idiom works
   without ceremony; debug breadcrumbs free.

This is the shape arc 145's failure paid for. REQUIRED `-> :T`
would have failed Good UX; pure macro would have failed Honest.
The substrate special form with no `-> :T` is the shape that
holds all four.

## Cross-references

- **Arc 135** — the cleanup sweep where the user named the crutch
- **Arc 145** — the typed-let arc that backed out as
  foundation-correction-non-shipping; arc 136 inherits the
  realization (`feedback_substrate_already_typed.md`)
- **Arc 153** — unit-to-nil rename; arc 136 returns are
  canonically `:wat::core::nil` post-arc-153
- **Arc 144 slice 2** — special-form registry that arc 136's
  registration plugged into
- **Arc 109 slice 1d** — Pattern 3 walker precedent that arc 153
  used for the unit-name retirement
- `feedback_simple_is_uniform_composition.md` — N identical 1:1
  transforms IS simple (slice 1b sweep applied this)

## Calibration record

- **Arc opened:** 2026-05-03 (mid arc 135 slice 1, when user
  named the crutch)
- **DESIGN amendments:** four (Option A; Option A-revised with
  `-> :T`; Option B with `-> :T`; Option B without `-> :T`)
- **Slice 1a substrate:** ~17 min wall-clock at `ff45f38`
- **Slice 1b consumer sweep:** ~21 min wall-clock at `f50edf7`
- **Arc 153 intersection:** the do form's return positions
  migrated through arc 153's sweep at `fd1b3fe`
- **Slice 2 closure:** this commit
- **Honest deltas:** the DESIGN's four amendments are the cost
  of the path. Each amendment ran the four questions; each lost
  one and pivoted. The shipped shape is the shape all four pass.
  Earlier amendments stay as historical record below the
  "Earlier amendments" header in the DESIGN.

## Status

**Arc 136 closes here.** The do form is permanent vocabulary.
Every existing let*-with-pure-unit-bindings site that fit the
mechanical 1:1 transform now uses `:wat::core::do`; mixed-binding
sites legitimately remain `let*`. The crutch is named in the arc
record as the failure pattern do replaces.

**Arc 109 v1 closure trajectory** advances by another link.

**The Lisp on Rust gains its sequencing form.** With arc 153's
`nil` and arc 136's `do`, two foundational vocabulary marks land
on the same day. The user's frame:

> *"we ride to compaction... i need a lisp on rust to satisfy
> what we're building towards"*

The substrate's vocabulary surface keeps gaining clean Lisp
markers atop Rust enforcement.

---

*the crutch retires. the form is named. the substrate teaches.
forward progress only.*

**PERSEVERARE.**
