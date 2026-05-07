# Arc 136 — `:wat::core::do` form (sequential side-effect chain)

**Status:** opened 2026-05-03. Arc 135 closed (unblocked).

**Revised 2026-05-06 (FOURTH amendment — drop `-> :T`):** after arc
145's back-out (closed as foundation-correction-non-shipping), the
substrate's existing inference + recipient unification provides
the static check arc 145 was attempting to add via REQUIRED
`-> :T`. Per the user direction:

> *"right now... let and a supposed do form is already implicitly
> typed so the type declaration is just unnecessary noise"*

> *"if we look at the clojure example.. wrap a ret val in a print
> via do would make an annoying as shit ux"*

`(:wat::core::do f1 f2 ... fN)` — no `-> :T` slot. Substrate
infers the do's type from the final form; recipient unification
verifies against whatever consumes the do. Pure Clojure-faithful
shape with the substrate's existing static-check machinery.

The four questions ran on the no-`-> :T` shape (2026-05-06):
1. Obvious? YES — three forms, no declaration tax
2. Simple? YES — substrate special form mirrors infer/eval pattern of `if`/`let*`/`try` minus the `-> :T` slot
3. Honest? YES — substrate infers from final form; recipient slot is the contract
4. Good UX? YES — `(do (println "LOG") (+ 1 1))` reads cleanly; debug breadcrumbs free

REQUIRED `-> :T` failed Good UX (the print-then-return idiom example);
the no-`-> :T` shape passes all four.

**Cross-arc:** arc 145 closed as a non-shipping foundation-correction
(see arc 145 DESIGN); arc 136 inherits the realization.

Per `feedback_inscription_immutable.md` the prior amendments
(below) stay as historical record — the lock on Option B
(substrate special form) STILL HOLDS; only the typed-form
discipline is dropped.

---

# Earlier amendments (historical record)

**Revised 2026-05-06** through three user clarifications:

1. *"do is value bearing, so it should be typed"* — REQUIRES `-> :T`
   at HEAD (matches `let`/`let*`/`if`/`match`/`cond`/
   `Option/expect`/`Result/expect`). Original draft predated arc 145's
   REQUIRED-`-> :T` stance and assumed untyped expansion; dead.

2. *"i don't think do is a macro... i think its just a form the runtime
   provides... the exception case would be confusing having it be a long
   let chain"* — Implementation Option A-revised (pure macro expanding
   to typed `let*`) FAILS Honest. Error messages from the macroexpansion
   would surface against let*'s shape with anonymous unit-bindings — a
   form the user didn't write. Substrate special form locks (Option B);
   the diagnostic stream tells the truth about the source form.

3. *"do is value bearing — same concern as let"* with the Clojure
   reference (`https://clojuredocs.org/clojure.core/do`):
   Non-final forms' return types are UNCONSTRAINED (Clojure-faithful).
   Each non-final is type-checked normally for internal consistency, but
   its resulting type is silently discarded. Only the final form's type
   unifies with `-> :T`. This is MORE permissive than the let*-with-unit-
   bindings crutch, which required each non-final to be `:unit`-typed
   via the binding slot. Existing migration sites are still clean (today's
   crutch sites all have :unit-returning non-finals — otherwise they'd
   already be broken under arc 145).

Q1 (placement) follows arc 145's HEAD position: bindings/non-finals
are setup; the form's contract is the FINAL form; `-> :T` declares
that contract before any forms.

Per `feedback_inscription_immutable.md` the prior shape stays as
historical record. Earlier drafts of this DESIGN locked Option
A-revised (the macro path); those drafts were Honest-wrong per
direction (2). The revised DESIGN below replaces Option A-revised's
lock with Option B's lock; the original "Rejected: Option B" framing
was the misjudgment direction (2) corrected.

## TL;DR

Mint `(:wat::core::do -> :T form1 form2 form3 ...)` as a typed
sequential evaluation form: evaluate forms left-to-right; return the
value of the last; declared return type is `:T`. Replaces the
let*-with-`((_ :wat::core::unit) ...)` crutch that propagates
through every test file.

## Provenance

The crutch surfaced 2026-05-03 mid arc 135 slice 1, when the
user noted:

