# DESIGN — arc 109 γ-i: `def` takes the `:- [T …]` binder

> **RULED, builder 2026-08-21.** **D1** — γ-i goes FIRST, before the identity stone. **E₀-b** — γ-i
> covers only the `def` name-binder; the anonymous `fn` binder is its own stone (γ-i-b). **E2** —
> the binder is consumed by `def`, first-class; the `defn` macro forwards it there.
>
> ⚠ **CORRECTED 2026-08-21 after macroexpanding.** The first draft of this DESIGN named `fn` as the
> landing site and listed `src/function/*` as the blast radius. That was wrong, and it was wrong
> because it reasoned from the ERROR MESSAGE (`"fn signature: …"`) instead of from the expansion.
> `wat-rs/CLAUDE.md` R4: *Debugging a MACRO? READ THE EXPANDED FORM FIRST.* Recorded rather than
> quietly fixed — the wrong version was the one that reached the builder.

## The spec, from the builder

```clojure
[:-> X]              0-arity, produces X    ⇔  (wat.core/fn :- [X]           []                :- X …)
[A :-> X]            1-arity                ⇔  (wat.core/fn :- [A X]         [a :- A]          :- X …)
[A B :-> X]          2-arity                ⇔  (wat.core/fn :- [A B X]       [a :- A b :- B]   :- X …)
[A B C D E :-> X]    5-arity                ⇔  (wat.core/fn :- [A B C D E X] [a :- A …]        :- X …)
```

**The binder lists every type var, the return's included.** Two properties, both checked against the
disk rather than assumed:

- **An occurrence in RETURN position is consumption.** `check_type_params_consumed`'s `Surface` arm
  already walks Method args AND ret; `def`/`defn` mint no `TypeDef`, so the wall never reaches them.
  Nothing to change — recorded so it is not rediscovered.
- **Arity is STRUCTURAL** — the position of `:->` IS the arity, so nullary needs no special case.
  Probed: `[:-> :wat::core::Record]` checks clean; its two real sites (`wat/spawn.wat:51`, `:105`)
  only CARRY the value, never apply it.

## The gap, measured

```
(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)
  ⛔ "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got keyword"
(:wat::core::defn :user::f<T>   [x <- :T] -> :T x)                                    ✅ clean
```

Every other head in the codemod's declarator list ACCEPTS the binder — probed one by one: `defenum`,
`defrecord`, `holon::defrecord`, `defstruct`, `defsurface`, `typealias`, `newtype`, `typeunion`.
**`defn` alone rejects.** Population: **40** parametric `defn`/`fn` in `wat/` (`test`, `spawn`,
`bracket`, `io`, `seq`, `cache`), 57 corpus-wide — every one already rewritten into the binder form
by the COMMITTED codemod.

## ★ The expansion, which is what decides the stone

```clojure
(:wat::core::defn :user::f<T,U> [x <- :T y <- :U] -> :T x)
;; macroexpands to
(:wat.core/def :user/f<T,U> (:wat.core/fn [x <- :T y <- :U] -> :T x))
```

**The type params ride the `def` NAME. The `fn` gets NONE** — `Function.type_params` is hardcoded
`Vec::new()` at `src/function/eval.rs:66`. The error names `fn` only because the macro forwards the
stray `:-` into the emitted `fn` form, where `parse_fn_signature_prefix` meets it in the args-vector
slot. **`fn` is where the error surfaces; `def` is where the fix belongs.**

