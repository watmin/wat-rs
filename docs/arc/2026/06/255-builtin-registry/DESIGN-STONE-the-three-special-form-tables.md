# DESIGN — STONE: three tables answer "is this a special form"; the registry already answers it

> **Builder, 2026-09-04:** *"what hand lists are still maintained.... those... are the things the
> registry must satisfy...... those are the targets."*
>
> Governed by `[[RULING-the-registry-is-the-sole-authority]]` — items **1** (every name), **4**
> (what they take), **7** (reflection). This is a **step-3 stone**: *the consumer asks, the
> duplicate dies.*

## The three tables — measured 2026-09-04

```
THE REGISTRY        #[wat_special_form(...)]        Kind::SpecialForm + entry.syntax
src/special_forms.rs   build_registry()             35 entries · 32 ALSO registered  (91%)
src/runtime.rs:5050    a LOCAL `const SPECIAL_FORMS` 11 entries · 10 ALSO registered  (91%)
                                                     · 10 of 11 ALSO in special_forms.rs
```

The third was not in the RULING's census and not in this arc's worklist: it is a `const` declared
**inside `eval_apply`**, used once, five lines later, for the STOP-8 "cannot apply a special form"
rejection.

## ★★★ THE MEASUREMENT THAT MAKES THIS A DELETION, NOT A MIGRATION

`eval_signature_of_defn` (`src/reflect/verbs.rs:83`) matches in this order:

```
:201   Binding::Registered  if !entry.syntax.is_empty()          →  renders entry.syntax
:227   Binding::Registered  if !entry.args.is_empty()            →  renders from args
:247   Binding::Registered  if lookup_special_form(&n).is_some() →  `let _ = entry;`  ← DISCARDS IT
```

**Arm 201 fires first for every registered special form carrying `@syntax`.** Verified by running
the substrate, not by reading it:

```
(signature-of-defn :wat::core::let)   → "(:wat::core::let [<binder> <expr> ...] <body>+)"
(signature-of-defn :wat::core::match) → "(:wat::core::match <scrutinee> (<pattern> <body>) ...)"
(signature-of-defn :wat::core::fn)    → "(:wat::core::fn [<param> <- :T ...] -> :RetType <body>+)"
```

Every one is the **registry's `@syntax`**. None is the hand-built sketch. **So for those names the
`special_forms.rs` rows are already unreachable, and deleting them changes no behaviour at all.**

★ And arm 247 is the RULING's violation in its purest form: the registry answered
(`Binding::Registered`), and the arm writes `let _ = entry;` to throw that answer away and re-ask
the hand-list.

## ⛔ AND THE HAND-BUILT SKETCH IS NOT MERELY DUPLICATE — IT IS WRONG

Of the 23 names carrying both a sketch and an `@syntax`, **12 agree and 11 disagree**, and in every
disagreement the sketch is the stale one:

```
:wat::core::match   sketch  (:wat::core::match <scrutinee> -> <T> <arm>+)
                    @syntax (:wat::core::match <scrutinee> (<pattern> <body>) ...)
:wat::core::let     sketch  (:wat::core::let <bindings> <body>+)
                    @syntax (:wat::core::let [<binder> <expr> ...] <body>+)
:wat::core::fn      sketch  (:wat::core::fn <params> <body>+)
                    @syntax (:wat::core::fn [<param> <- :T ...] -> :RetType <body>+)
:wat::core::defmacro · defenum · def · newtype · typealias · use! · digest-load! · signed-load!
```

★★★ **`special_forms.rs` contains, at its own line 172, the string
*"`:wat::core::match` no longer takes `-> :T`"*** — the file documents that exact retirement while
its own table still ships the retired shape. The only reason no user sees it is that arm 201
already shadows the row.

## The three buckets — derived, not chosen

```
23  registered AND carrying @syntax   → row is ALREADY DEAD. Delete. Zero behaviour change.
 9  registered, NO @syntax            → arm 201 cannot fire; the hand-list is still LIVE for these.
                                         MOVE the sketch to an `@syntax` at the registration site,
                                         then delete the row.
                                         (Option/expect · Option/try · Result/expect · Result/try ·
                                          and · if · or · form::matches? · holon::literal)
 3  NOT registered                     → STAY. defstruct is a stdlib macro with no registration
                                         site (the FOURTH-registry fork); unquote and
                                         unquote-splicing are punctuation, not verbs
                                         ([[NOTE-unquote-is-punctuation-not-a-verb]]).
```

⛔ **For the 9, transcribe the CURRENT rendering exactly — move the data, do not redesign it.**
Changing what those nine render is a separate question from killing the duplication, and bundling
them would make a behaviour change hide inside a deletion. Where a sketch looks wrong, flag it in
the report; do not fix it here.

## `runtime.rs:5050` — membership only, and one honest exception

`eval_apply`'s STOP-8 check asks one question: *may `apply` dispatch this head?* The registry
answers it — `lookup_entry(head).map(|e| e.kind) == Some(Kind::SpecialForm)` — and that exact test
is already used at `src/reflect/lookup.rs:418` and `src/intrinsic/reflect.rs:384`.

⚠ **10 of its 11 names are registered. The eleventh is `:wat::core::defn`, and it is not a special
form at all — it is a stdlib macro.** It sits in a list named `SPECIAL_FORMS` because `apply` must
reject it, not because the name is one. It cannot be answered by the registry today (the
FOURTH-registry fork: 41 stdlib macros are invisible). It stays as a **named, reasoned exception
beside the registry query**, never silently folded in.

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **make both consumers ask; delete the dead rows** | YES | YES | YES | YES |

- **Obvious? YES** — three tables answering one question, and the substrate already shows which one
  wins.
- **Simple? YES** — one membership query replacing a `const`, one `@syntax` per name for nine names,
  and a deletion. No new type, no new field, no new mechanism.
- **Honest? YES** — the sharpest one. The hand-list currently ships a `match` grammar the same file
  documents as retired. A table that is shadowed *and* wrong is worse than a table that is merely
  redundant, because the day arm 201 stops firing it becomes a lie with no warning.
- **Good UX? YES** — a reader asking "what is a special form?" gets one answer from one place.

## Scope

**In:** `runtime.rs:5050`'s const replaced by a registry query with `defn` as a named exception ·
`@syntax` added at the registration site for the 9 · the 32 duplicated rows deleted from
`special_forms.rs` · arm 247 in `eval_signature_of_defn` retired if the deletion makes it
unreachable · `reflect/verbs.rs`/`lookup.rs`'s remaining `lookup_special_form` consumers pointed at
the registry where the registry can answer.

**Out of scope, affirmatively:**
- **Correcting any sketch's content.** The 9 transcribe as-is. Fixing a wrong grammar is a
  different stone and must not ride inside a deletion.
- **`defstruct` / `unquote` / `unquote-splicing`.** No registration site exists; the FOURTH-registry
  fork owns `defstruct` and `[[NOTE-unquote-is-punctuation-not-a-verb]]` owns the other two.
- **Deleting `special_forms.rs` entirely.** Three rows survive, so the file survives. A stone that
  claimed otherwise would be claiming the fork above is closed.