> i think we need a (do ...) form?... [showing the let*-with-
> ((_ :unit)) pattern]... this pattern is a crutch we keep
> leaning on?...

It IS a crutch. Every wat test file uses the let*-with-anonymous-
binding pattern as a poor-man's `progn` / `begin` / `do`. The
binding name `_` LIES about what's being declared (it's not a
binding; it's a sequencing artifact). The four questions all
degrade against this pattern:

- **Obvious?** No — what does `_` name?
- **Simple?** No — five lines of binding ceremony for what should be three.
- **Honest?** No — `_` pretends to be a binding while actually being syntactic glue.
- **Good UX?** No — readers must mentally translate "unused let* binding" → "side effect sequence."

## The pattern in question

```scheme
;; ❌ Today's crutch — let* with anonymous unit bindings.
;; Post-arc-145, this becomes typed; the noise compounds:
(:wat::core::let* -> :wat::core::unit
  (((_ :wat::core::unit) (:wat::test::assert-eq v1 e1))
   ((_ :wat::core::unit) (:wat::test::assert-eq v2 e2)))
  (:wat::test::assert-eq v3 e3))

;; ✓ With (:wat::core::do -> :T ...) — declared once, three forms, clean intent.
(:wat::core::do -> :wat::core::unit
  (:wat::test::assert-eq v1 e1)
  (:wat::test::assert-eq v2 e2)
  (:wat::test::assert-eq v3 e3))
```

## Naming choice — `do`

Considered:

| Name | Origin | Verdict |
|---|---|---|
| `do` | Clojure | **chosen** — short, modern Lisp idiom |
| `begin` | Scheme | longer; less compact |
| `progn` | Common Lisp | older; "PROGram N" reads cryptic |
| `seq` | various | overloads with sequence types |
| `then` | English | less Lispy |

`(:wat::core::do form1 form2 form3)` reads cleanly. Idiomatic to
Clojure; familiar to anyone reading modern Lisp.

## Semantics

Clojure-faithful sequential evaluation with a typed return contract.

- `(:wat::core::do -> :T)` — zero forms; ill-formed (parse error). A do with a declared `:T` and no body has nothing to produce that value; the form would lie.
- `(:wat::core::do -> :T form1)` — single form; evaluates to form1's value; form1's inferred type unifies with `:T`.
- `(:wat::core::do -> :T form1 form2 ... formN)` — evaluates form1, discards its result, ..., evaluates formN, returns formN's value. formN's inferred type unifies with `:T`.

Type rule:
- **Non-final forms** are type-checked normally for internal consistency (each form must parse + check), but their resulting types are UNCONSTRAINED. The substrate evaluates each non-final and silently discards the result. This matches Clojure's `do` semantics: non-finals are pure side effect; their values are intentionally dropped.
- **Final form** unifies with the declared `-> :T`. Mismatch surfaces a TypeMismatch at the final form's position with the do-form's contract clearly named in the diagnostic.
- **Untyped form** `(:wat::core::do f1 f2)` without `-> :T` is a parse error post-arc-136. Mirrors arc 145's MalformedForm migration-hint shape.
- **`:Any` is forbidden** — the substrate has no wildcard type. `:T` is concrete or parametric.

This is MORE permissive than the let*-with-unit-bindings crutch it replaces. The crutch's `((_ :wat::core::unit) expr)` slot REQUIRED expr to be `:unit`-typed (via the binding's declared type). Existing crutch sites all have :unit-returning non-finals (otherwise they'd already be broken under arc 145's typed let*). Migration to do: clean for all current sites; opens future sites where a non-unit value should just be discarded without binding ceremony.

## Implementation surface — Option B (substrate special form, locked 2026-05-06)

Substrate-level special form alongside `if`/`match`/`cond`/`let`/
`let*`/`try`/`Option/expect`/`Result/expect` — every value-bearing
core form in the substrate is a special form, and `do` joins that
set. ~150 LOC across `src/check.rs` + `src/runtime.rs` +
`src/special_forms.rs` + new `tests/wat_arc136_do_form.rs`.

### Substrate edits (sketch)

