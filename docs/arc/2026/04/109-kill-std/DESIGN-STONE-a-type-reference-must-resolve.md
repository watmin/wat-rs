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

### D1 — Where does the pass live: the resolver (step 7) or the checker (step 8)?

**Option A — the resolver, beside `is_resolvable_call_head`, emitting `UnresolvedReference`.**
- *Obvious?* **YES.** A type reference is a reference. The struct already carries `path` + `context`,
  and the context string is where "type position in the signature of :user::f" goes.
- *Simple?* **YES.** One question — "does this name resolve?" — answered in the pass whose whole job
  is that question, against a registry populated two steps earlier.
- *Honest?* **YES.** It reports the phantom as an unresolved NAME at the declaration, which is what
  it is, rather than as a mismatch at a caller.
- *Good UX?* **YES.** One diagnostic kind for both halves of the pair; a reader who has seen
  `UnresolvedReferences` for a call already knows how to read it for a type.

**Option B — the checker, as a new `CheckErrorKind::UnknownType`.**
- *Obvious?* **YES.** Types are the checker's subject.
- *Simple?* **NO.** The checker already walks these expressions to UNIFY them, so the phantom is
  reachable there — but it is reachable at USE sites, which is exactly the behaviour being fixed. To
  catch an uncalled declaration the checker would need a separate declaration sweep, i.e. Option A's
  pass wearing the checker's coat.
- Fails Simple → disqualified.

**Ruling sought: A.** The one thing that could overturn it: if step 7 cannot see declarations that
step 8 can (e.g. surface/field types registered later than `register_types`). **Unverified —
name it as the brief's first STOP.**

### D2 — Which type positions?

**Option A — DECLARED positions only:** `defn`/`fn` parameter and return slots, record/struct field
types, enum variant payloads, typealias right-hand sides, surface method signatures.
- *Obvious?* **YES.** "A declaration may not name a type that does not exist" is one sentence.
- *Simple?* **YES.** One list of slots, each already parsed into a `TypeExpr`.
- *Honest?* **YES** — provided the diagnostic says which slot, and provided the stone does not claim
  to cover inline ascriptions it did not walk.
- *Good UX?* **YES.** The error lands where the author wrote the name.

**Option B — every type expression anywhere, including inline `let` ascriptions and `match` arms.**
- *Obvious?* **YES.**
- *Simple?* **NO.** Inline positions are inside function BODIES, which means scope is no longer just
  the declaration's `type_params` — it is whatever the enclosing expression has bound. That is a
  different, larger mechanism.
- Fails Simple → disqualified for this stone, and affirmatively **out of scope**: inline ascriptions
  are checked at use by the existing unifier, which is the position that already works.

**Ruling sought: A.**

### D3 — Does the reserved-prefix exemption carry over?

**Option A — exempt `:wat::*` type references, mirroring `is_resolvable_call_head`.**
- *Obvious?* **YES**, by symmetry with the call-head rule.
- *Simple?* **YES.**
- *Honest?* **NO.** The call-head exemption is earned by a deferral target that FIRES —
  `UnknownFunction` at runtime. There is no equivalent for types: a phantom type in an uncalled
  declaration raises nothing, ever, at any stage. Copying the exemption without copying the late
  catch keeps the exact hole this stone exists to close, and `wat/` is precisely the corpus ②-iii
  rewrites.
- Fails Honest → disqualified.

**Option B — every namespace, no exemption.**
- *Obvious?* **YES.** *Simple?* **YES.** *Honest?* **YES** — it says what it checks and checks it.
- *Good UX?* **YES**, with one cost: turning it on will surface whatever is already wrong in the
  stdlib, and that is the point.

**Ruling sought: B.**

⚠ **Do NOT pre-census the violations with grep.** Every count this arc has taken by pattern-matching
`.wat` text has been wrong, and the one that mattered most was invisible to source entirely — a
`defservice`-generated type that appears in no file. **Impose the wall and read the screams.**
`[[feedback_impose_the_check_and_read_the_screams]]`

### D4 — Scope for type variables

Not an option; a constraint (see above). The walk carries the enclosing declaration's `type_params`
and treats a `Path` matching one as bound. The carriers already exist: `TypeScheme.type_params` for
functions, `TypeDef`'s per-variant `type_params` for types.

The open sub-question the brief must settle by measurement, not assumption: **a `fn` nested inside a
`defn` body** — does its own binder extend the outer scope or shadow it? The stone's answer must be
whatever the checker already does, not a new rule.

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
