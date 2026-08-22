# DESIGN — a type reference must RESOLVE

**Status: written 2026-08-22, against `faaec192b`. Grounded, not yet ruled.**

A declaration may name a type that does not exist, and nothing says so.

```
(:wat::core::defn :user::f [x <- :user::NoSuchType] -> :wat::core::i64 0)
    target/release/wat --check   →  EXIT 0
```

Any namespace — `:user::`, `:wat::core::`, `:wat::cache::`. Parametric form spelling too:
`(:wat::cache::NoSuchType :- [:wat::core::i64])` is equally accepted.

## What it costs today

The phantom surfaces only when a CALLER supplies a value, and then it blames the caller:

```
#wat.check/TypeMismatch  ":user::takes: parameter #1 expects :user::NoSuchType; got :wat::core::i64"
```

The declaration is the defect; the call site gets named. This arc has already paid for that exact
shape once — `defsurface` reported *"triple is incomplete"* at a field that was fine. A return slot
behaves the same way: `:- :user::NoSuchType` produces `ReturnTypeMismatch: "body produces
:wat::core::i64; signature declares :user::NoSuchType"` — the checker treats the phantom as a real,
distinct type and reports a mismatch AGAINST it.

★ **And an UNCALLED declaration naming a phantom is accepted forever, silently.** There is no
caller to trip the mismatch, so nothing ever evaluates the name at all.

## Why now, and not later

This is not hygiene; it is a **prerequisite for trusting ②-iii's verdict.** The codemod's entire
output is type references — 805 of them in the 2026-08-21 dry-run. The plan for the re-run is
"migrate, floor, read what breaks." **A green floor is a far weaker statement than it looks if an
unresolvable type reference does not go red on its own**: it only goes red where a caller happens to
exercise it, so a mangled name on a rarely-called path ships silently. That is verbatim the failure
`a9168b851`'s slot rule exists to prevent, one layer down.

Close this first and the re-run's verdict upgrades from *"the tests still pass"* to *"every type
reference in the corpus names something real."*

## The mirror

`src/resolve/walk.rs:259` — `is_resolvable_call_head` — is one half of a pair with no twin:

```
CALL head    :user::totally-undefined     --check EXIT 1   #wat.resolve/UnresolvedReferences
             :wat::core::totally-undefined --check EXIT 0  → RUN EXIT 1  #wat.runtime/UnknownFunction
TYPE ref     :user::NoSuchType             --check EXIT 0   ⛔ no pass exists
```

⚠ The call-head reserved-prefix exemption is **deliberate, documented, and its deferral target
genuinely fires** — I ran it rather than trusting the comment. `is_resolvable_call_head` returns
`true` for any reserved prefix because *"the name-resolution pass is scoped to catch 'no such
namespace' mistakes, not 'wrong name inside a known namespace' mistakes"*, and a wrong leaf raises
`UnknownFunction` at runtime with a span. That half is LATE, not broken, and this stone does not
touch it. The type half has no late catch either — that is the asymmetry.

`[[feedback_the_mirror_is_an_instrument_not_a_fix]]`

## The ground — what already exists

Measured this session, all on the current disk:

```
src/value/symbol_table.rs:33   functions: HashMap<String, Arc<Function>>   (private; needs an iterator door)
src/check.rs:80                TypeScheme { type_params, params, ret, rest_param_type }
src/types.rs:522               TypeEnv::contains(&str) -> bool             ← THE DOOR. Already exists.
src/types.rs:516               with_builtins() → register_builtin_types    so `contains` covers :wat::core::i64 etc.
src/types.rs                   TypeDef's six variants EACH carry type_params: Vec<String>
src/freeze.rs:886-894          step 5 register_types → step 7 resolve → step 8 check
```

**The order is what makes this cheap.** Resolution (step 7) runs AFTER type registration (step 5),
so the `TypeEnv` is fully populated when the resolver runs. No new registry, no new pass ordering —
the door is `TypeEnv::contains`, and it is already public.

## ⛔ The one hard constraint — type VARIABLES are `Path`s

`src/types.rs:70-77`, verbatim:

> *"Lexically-scoped type variables (`:T`, `:K`, `:V`) also appear as `Path` when parsed — the type
> checker distinguishes them via the enclosing scheme's / declaration's `type_params`."*