- `src/check.rs::infer_do(args, ...)`:
  - args[0] = `->` Symbol (verify; else MalformedForm migration-hint)
  - args[1] = `:T` Keyword (parse via `parse_type_expr`)
  - args[2..N-1] = non-final forms; `infer` each (must internally type-check) but DO NOT unify against anything (type discarded)
  - args[N] = final form; `infer` and unify with declared `:T`; mismatch surfaces TypeMismatch on `:wat::core::do` `body`
  - return declared `:T`

- `src/runtime.rs::eval_do(args, env, ...)`:
  - Skip args[0..2] (`->` + `:T`)
  - Iterate args[2..N-1]; `eval` each; discard each result
  - `eval` args[N]; return its value
  - Tail-call: `eval_do_tail` mirrors but jumps into final form's tail context
  - Incremental step (`step_do`): single-step semantics for the eval-step!
    interpreter; preserve `-> :T` tokens verbatim when rebuilding intermediate forms

- `src/special_forms.rs`:
  - Register `:wat::core::do` with sketch `(-> <T> <form>...)` — variadic non-final + final form positions
  - Reflectable via arc 144's lookup-form trio

### Why substrate special form (the four-questions re-run)

After user direction (2): `do` is a runtime form, not a macro.

- **Obvious?** YES. `(:wat::core::do -> :T f1 f2 f3)` reads cleanly; substrate handles directly; errors say "do form."
- **Simple?** YES. ~150 LOC across the established substrate pattern. Each piece (infer arm, eval arm, registry sketch, test file) is atomic; composition follows the pattern shared with `if`/`match`/`let`/`let*`/`try`/`option::expect`/`result::expect`. More LOC than a macro, but the SAME shape as siblings — uniform composition.
- **Honest?** YES. Errors say "do form" because the form IS "do form." No macroexpansion noise; the diagnostic stream tells the truth about the source form. Substrate-as-teacher doctrine + arc 145's typed-let pattern both align.
- **Good UX?** YES. Diagnostics surface against what the user wrote. Substrate-as-teacher per arc 145 sweep 1b loop: write `-> :T`; iterate per error; converge on honest contract.

### Rejected: Option A-revised (pure macro expanding to typed `let*`)

Earlier drafts of this DESIGN locked Option A-revised: a defmacro
consuming `-> :T` and emitting a typed `let*` with unit-discarded
non-final bindings. ~15 LOC of wat; no substrate change.

**Rejection rationale (Honest):** the macro lies via expansion.
When type-check fails on `(:wat::core::do -> :T f1 f2 f3)`, the
diagnostic surfaces against the macroexpanded `let*`-with-anonymous-
unit-bindings shape — a form the user did NOT write. Caller has to
mentally decompile the macroexpansion to understand the error. The
"macroexpand surfaces the underlying shape if asked" framing was
papering over the lie — from the caller's perspective, the
diagnostic doesn't tell the truth about what they typed.

The Honest test fails at the consumer surface even when it passes
at the source-availability surface. Stop at first NO. Option
A-revised is dead.

**Cost of the Honest cut:** ~150 LOC vs ~15 LOC. The user direction
"build complexity up from simplicity composition" picks the
honest-and-larger over the dishonest-and-smaller. The substrate
ALREADY has the special-form pattern; adding `do` to that family
follows established composition. No new entity kind; no novel shape.

## Sweep scope

Once the form ships, every wat-tests file gets a sweep:

```
grep -rn '((_ :wat::core::unit)' wat-tests/ crates/*/wat-tests/ | wc -l
```

Estimated 100+ sites. Note: arc 145 sweep 1b will have ALREADY
typed every existing `let*` with `-> :T` before arc 136 runs —
so the sweep target is typed `let*`-with-unit-bindings, not
untyped. The `:T` carries through the transform unchanged:

```
(:wat::core::let* -> :T
  (((_ :wat::core::unit) FORM-1)
   ((_ :wat::core::unit) FORM-2))
  FORM-3)
```
becomes:
```
(:wat::core::do -> :T FORM-1 FORM-2 FORM-3)
```

Pure mechanical 1:1 transform. The `-> :T` arc-145 already added
maps directly to the `do`'s `-> :T` slot.

