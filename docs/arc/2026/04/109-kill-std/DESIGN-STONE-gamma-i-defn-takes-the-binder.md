# DESIGN — arc 109 γ-i: `defn` / `fn` take the `:- [T …]` binder

> **RULED D1 (builder, 2026-08-21): γ-i goes FIRST**, before the identity stone. It is the smallest
> of ②-iii's four blockers and the committed codemod already rewrites 40 stdlib declarations into
> the form this stone makes legal.
>
> ⛔ **One fork inside it is UNRULED — decision E below. No brief until E is ruled, because E
> decides the rooms.**

## The spec, from the builder

```clojure
[:-> X]              0-arity, produces X    ⇔  (wat.core/fn :- [X]           []                :- X …)
[A :-> X]            1-arity                ⇔  (wat.core/fn :- [A X]         [a :- A]          :- X …)
[A B :-> X]          2-arity                ⇔  (wat.core/fn :- [A B X]       [a :- A b :- B]   :- X …)
[A B C D E :-> X]    5-arity                ⇔  (wat.core/fn :- [A B C D E X] [a :- A …]        :- X …)
```

**The binder lists every type var, the return's included** — `[:-> X]` binds `X` though `X` appears
nowhere but the return. Two consequences, both checked against the disk rather than assumed:

- **An occurrence in RETURN position is consumption.** `check_type_params_consumed`'s `Surface` arm
  already walks Method args AND ret, so the wall agrees; `defn` mints no `TypeDef`, so the wall never
  reaches it. Nothing to change — recorded so it is not rediscovered.
- **Arity is STRUCTURAL** — the position of `:->` is the arity; there is no separate count to
  disagree with it. That is why the nullary `[:-> X]` needs no special case. Probed:
  `[:-> :wat::core::Record]` checks clean, and its two real sites (`wat/spawn.wat:51`, `:105`) only
  CARRY the value, never apply it.

## The gap, measured

```
(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)
  ⛔ "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got keyword"
(:wat::core::defn :user::f<T>   [x <- :T] -> :T x)                                    ✅ clean
```

Every other head in the codemod's declarator list ACCEPTS the binder — probed one by one: `defenum`,
`defrecord`, `holon::defrecord`, `defstruct`, `defsurface`, `typealias`, `newtype`, `typeunion`.
**`defn` alone rejects.** Population: **40** parametric `defn`/`fn` in `wat/` (`test`, `spawn`,
`bracket`, `io`, `seq`, `cache`), 57 corpus-wide.

## ★ Why it rejects — and it is NOT an oversight

`parse_fn_signature_prefix` (`src/function/parse.rs:145`) takes **`&[WatAST; 3]`** —
`[ARGS-VECTOR, ->, :RET-TYPE]` — with its own doc: *"Arity is type-guaranteed — no runtime arity
check required"* (Stone 243.4.1, `CONFORMARE.md`'s worked example of making arity type-impossible).
**A deliberate constraint-engineering wall is what makes the binder unrepresentable here**, so the
fix belongs at the CALLER that slices those three, never inside the wall.

That caller already has the hook: **`peel_metadata_preamble`**, run immediately before the slice at
`src/function/eval.rs:42` and `src/function/infer.rs:105`. A binder peel is a second peel in the same
place.

Two slots inside the signature are already migrated and need nothing: the arrow dual-reads `->` and
`:-` (251.4a), and the RET slot takes Keyword / Symbol / List / Vector via `parse_type_node`
(251.3a). **The binder is the last un-migrated slot of the fn form.**

## ★★ The proven shape to copy — do NOT invent one

On the TYPE side, α paired two functions:

```
parse_declared_name      reads `<T,…>` off the name keyword          src/types.rs:4390
take_declared_binder     consumes an optional `:- [T …]` from the    src/types.rs (7 callers,
                         arg stream, and ERRORS when BOTH are        all TYPE declarators)
                         present — "a contradiction, never something
                         to silently resolve"
```

On the FUNCTION side, `split_name_and_type_params` (`src/runtime.rs:4156`) is
`parse_declared_name`'s exact twin — and **has no `take_declared_binder`**. That single missing
pairing is this whole stone. Its callers: `runtime.rs:3410, 3424, 3556, 3676, 3689` and
`freeze/env.rs:398`.