`TypeExpr::Var(u64)` is synthetic and never produced by parsing. So a pass that walks type
expressions and asks `contains` on every `Path` **will flag every type variable in the corpus**. The
walk MUST carry the enclosing declaration's `type_params` as a scope and treat a `Path` matching one
as bound.

★ This is where the arc pays itself back: `:- [T …]` is the binder, and the binder IS the scope. The
work that made the param-spec explicit is what makes this pass writable at all.

## Decisions

Every option carries all four questions, flat YES/NO — including options already disqualified on an
earlier axis. A lean that stops at the first NO hides WHICH axis decided, and forced enumeration is
what surfaces the option that reads best and fails Honest.
`[[feedback_four_questions_for_any_multi_option_decision]]`

### D1 — Where does the pass live?

| # | option | Obvious | Simple | Honest | Good UX | verdict |
|---|---|:---:|:---:|:---:|:---:|---|
| **A** | **the RESOLVER (step 7), emitting `UnresolvedReference`** | **YES** | **YES** | **YES** | **YES** | **TAKE** |
| B | the CHECKER (step 8), new `CheckErrorKind::UnknownType` | YES | **NO** | YES | YES | reject — Simple |
| C | a project lint under `tests/lint/` | **NO** | YES | **NO** | **NO** | reject — Obvious·Honest·UX |
| D | validate inside `register_types` (step 5) | YES | **NO** | YES | **NO** | reject — Simple·UX |

**A.** *Obvious* — a type reference is a reference; the pass whose entire job is "does this name
resolve" answers it, and `UnresolvedReference` already carries `path` + `context`. *Simple* — one
question, one registry, already populated by step 5. *Honest* — reports the phantom as an unresolved
NAME at the declaration, which is what it is. *Good UX* — one diagnostic family for both halves of
the pair; a reader who has seen `UnresolvedReferences` for a call reads it for a type unchanged.

**B.** *Obvious* YES — types are the checker's subject and that is where a reader looks for type
errors. *Simple* **NO** — the checker reaches type expressions while UNIFYING, i.e. at use sites,
which is precisely the behaviour being fixed; catching an uncalled declaration needs a separate
declaration sweep, which is option A wearing the checker's coat. *Honest* YES — a
`CheckErrorKind::UnknownType` at the declaration span would tell the truth; nothing about this option
lies. *Good UX* YES — though it splits the pair across two diagnostic families, that is a cost, not a
falsehood. **Decided on Simple alone.**

**C.** *Obvious* **NO** — an unknown type is a language error, not a style rule; nobody expects
"unknown type" to arrive from the test suite. *Simple* YES — the lint harness exists and is cheap,
and this is the option's real attraction. *Honest* **NO** — a lint runs in OUR test suite only, so a
consumer running `wat --check` on their own program gets nothing while the language appears to check
it. *Good UX* **NO** — the error arrives from a test run rather than the compiler, and never at all
for a downstream user.

**D.** *Obvious* YES — validate a declaration as you register it is the naive first instinct and it
reads well. *Simple* **NO**, and this one is MEASURED rather than argued: step 5 registers in file
order, and **forward type references are legal**. `(defn :user::takes [x <- :user::Later] …)` above
`(defrecord :user::Later …)` resolves — proven with a control, since exit 0 alone proves nothing
while type refs go unresolved: passing a real `:user::Later` checks clean, passing an `i64` fails
with *"parameter #1 expects :user::Later; got :wat::core::i64"*. So the type genuinely resolves.
*Honest* YES. *Good UX* **NO** — validating in registration order would reject legal programs, and a
false rejection is worse UX than the silent acceptance being fixed. Deferring to the end of step 5 to
avoid that IS option A, one step early.

### D2 — Which type positions?

| # | option | Obvious | Simple | Honest | Good UX | verdict |
|---|---|:---:|:---:|:---:|:---:|---|
| **A** | **DECLARED positions only** (params, returns, fields, variant payloads, alias RHS, surface methods) | **YES** | **YES** | **YES** | **YES** | **TAKE** |
| B | every type expression anywhere, incl. inline `let`/`match` ascriptions | YES | **NO** | YES | YES | reject — Simple |

**A.** *Obvious* — "a declaration may not name a type that does not exist" is one sentence. *Simple*
— one list of slots, each already parsed into a `TypeExpr`. *Honest* — provided the diagnostic names
the slot, and provided the stone does not claim coverage of inline positions it never walked.
*Good UX* — the error lands where the author wrote the name.