The do form is MORE permissive at non-final positions than the
let*-with-unit-bindings crutch. Today's `((_ :unit) FORM-i)` slot
required FORM-i to be `:unit`-typed (binding-slot type-check).
After arc 136, `(:do -> :T FORM-i ... FORM-N)` allows FORM-i to
be ANY type — its value is discarded. Existing crutch sites all
have :unit-returning non-finals (otherwise arc 145 would already
have rejected them); the migration is clean. Future sites where
"discard non-unit return without binding ceremony" is wanted
become first-class.

Some sites are MIXED — `((_ :unit) ...)` interspersed with real
bindings. Those stay as `let*`. Phase-2 judgment.

## Slice plan

**Sequencing constraint:** arc 145 must close before arc 136
slice 1 spawns. Arc 145's sweep 1b types every existing `let*`
with `-> :T`; arc 136's slice 2 then sweeps those typed `let*`-
with-unit-bindings into typed `do`. If arc 136 runs before arc
145 ships, the sweep target shape is wrong (untyped) and the
transform breaks.

- **Slice 1a (substrate)** — mint `:wat::core::do` as a substrate
  special form per the Implementation surface sketch above:
  - `infer_do` arm in `src/check.rs` (mirror `infer_let_star`'s
    HEAD-position `-> :T` parsing + migration-hint MalformedForm)
  - `eval_do` + `eval_do_tail` + `step_do` arms in `src/runtime.rs`
  - Registry sketch update in `src/special_forms.rs`
  - 6-10 unit tests in new `tests/wat_arc136_do_form.rs`:
    - Empty: `(do -> :T)` parse error
    - Single: `(do -> :T f)` returns f's value
    - Multi: `(do -> :T f1 f2 f3)` evaluates all, returns f3's value, discards f1/f2 results
    - Final-form type-error: declared `-> :i64` but final form returns String
    - Non-final value discarded: a non-final form returning non-unit type evaluates cleanly (final form's type is the contract)
    - Untyped `(do f1 f2)` parse error with migration-hint MalformedForm
    - Reflection round-trip via lookup-form
  - This substrate change MAY break consumer sites that the user
    didn't intend to discard non-final results — sweep 1b (below)
    catches those via substrate-as-teacher.
- **Slice 1b (consumer migration)** — sweep ~100+ typed `let*`-with-
  unit-bindings sites → typed `do`. Substrate-as-teacher loop per
  arc 145's discipline:
  - For each `(:let* -> :T (((_ :unit) f) ...) body)` site, transform to `(:do -> :T f ... body)`
  - Run cargo test; mixed-binding sites (real bindings + unit-discards interspersed) stay `let*` and surface no error
  - Iterate per cargo error until 0-failed
  - Per `feedback_simple_is_uniform_composition.md`: N identical 1:1 transforms IS simple
  - Atomic commit with slice 1a per recovery doc § 7
- **Slice 2 (closure)** — INSCRIPTION + 058 row + USER-GUIDE entry +
  WAT-CHEATSHEET note + cross-references to arc 145 (typed-let
  precedent) + arc 108 (typed-`-> :T` precedent for value-bearing
  forms). Pre-INSCRIPTION grep mandatory per FM 11.

## Cross-references

- `.claude/skills/complectens/SKILL.md` — the spell whose calibration surfaced the need.
- `docs/arc/2026/05/135-complectens-cleanup-sweep/SCORE-SLICE-1.md` — the calibration record where the user named the crutch.
- arc 118 — the queued "lazy seqs vs threaded streams" arc (sibling pending; both are core-form additions).
- `wat/test.wat` — uses the let*-with-unit-bindings pattern extensively in deftest scaffolding; will benefit.

## When to start

After arc 145 closes (typed-let substrate + consumer migration shipped). Arc 145's sweep 1b types every existing `let*`-with-unit-bindings chain with `-> :T`; arc 136's slice 2 then sweeps those typed chains into typed `do`. Running arc 136 before arc 145 ships breaks the sweep target shape.

Original (pre-arc-145-revision) sequencing rationale: arc 135 closed first to land cleanup before the do-form sweep. That rationale still holds (arc 135 closed); the new gate is arc 145.

## Why this matters

The codebase advertises bad practices wherever the crutch shows. Wat is a Lisp; minting a clean `do` form is one line of new vocabulary that retires hundreds of lines of awkward let* boilerplate. The four questions answer YES at every test site after this arc.
