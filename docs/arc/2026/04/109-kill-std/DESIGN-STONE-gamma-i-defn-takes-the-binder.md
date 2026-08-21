# DESIGN — arc 109 γ-i: `fn` takes the `:- [T …]` binder

> **RULED, builder 2026-08-21.**
> **D1** — γ-i goes FIRST, before the identity stone.
> **G3** — **`fn` carries the binder; `def` derives.** `defn` forwards it into the emitted `fn`.
>
> ⚠ **RULING HISTORY — two premises expired under measurement, and both corrections came from the
> builder's questions, not from my checking.** Kept visible because the lesson is the point.
>
> | ruled | superseded by | what the measurement said |
> |---|---|---|
> | **E2** `def` consumes the binder | **G3** | Stone 251.7 already unions the fn signature's free type-vars into the def's scheme, so the name-embedded list is nearly vestigial — and there are ZERO parametric `def`s whose value is not an fn. The binder belongs to the thing that is generic. |
> | **E₀-b** the anonymous `fn` binder is a separate stone (γ-i-b) | **G3** | Under G3 the anonymous capability IS the change. γ-i and γ-i-b collapse into one. |
>
> The first draft of this DESIGN also named `fn` — for the WRONG reason (it reasoned from the error
> message). The macroexpansion corrected it to `def`; the measurements below corrected it back to
> `fn`. **Landing on the right answer by a wrong route is not the same as being right.**
> `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

## The spec, from the builder

```clojure
[:-> X]              0-arity, produces X    ⇔  (wat.core/fn :- [X]           []                :- X …)
[A :-> X]            1-arity                ⇔  (wat.core/fn :- [A X]         [a :- A]          :- X …)
[A B :-> X]          2-arity                ⇔  (wat.core/fn :- [A B X]       [a :- A b :- B]   :- X …)
[A B C D E :-> X]    5-arity                ⇔  (wat.core/fn :- [A B C D E X] [a :- A …]        :- X …)
```

**The binder lists every type var, the return's included** — `[:-> X]` binds `X` though `X` appears
nowhere but the return. Arity is **structural** (the position of `:->` IS the arity), which is why
nullary needs no special case: probed, `[:-> :wat::core::Record]` checks clean.

An occurrence in RETURN position is consumption. `check_type_params_consumed`'s `Surface` arm already
walks Method args AND ret; `fn` mints no `TypeDef`, so the wall never reaches it. Nothing to change.

## The gap, measured

```
(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)
  ⛔ "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got keyword"
(:wat::core::defn :user::f<T>   [x <- :T] -> :T x)                                    ✅ clean
```

Every other head in the codemod's declarator list ACCEPTS the binder — probed one by one: `defenum`,
`defrecord`, `holon::defrecord`, `defstruct`, `defsurface`, `typealias`, `newtype`, `typeunion`.
**`defn` alone rejects.** 40 parametric `defn`/`fn` in `wat/`, 57 corpus-wide, every one already
rewritten into the binder form by the COMMITTED codemod.

## ★ The four measurements that decided G3

**1 — the expansion.** `defn` is sugar; the params ride the def NAME, and `fn` gets none:

```clojure
(:wat::core::defn :user::f<T,U> [x <- :T y <- :U] -> :T x)
  →  (:wat.core/def :user/f<T,U> (:wat.core/fn [x <- :T y <- :U] -> :T x))
```

**2 — `def` already hands the params to the fn, by CONSTRUCTION.** `try_parse_fn_shape_def`
(`src/runtime.rs:3395`) reads them off the name and then, **Stone 251.7**, unions them with every free
type-var in the fn's signature before stamping `Function.type_params`. The `fn` FORM never carries a
list; the `def` PATH builds the `Function`.

**3 — so the name-embedded list is already nearly vestigial.** Probed: a `defn` with **no param list
at all** is generic and instantiates at two types.

```clojure
(:wat::core::defn :user::b [x <- :T] -> :T x)   ;; ✅ generic; applied at :i64 AND :String
```

**4 — and the ANONYMOUS path is rigid.** `src/function/eval.rs:66` hardcodes
`type_params: Vec::new()` with no union, so an anonymous fn's `:T` is a concrete Path:

```
(:wat::core::fn [x <- :T] -> :T x)  applied to 1
  ⛔ "(value head): parameter #1 expects :T; got :wat::core::i64"
```

**Plus: ZERO parametric `def`s whose value is not an fn**, corpus-wide. Only functions are generic —
so the binder belongs on the function, and `def` needs nothing.

## What G3 makes GO AWAY

The E2 design required widening `def`'s `(name [meta] expr)` shape, hand-rolled with a
`len() != 3 && len() != 4` guard in SEVEN places — `check.rs:545,8445` and
`runtime.rs:1291,2649,3395,3551,3671` — **every one of which treats an unexpected arity as
"malformed; the type checker already caught it" and silently `continue`s.** A 5/6-item `def` one
guard had not learned would have been a binding that never registers, with no error.

**Under G3 `def`'s shape is untouched.** That hazard, and the `split_def_form` consolidation invented
to contain it, both drop out. `src/check.rs` leaves the blast radius entirely.

## The mechanism

```
wat/core.wat            the defn macro peels `:- [T …]` after the name and forwards it INTO the
                        emitted `fn` (not the `def`); name-tp/name-base learn the binder spelling
