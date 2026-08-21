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

> ⛔ **CORRECTED 2026-08-21 after flight 1.** The original row 2 demanded that ONE let-bound value
> apply at TWO different types. That is **let-polymorphism** — HM `let`-generalization, which needs
> `locals` to hold `TypeScheme` instead of `TypeExpr` — and it has nothing to do with whether an
> anonymous `fn` can declare its type params. I made it ★ load-bearing; it fired STOP-2 and killed a
> stone that was already delivered. My justification (*"one instantiation proves nothing"*) was a
> correct concern behind a wrong vehicle: **a rigid `:T` and a missing let-generalization fail that
> row identically**, so it could never say which one it had found.
> `[[feedback_a_gate_must_fire_the_mechanism_the_way_production_fires_it]]`

The non-vacuity question is *"is `X` a real variable?"*, and it is answered WITHOUT let-polymorphism
by whether `X` unifies across positions. Measured against flight 1's tree:

| # | row | expected | flight 1 |
|---|---|---|---|
| 1 | `(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)` | checks | ✅ |
| 2a | `(fn :- [X] [a <- :X b <- :X] -> :X …)` applied `(f 1 2)` | checks | ✅ |
| 2b | …applied `(f 1 "s")` | **REJECTS** — `X` unifies across positions | ✅ |
| 2c | …applied `(f "p" "q")` | checks — `X` is not pinned to the first use | ✅ |
| 2d | …`(takes-str (f 1 2))` | **REJECTS** — the return is tied to `X` | ✅ |
| 2e | the same fn passed DIRECTLY to a generic HOF | checks | ✅ |
| 3 | a decl carrying BOTH `<T>` and `:- [T]` | a located contradiction ERROR | ⛔ silently checks |
| 4 | a no-param-list `defn` stays generic (251.7 must not regress) | checks | ✅ |
| 5 | the concrete-type HOF control (probe rung 4) | checks | ✅ |
| 6 | a parametric **kwargs** `defn` in binder spelling | checks | ⛔ *"triple is incomplete"* |
| 7 | a **variadic** `defn` in binder spelling | registers | ✅ |
| 8 | `def` of a non-fn value; `git diff --stat src/check.rs` EMPTY | registers; zero changes | ✅ |

★ **Rows 2a-2e together are the non-vacuity gate.** 2b and 2d are the ones that bite: they
distinguish a real type VARIABLE from a rigid Path *and* from an unconstrained wildcard, which a
single application cannot.

### Row 3 — the message already exists; mirror it

The TYPE side emits it today: *"declaration carries BOTH a name-embedded `<...>` type-param spelling
… and a `:- [...]` binder — pick one; a declaration with both is a contradiction, never something to
silently resolve."* `try_parse_fn_shape_def` returns `Option`, so it has no channel to carry a
located error — that is the work.

### Row 6 — and the `wat/core.wat` edit IS needed, for the kwargs branch

Flight 1 measured that `defn`'s macro already forwards a stray `:- [T]` into the emitted `fn`, so no
edit is needed for FORWARDING — my original sketch item was invented, and I deleted it. But an A/B
with an identical argspec shows the kwargs path does break:

```
(defn :user::mk  :- [T] [a <- :T & [b <- :T]] …)  ⛔ "malformed :wat::core::fn form: triple is incomplete"
(defn :user::mk2<T>     [a <- :T & [b <- :T]] …)  ✅ checks          ← same argspec, control
```

`defn`'s kwargs branch keys on `name-parametric?` / `name-tp`, the string suffix taken off the NAME,
which the binder spelling leaves empty. **So the edit returns, scoped to the kwargs branch only.**

## Out of scope, by MEASUREMENT — named, never deferred

- **let-polymorphism.** `check.rs:11757` infers a `let` RHS once and stores a `TypeExpr`;
  `check.rs:2065` clones it per reference; `derive_scheme_from_function` (`:15977`) is gated
  `func.name.as_ref()?` — anonymous fns get no scheme, by design and by its own doc. Making a
  let-bound value polymorphic is its own arc. Until then an anonymous fn is monomorphic **at its
  binding**, which is ordinary for ML-family languages lacking `let`-generalization.
- **The anonymous-`fn` silent-accept.** `infer_fn`'s `SigParse::SilentReject` returns
  `CheckResult::ok(fresh.fresh())`, so ANY junk in the first slot — `:foo`, `42`, `"s"` — makes the
  whole fn unconstrained and every call to it check vacuously. Measured on the RELEASE binary from
  before flight 1: **pre-existing on `main`, reachable by a typo, and silent.** Its own stone.

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