⚠ `parse_fn_signature_prefix` (`src/function/parse.rs:145`) takes **`&[WatAST; 3]`** with its own
doc — *"Arity is type-guaranteed — no runtime arity check required"* (Stone 243.4.1,
`CONFORMARE.md`'s worked example of making arity type-impossible). That deliberate wall is a REASON
NOT TO LAND THE BINDER THERE, not an obstacle to work around. Under E2 `fn` never sees a binder and
the wall stands untouched.

## ★★ The proven shape to copy — do NOT invent one

On the TYPE side, α paired two functions:

```
parse_declared_name      reads `<T,…>` off the name keyword                  src/types.rs:4390
take_declared_binder     consumes an optional `:- [T …]`, and ERRORS when    src/types.rs, 7 callers,
                         BOTH are present — "a contradiction, never          all TYPE declarators
                         something to silently resolve"
```

On the `def` side, `split_name_and_type_params` (`src/runtime.rs:4156`) is `parse_declared_name`'s
exact twin — and **has no `take_declared_binder`**. That single missing pairing is this stone.

⚠ The both-spellings contradiction error is NOT garnish — it is what stops a half-applied codemod
from silently picking one. Copy it.

## ⛔ THE HAZARD — `def`'s shape is hand-rolled in SEVEN places, and every one fails SILENTLY

`(def :name expr)` / `(def :name {meta} expr)` is validated by an open-coded `len() != 3 && len() != 4`
guard at each of:

```
src/check.rs:545          src/runtime.rs:1291        src/runtime.rs:3395  try_parse_fn_shape_def
src/check.rs:8445         src/runtime.rs:2649        src/runtime.rs:3551  try_parse_variadic_def_fn_form
                                                     src/runtime.rs:3671  try_parse_user_variadic_def_fn_form
```

**Every one of them treats an unexpected arity as "malformed; the type checker already caught it"
and `continue`s or returns `Ok(())`.** So a 5/6-item `def` that one guard has not learned is not an
error — it is a binding that **silently fails to register**. Widening seven guards by hand is the
same worklist shape that has produced every miss in this arc.

### The ONE contract decision

**A single `split_def_form(items) -> (name, type_params, metadata, expr)` door replaces all seven
hand-rolls.** Adding a slot then touches one function, and a guard that has not learned the shape
cannot exist because there is only one.

This is not a novel proposal — it is the consolidation this arc already ran. `is_binder_marker`'s
own doc: *"the one door 251.8a consolidated four hand-rolled checks into."* Same move, same arc,
one level over.

## ⚠ The trap E2 must not spring — the kwargs bundle

`defn`'s kwargs path derives its bundle type as `{b}::Kwargs{p}`, where `{p}` is `name-tp` — the
**string suffix taken off the name**. A `defn` written with the binder has an empty `name-tp`, so a
parametric kwargs `defn` would silently mint a MONOMORPHIC `Kwargs` bundle. `defn` carries the same
`name-parametric?` / `name-base` / `name-tp` string surgery `defservice` does, and re-attaches it in
`{b}::Kwargs{p}` and `:{b}$impl{p}`.

**Zero instances in `wat/` today** — which is exactly why it is an ACCEPTANCE ROW and not a corpus
census. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`

## Blast radius

```
src/runtime.rs     split_def_form (new, ONE door) + the split_name_and_type_params pairing;
                   the five def-shape sites route through it
src/check.rs       :545 and :8445 route through the same door
wat/core.wat       the defn macro forwards `:- [T …]` into the emitted `def` rather than
                   into the emitted `fn`; name-tp/name-base learn the binder spelling
```

**No `.wat` corpus migration** — the 40 sites keep `<T,U>` until ②-iii re-runs. **`src/function/*`
is NOT touched**: `fn` never sees the binder.

⚠ A stdlib `.wat` edit is INVISIBLE until a rebuild and **a rider cannot test one**. The
`wat/core.wat` half is rider-edits, orchestrator-builds-and-floors.

## Acceptance — the rows that must be PROVEN

The load-bearing row is a generic whose body USES its params, in BOTH spellings — parsed is not
enough:

```clojure
(:wat::core::defn :user::fold<T,U>     [f <- [U T :-> U] init <- :U] -> :U …)   ← control
(:wat::core::defn :user::fold :- [T U] [f <- [U T :-> U] init <- :U] -> :U …)   ← subject
```

1. Both spellings check, and the body resolves `:T` / `:U` in each.
2. BOTH on one declaration → the contradiction ERROR, not a silent pick.
3. A parametric **kwargs** `defn` in binder spelling mints `Kwargs<T,U>`, not a monomorphic bundle.
4. A **variadic** `defn` in binder spelling registers — the `try_parse_*_variadic_def_fn_form` paths.
5. A `def` of a NON-fn value still registers (the door must not narrow `def` to fn-shapes).
6. `[:-> X]` nullary stays clean; the anonymous `(:wat::core::fn :- [T] …)` stays REJECTED — that is
   γ-i-b, and its rejection is the CONTROL proving γ-i did not silently widen.

⚠ **Scope the check from the RULE, not the diff** — this stone exists because a verification landed
on the six heads a diff added and never touched the one already in the list.

## The four questions

### On γ-i itself

- **Obvious?** YES — `defn` is the only declarator head refusing the operator every other head takes,
  and the committed codemod already writes the binder at 40 stdlib sites.
- **Simple?** YES — one missing pairing, copied from a shape proven at seven call sites.
- **Honest?** YES — the alternative (dropping `defn` from the codemod's head list) renders a
  declaration NAME as a REFERENCE, the silent corruption `a9168b851` exists to prevent.
- **Good UX?** YES — independent of the identity stone, so it lands and floors on its own.

### E₀ — does γ-i include the anonymous `fn` binder?  **RULED: E₀-b**

*Shared premise, and the measurement refutes it: that both capabilities are one stone.*

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **E₀-a** both — `def`'s name-binder AND anonymous `fn` binding its own vars | YES | **NO** | YES | — |
| **E₀-b** only the `def` name-binder; the anonymous form is γ-i-b | YES | YES | YES | YES |

**E₀-a fails Simple** — two unlike mechanisms. One is a parse-slot peel; the other is *generics for
lambdas*: populating `Function.type_params` (hardcoded empty) and instantiating it per call site.

**E₀-b's Obvious is measured:** all 40 sites are `defn` declarations, and **zero anonymous parametric
`fn`s exist because the form is currently unwritable** — no name slot, no binder. The builder's
anonymous spelling is a NEW capability, not a migration target. It is named to **γ-i-b**, never
deferred.

### E — where the binder is consumed.  **RULED: E2**

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **E1** the `defn` macro peels `:- [T U]` and re-emits `:user::f<T,U>` | YES | YES | **NO** | — |
| **E2** `def` accepts the binder beside its name; `defn` forwards it there | YES | YES | YES | YES |
| **E3** both | **NO** | **NO** | YES | — |

**E1 fails Honest, and the expansion is the proof** — it makes our own macro MANUFACTURE
`:user/f<T,U>`, the exact spelling ③ makes illegal. Blocker 3's disease (`defservice` emitting the
retired form so a migrated corpus regrows it) adopted deliberately, in the stone whose job is to
retire it. ③ would then break on names our own macro minted. E1 is seductive precisely because
`defn` already carries the `name-base`/`name-tp` split — machinery being there is a reason it would
be cheap, never a reason it would be right.

**E3 fails Obvious and Simple** — two peels for one spelling; a reader must work out which fires.

**E2 is where the params already attach** — the expansion shows them riding the `def` name, so the
binder belongs where the params land.
