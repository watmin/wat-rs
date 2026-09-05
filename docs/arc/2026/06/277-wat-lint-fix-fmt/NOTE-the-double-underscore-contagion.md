# NOTE — the `__` prefix is a contagion, and it belongs to the LINTER, not the formatter (2026-09-05)

> **Builder:** *"the `__` prefix ..... i've wanted that dead for a long time..... its completley
> unnecessary...... we have racket's hygeine ..... some agent started doing that and it propagated
> like a contagion.... i think its best to attack that in our linter.... the formatter.... should
> just format....."*

## ⛔ THIS BLOCKS NOTHING. IT IS PARKED, NOT PENDING.

> **Builder, immediately after:** *"do we need to deal with the `__` names now?.... aren't we working
> on rules for how good wat should look as code forms?... the names we use in the binders can just be
> processed using whatever names they are given?...... we don't need to rename anything now... right?"*

**Right. No rename is needed for any layout work, now or later.** To the formatter a binder name is
an OPAQUE TOKEN — it enters layout only through its *length* (the 120-column budget, and alignment
columns). `wat fmt` reads whatever is written and organises around it.

The one interaction, named so it is not rediscovered: renaming `__datum` → `datum` later shifts the
alignment columns on those lines. **That is a non-issue by construction** — a canonical formatter
RE-DERIVES alignment from content on every run, so the answer is to run it again. The two sweeps
touch the same lines and do not conflict, in either order.

★ This NOTE exists because the `__` finding fell out of the R16 *manufacturing* investigation, not
because it gates anything. It is a lint work item waiting for a taker.

## ★ THE SCOPE RULING, and it is the important half

**`wat fmt` formats LAYOUT. It never touches a NAME.** A rename is a semantic transform — it can
capture, shadow, and change meaning — while layout provably cannot. Braiding the two would make
every reformatting commit a potential behaviour change and destroy the one property that makes a
canonical formatter safe to run over 1,740 files unattended.

So the `__` sweep is **wat-lint + a recorded `wat-fix` codemod** (R21 — *"we use wat-fix to unfuck
the farm"*), tracked here in 277 because 277 is lint-fix-**fmt** and this is the lint third.

## THE MEASUREMENT

```
1,391 occurrences   ·   303 files   ·   12 of them in the STDLIB

__cause  530  ┐
__recv   290  │ 1,341 = 96% of all occurrences
__datum  278  │
__forms  243  ┘
__d 8 · __work 7 · __f 6 · __pool-work 5 · __c 5 · __runner 3 · __pool-runner 3 · __x 2
__start 2 · __r 2 · __kwargs 2 · __acc 2 · __internal 1 · __hdr 1 · __ftr 1
```

Stdlib files carrying it: `Record.wat` `bracket.wat` `core.wat` `deporder.wat` `doctest.wat`
`fix.wat` `grep.wat` `kernel/readln.wat` `lint.wat` `rete/compile.wat` `service.wat`
`telemetry/journal.wat`.

## ⭐ IT IS HAND-WRITTEN. NOTHING EMITS IT.

I expected to find a macro emitting these — a `readln` sugar expanding to `__datum`, say — because
the `__` convention *reads* like generated hygiene naming. **There is no such emitter.** No macro in
`wat/`, no Rust codegen, no gensym path produces a `__` name. Every one of the 1,391 was typed.

★ **The long tail is the proof of contagion.** The four big names could be one copied idiom, but
`__d`, `__x`, `__acc`, `__hdr`, `__ftr` are one-offs — someone applied the *pattern* to a fresh
name, having inferred a convention from the code around them. That is imitation, not derivation,
and it is exactly how a corpus teaches the wrong thing to its next reader (the reason the builder
made beauty a correctness property in `[[STYLE-TABLE-draft]]`'s R15).

## AND IT BUYS NOTHING — the hygiene is real

`src/runtime.rs:11978`:

```
// Wat is hygienic; identifier matching uses (name, scope set) so
```

That is Racket's set-of-scopes model. A macro-introduced binder **cannot** capture a user's binder
whatever it is called, and a user's binder cannot capture a macro's. The prefix defends against a
failure the substrate already makes unrepresentable — a **convention wearing a wall's clothes**,
which the grimoire names as the thing that rots.

## ⛔ THE ONE REAL HAZARD IN THE SWEEP — and it is a STOP, not a caveat

The codemod is *"strip the leading `__`"*, and that is **not** unconditionally safe. Hygiene protects
MACRO-introduced names; these are ordinary source binders in ordinary scopes. Renaming
`((:…::Datum __datum) __datum)` to `((:…::Datum datum) datum)` **introduces capture** if that arm's
body also references an OUTER `datum`.

> **STOP-1 for whoever takes this:** for each rename site, if the target name (the `__` stripped) is
> already bound or referenced anywhere in the binder's scope, STOP and surface it. Do not
> disambiguate by inventing a suffix; the point is to remove noise, not relocate it.

**This is measurable before the sweep, not during it** — the fact base already holds what is needed
(`wat/grep.wat`'s `Node.parent` gives scope containment, `Named` gives the references). A census of
"sites where the stripped name already occurs in an enclosing scope" is the pre-flight, and if it
comes back zero the codemod is unconditional.

⚠ **Not measured yet. I am not claiming the collision set is empty — only that it is cheap to ask.**

## WHY THIS IS 277's, and what it shares with the formatter

The `__` sweep and `wat fmt` are the same doctrine at two layers, and both are
`[[SELF-FIXING-TOOLCHAIN]]`: *a tool alone is a suggestion; a tool PLUS a rule that finds every place
the old form survives is a cure.* One lints NAMES and rewrites them; the other lints LAYOUT and
rewrites it. **They must stay separate programs** — see the scope ruling above — but they share the
fact base, the codemod harness, and the corpus gate.