src/function/metadata.rs  a binder peel beside peel_metadata_preamble — same shape, same two callers
src/function/eval.rs:42   peel, then stamp Function.type_params (today: hardcoded Vec::new())
src/function/infer.rs:105 peel, and GENERALIZE — see the risk below
src/runtime.rs:3395     try_parse_fn_shape_def reads the fn's binder and unions it exactly where
                        251.7 already unions the signature's free vars; :3551 / :3671 likewise
```

⚠ `parse_fn_signature_prefix` (`src/function/parse.rs:145`) takes **`&[WatAST; 3]`** with its own
doc — *"Arity is type-guaranteed — no runtime arity check required"* (Stone 243.4.1,
`CONFORMARE.md`'s worked example). **The binder is peeled BEFORE that slice**, exactly as metadata
already is, so the wall stands untouched. Do not widen it.

## ⛔ THE RISK — `infer_fn` builds no scheme at all

`src/function/infer.rs` binds the params into `body_locals` and checks the body. **There is no
generalization step**, which is precisely why an anonymous `:T` is rigid today. G3's anonymous
capability is therefore NOT just a peel: the check side must introduce type VARIABLES for the
binder's names and produce a polymorphic result at the binding site.

**This is the stone's real content and its load-bearing acceptance row.** It is also why G3 subsumes
γ-i-b rather than merely enabling it.

## Acceptance — the rows that must be PROVEN, not parsed

```clojure
(:wat::core::defn :user::fold<T,U>     [f <- [U T :-> U] init <- :U] -> :U …)   ← control
(:wat::core::defn :user::fold :- [T U] [f <- [U T :-> U] init <- :U] -> :U …)   ← subject
```

1. Both spellings check, and the body resolves `:T` / `:U` in each.
2. BOTH on one declaration → a contradiction ERROR, never a silent pick. (`take_declared_binder`'s
   own rule: *"a contradiction, never something to silently resolve"*.)
3. ★ **The load-bearing row** — an ANONYMOUS binder-carrying fn, bound in a `let`, applied at **TWO**
   types: `(:wat::core::let [f (:wat::core::fn :- [T] [x <- :T] -> :T x) _ (f 1) __ (f "s")] …)`.
   One instantiation proves nothing; two is what separates generalized from rigid.
4. A `defn` with **no** param list stays generic (the 251.7 behaviour must not regress).
5. A parametric **kwargs** `defn` in binder spelling mints `Kwargs<T,U>`, not a monomorphic bundle —
   `defn` derives `{b}::Kwargs{p}` from `name-tp`, the string suffix off the NAME, which the binder
   spelling empties. **Zero instances in `wat/`** — an acceptance row precisely because it is not a
   census. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`
6. A **variadic** `defn` in binder spelling registers — the `try_parse_*_variadic_def_fn_form` paths.
7. A `def` of a NON-fn value still registers, and `def`'s arity is UNCHANGED — the negative control
   proving G3 did not quietly widen `def`.

## Blast radius

`wat/core.wat` · `src/function/{metadata,eval,infer}.rs` · `src/runtime.rs` (the three `def`-fn
recognizers). **NOT `src/check.rs`. NOT `parse_fn_signature_prefix`'s `[WatAST; 3]`.**
**No `.wat` corpus migration** — the 40 sites keep `<T,U>` until ②-iii re-runs.

⚠ A stdlib `.wat` edit is INVISIBLE until a rebuild and **a rider cannot test one**. The
`wat/core.wat` half is rider-edits, orchestrator-builds-and-floors.

## The four questions

### On γ-i itself

- **Obvious?** YES — `defn` is the only declarator head refusing the operator every other head takes,
  and the committed codemod already writes the binder at 40 stdlib sites.
- **Simple?** YES — one peel, at the place metadata is already peeled, plus the generalization the
  anonymous path has always lacked.
- **Honest?** YES — the alternative (dropping `defn` from the codemod's head list) renders a
  declaration NAME as a REFERENCE, the silent corruption `a9168b851` exists to prevent.
- **Good UX?** YES — it lands and floors independently of the identity stone, and it makes the
  builder's own anonymous spelling legal.

### G — which form carries the binder.  **RULED: G3**

| | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **G1** `def` only (the superseded E2) | YES | YES | YES | **NO** |
| **G2** both; `defn` emits to each | YES | **NO** | **NO** | — |
| **G3** `fn` only; `def` derives — which 251.7 already does | YES | YES | YES | YES |

**G1 fails UX** — it leaves the builder's `(fn :- [A B X] …)` illegal, and pays the seven-guard
silent-skip hazard on `def`'s arity to add a list the signature already yields.

**G2 fails Simple and Honest** — two lists for one fact, with `(def :f :- [T] (fn :- [U] …))` a
REPRESENTABLE DISAGREEMENT. A structure that can hold two contradicting bindings can lie about what
is bound; it is the exact state `take_declared_binder` refuses by erroring when both spellings appear.

**G3** puts the binder on the thing that is generic — measured: only functions are — keeps ONE source
of truth, leaves `def` untouched, and collapses γ-i-b into this stone.