**B.** *Obvious* YES. *Simple* **NO** — inline positions sit inside function BODIES, so scope is no
longer the declaration's `type_params` but whatever the enclosing expression has bound; that is a
different and larger mechanism. *Honest* YES. *Good UX* YES — strictly more coverage, which is the
option's genuine appeal. **Decided on Simple.** Inline ascriptions are affirmatively OUT OF SCOPE,
not deferred: they are checked at use by the existing unifier, and that position already works.

### D3 — Does the reserved-prefix exemption carry over to types?

| # | option | Obvious | Simple | Honest | Good UX | verdict |
|---|---|:---:|:---:|:---:|:---:|---|
| A | exempt `:wat::*`, mirroring `is_resolvable_call_head` | YES | YES | **NO** | **NO** | reject — Honest |
| **B** | **no exemption — every namespace** | **YES** | **YES** | **YES** | **YES** | **TAKE** |
| C | no exemption, but stdlib violations are WARNINGS (a ratchet) | **NO** | **NO** | **NO** | **NO** | reject — all four |

**A** is the dangerous one, because it passes the first two and reads as principled symmetry.
*Obvious* YES — it mirrors an existing documented rule. *Simple* YES — one prefix test, already
written. *Honest* **NO** — the call-head exemption is EARNED by a deferral target that fires
(`UnknownFunction` at runtime, verified this session rather than trusted from the comment); types
have no late catch at any stage, so copying the exemption without copying the catch preserves exactly
the hole this stone exists to close. *Good UX* **NO** — it aims the wall away from `wat/`, which is
the corpus ②-iii rewrites, leaving unprotected the user most likely to be bitten.

**B.** *Obvious* — it says what it checks and checks it. *Simple* — no prefix logic at all, which is
less code than A. *Honest* — no class of declaration is quietly exempt. *Good UX* — turning it on
will surface whatever is already wrong in the stdlib, and that is the point, not a cost.

**C** is the tempting "don't break the build" move and fails everything. *Obvious* **NO** — two
severities for one defect makes every reader ask which one they are. *Simple* **NO** — two code
paths, a severity policy, and a list to maintain. *Honest* **NO** — a warning nothing gates is a
violation that ships, and this arc has already recorded that a count-based ratchet cannot distinguish
"+1 new, −1 fixed" from "nothing happened" `[[feedback_a_gate_freezes_names_never_a_count]]`.
*Good UX* **NO** — it trains readers to scroll past the diagnostic.

### D4 — Scope for type variables

Not a decision; a constraint. Type variables parse as `TypeExpr::Path` (`src/types.rs:70-77`), so the
walk MUST carry the enclosing declaration's `type_params` and treat a matching `Path` as bound.
Carriers already exist: `TypeScheme.type_params` for functions, `TypeDef`'s per-variant `type_params`
for types.

One sub-question the brief settles by MEASUREMENT, not assumption: **a `fn` nested inside a `defn`
body** — does its binder extend the enclosing scope or shadow it? The stone's answer must be whatever
the checker already does, not a new rule invented here.
## What this stone does NOT do

- It does not touch `is_resolvable_call_head` or the reserved-prefix exemption for CALL heads. That
  half is late-but-honest and is a separate question nobody has asked.
- It does not check inline ascriptions inside function bodies (D2 option B) — those are checked at
  use by the unifier today, and that position works.
- It does not make the `ReturnTypeMismatch`/`TypeMismatch` messages smarter. Once a phantom cannot
  reach the checker, those messages stop being reachable by this cause.

## Acceptance — the shape, not the rows

The rows belong in the BRIEF. Two properties decide the stone:

1. **A phantom in an UNCALLED declaration is rejected.** This is the case nothing catches today, and
   the only one that proves the pass is a declaration sweep rather than a use-site check.
2. **Every type VARIABLE in the existing corpus still passes.** The floor is the instrument: `wat/`
   is full of parametric declarations, and a pass that mishandles scope will light up in the
   hundreds. A green floor here is meaningful precisely because the corpus is large.

⛔ **And the negative control that must exist before either row means anything:** the two shapes of a
probe I ran today BOTH returned five greens while measuring nothing — a bare `typealias` file, and a
`defn` signature naming an unresolvable type, each exiting 0 for reasons unrelated to the subject.
**The acceptance must name a command that FAILS today and passes after, and the brief must run it
before the work starts.** `[[feedback_a_green_test_can_prove_nothing]]`