⚠ The both-spellings contradiction error is NOT optional garnish — it is what stops a half-applied
codemod from silently picking one. Copy it.

## ⛔ Decision E — where the binder is CONSUMED (UNRULED)

`defn` is a **wat macro** (`wat/core.wat:673`, `[name & rest]`), so `:- [T]` currently lands in
`rest` and is forwarded verbatim into the emitted `fn` form — which is exactly why the error reads
as an `fn` signature error at a `defn` site.

- **E1 — surface alias in the macro.** `defn` peels `:- [T …]` and re-emits the name as
  `name<T,…>`. No Rust change at all.
- **E2 — first-class down to `def`/`fn`.** `split_name_and_type_params` gains its
  `take_declared_binder` twin; `peel_metadata_preamble` peels the binder; the `defn` macro merely
  forwards what it already forwards.
- **E3 — both:** E2, plus the macro also normalizes.

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **E1** surface alias in the `defn` macro | YES | YES | **NO** | — |
| **E2** first-class in `def`/`fn` | YES | YES | YES | YES |
| **E3** both | **NO** | **NO** | YES | — |

**E1 fails Honest** — it makes the macro MANUFACTURE the angle spelling that ③ makes illegal. That is
`defservice`'s disease exactly (blocker 3: a macro that emits the retired form, so a migrated corpus
regrows it at every expansion), adopted deliberately in a stone whose purpose is to retire that form.
It also leaves `(:wat::core::fn :- [T] …)` — the builder's own spelling, anonymous — still rejected,
because only `defn` would have learned anything.

**E3 fails Obvious and Simple** — two peels for one spelling, and a reader must work out which one
fires. It buys nothing E2 does not: once `fn` takes the binder, the macro forwarding it IS the
support.

**E2 is the recommendation**, and it is the only option that makes the builder's anonymous
`(wat.core/fn :- [A B X] [a :- A b :- B] :- X …)` legal.

## Blast radius

`src/function/parse.rs` (peel + the `ParseStep` arm), `src/function/eval.rs`,
`src/function/infer.rs`, `src/runtime.rs` (the `split_name_and_type_params` pairing).
**No `.wat` corpus change** — the 40 sites keep their `<T,U>` spelling until ②-iii re-runs.

⚠ A stdlib `.wat` edit is INVISIBLE until a rebuild and **a rider cannot test one**. E2 needs no
stdlib edit, which is a further point in its favour: the rider stays entirely in `src/`.

## Acceptance — the row that must be PROVEN, not assumed

**After the name moves from `foldl-spec<T,U>` to `foldl-spec :- [T U]`, the body still resolves `:T`
and `:U`.** Both spellings should feed the same `type_params` — and "should" is where all four
②-iii blockers lived. The load-bearing row is a real generic whose body USES its params, checked in
both spellings, not merely parsed:

```clojure
(:wat::core::defn :user::fold<T,U>  [f <- [U T :-> U] init <- :U] -> :U (f init …))   ← control
(:wat::core::defn :user::fold :- [T U] [f <- [U T :-> U] init <- :U] -> :U (f init …)) ← subject
```

Plus: BOTH spellings on one declaration must ERROR (the contradiction), the anonymous
`(:wat::core::fn :- [T] …)` must check, and `[:-> X]` nullary must stay clean.

⚠ **Scope the check from the RULE, not the diff** — this stone exists because a verification landed
on the six heads a diff added and never touched the one that was already in the list.
`[[feedback_scope_the_check_from_the_rule_not_the_diff]]`

## The four questions on γ-i itself

- **Obvious?** YES — `defn` is the only declarator head that refuses the operator every other head
  takes, and the corpus already writes the binder at 40 sites through the committed codemod.
- **Simple?** YES — one missing pairing (`split_name_and_type_params` ← `take_declared_binder`),
  copied from a shape proven at seven call sites.
- **Honest?** YES — the alternative (dropping `defn` from the codemod's head list) renders a
  declaration NAME as a REFERENCE: `(:wat::core::defn (:wat::core::foldl-spec :- [T U]) …)`, the
  silent corruption `a9168b851` exists to prevent.
- **Good UX?** YES — it makes the builder's own spelling legal, anonymous `fn` included, and it is
  independent of the identity stone, so it can land and be floored on its own.
