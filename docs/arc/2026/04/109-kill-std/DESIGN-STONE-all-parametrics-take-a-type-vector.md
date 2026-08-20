# DESIGN — arc 109: ALL PARAMETRICS TAKE A TYPE-VECTOR. The angle form dies.

Opened 2026-08-20 by builder ruling. Home is 109 because 109's purpose is killing the dialect we
bootstrapped with, and the angle form is the last large piece of it. 285 and 255 resume after.

> *"i think we just do a hard, decisive migration that breaks a ton of shit … all parametrics share
> one form … let's make the hard cut now that all parametrics must have their types expressed as a
> vec of types."*
>
> *"this is a lot of work, but we do not fear it. we fear being dishonest, incoherent — we know how
> to do this. slow is smooth, smooth is fast. we strike to kill."*

## The rule

**Every parametric — type or literal — is `(Head [type…])`.** One shape, nesting uniformly:

```wat
;; TYPE (after <- or ->, or in a declaration slot)
(:wat::core::Tuple [:wat::core::i64 :wat::core::String
                    (:wat::core::HashMap [:wat::core::Keyword
                                          (:wat::core::HashSet [:wat::core::f64])])])

;; EMPTY LITERAL — the same form, anywhere else
(:wat::core::HashMap [:wat::core::Keyword :wat::core::String])

;; LITERAL WITH VALUES
(:wat::core::HashMap [:wat::core::Keyword :wat::core::String] :some-kw "some-str")
```

The `wat.type/HashMap` Clojure spelling comes LATER; this stone keeps the rust-ish FQDN head and
changes only the shape of the type-arg group.

## ★ The cut is SYNTACTIC, not semantic — measured, at HEAD

Position already decides type-vs-literal. The identical form, unchanged, today:

```
(:wat::core::HashMap :wat::core::String :wat::core::i64)   in value position  →  {}
                     …the same form after `<-`             as a TYPE          →  accepted, length 0
```

So the builder's rule is **already the substrate's semantics**; only the type-arg grouping moves.
Nothing can quietly change behaviour while ~3,236 sites churn — the breakage is loud and inert. That
is the safest possible shape for a total migration, and it is why this is a killshot rather than a
gamble.

The 2026-08-15 addendum to `NOTE-typed-literal-constructors.md` already closed the "is it ambiguous?"
question at the grammar level: a type is reachable by exactly one production, so annotation-vs-literal
has no ambiguous form. This stone inherits that closure; it does not re-open it.

## Scale, measured this session

```
wat/*.wat        821 in  26 files      ← the stdlib. loaded+checked at startup. see BOOTSTRAP.
wat-scripts/    1228 in 232
tests/*.wat      489 in 213
src/*.rs         566 in  70            ← Rust string literals; NOT wat-fix's domain
tests/.rs/.edn   132
                ─────
              ~3,236 sites  ·  977 comma-bearing
```

Type position is **always marked**, so the codemod cannot guess wrong. Classifying every parametric in
`wat/` by the token immediately before it: **464 after `<-`, 276 after `->`**, the remainder after a
declaration head (`defn` / `typealias` / `defsurface` / `defenum` / `defstruct` / `defrecord` /
`:satisfies`) or nested inside another type. No bare parametric sits in an unmarked position.

## ★★ THE PAYOFF IS NOT SYNTAX — IT IS THE DELETION OF A FAILURE CLASS'S APPARATUS

Three rooms hold the angle form. The second is the reason to do this at all.

**Room 1 — birth.** `crates/wat-reader/src/lexer.rs:792` and `:942`. The lexer swallows `<…>` INTO the
keyword token, tracking `angle_depth` and deliberately retaining commas. `HashMap<K,V>` is ONE token.

**Room 2 — the tear.** Because the type args live inside a string, every consumer must re-split it.
`split_type_list_top_level` (`types.rs:4858`) is 40+ lines of hand-rolled depth tracking that exists
for exactly one reason, in its own words: *"a flat `split(',')` tore `State<K` / `V>` apart and shifted
every subsequent type-arg by one."* Three flat splits still slice the inside of a `<…>`:

```
src/types.rs:4293          inner = &params_part[1..len-1]  → .split(',')
src/types/surface.rs:212   inside = &name[lt+1..len-1]     → .split(',')
src/runtime.rs:4176        inside = &kw[lt+1..len-1]       → .split(',')
```

⚠ Whether those three are reachable with nested args is UNMEASURED — they appear to split declaration
params, which are bare identifiers. **The stone must not claim they are bugs without proving it.**

**With the bracket, the READER returns a vector of forms. There is no string, nothing to split, and
the whole apparatus — the splitter and every caller — becomes deletable.** CLAUDE.md names this the
recurring class ("suspect a string comparison with one side normalized and the other not"); three
instances in arc 278 alone. This cut removes the ground it grows in. That is the extirpare top rung:
not a check that catches the tear, a shape in which the tear has no form.

**Room 3 — speech.** `check.rs:15619`, `format_type_inner`'s Parametric arm (`format!("{}<{}>")`) and
its `format_type` twin. One site each. See the pinned decision.

## ⊹ RULED 2026-08-20 — RENDERING FOLLOWS. THE ANNIHILATION IS COMPLETE.

> Builder: *"this move is the annihilation of angle brackets from wat…. they are gone, completely….
> they must be annihilated…. we've been dragging our feet on this for a long time…. they die this day."*

"Completely" settles the open this section used to hold. A `Vector<T>` rendered into a diagnostic IS
an angle bracket in wat; a compiler that PRINTS a syntax it REFUSES TO READ is the incoherence this
arc exists to end. So `format_type` / `format_type_inner` (`check.rs:15619`) emit the bracketed form,
and the **113 angle-form occurrences across 29 `tests/**/*.edn` goldens** move with them.

## ⛔ THE SCOPE BOUNDARY — NOT EVERY `<` IS AN ANGLE BRACKET

This is the number that defines the strike, measured across `wat/` + `tests/**/*.wat`:

```
must DIE    parametric  Head<…>                        ~3,236 sites
must LIVE   <- (5261) · -> (4446) · > >= (87) · <  (81)
            · <= (23) · :-> (14)                        9,912 sites
```

**A textual `<` → `[` sweep destroys the corpus.** The migration is a FORM-AWARE wat-fix codemod
(R21) and nothing else.

**The discriminator is already written — it is the lexer's own rule** (`lexer.rs:792`, `:942`), and the
codemod must use the same one or it is guessing: inside a keyword token, `<` preceded by `::` is the
**operator** (`:wat::core::<`); `<` preceded by alphanumeric / `_` / `'` **opens type params**
(`Vector<`, `Thread'<`). `<-` / `->` / `:->` are separate tokens entirely and never enter a keyword.

⚠ The angle form lives INSIDE a keyword token, so the rewrite is a NODE REPLACEMENT — one `Keyword`
node becomes a `List` node — not a text substitution within a node. wat-fix's span-faithful
`ast-span` + `fix-text-apply` is the mechanism.

**Head census (head-by-codemod, tail-by-name):** `Vector` 674 · `PersistentVector` 232 · `Peer` 119 ·
`Option` 98 · `Stream` 63 · `Seqable` 63 · `HashMap` 60 · `Address` 56 · `Result` 48 ·
`ThreadSelfPeer` 42 · `Peer'` 29 (⚠ the primed head — `'` before `<` is legal and must survive).

## ⊹ WHAT THIS STONE IS NOT — the second hard problem is NOT here

> Builder: *"this removes one of two hard problems for the clojure flip…. the second one is the
> annihilation of illegal keywords…. `:wat::core::+` is illegal, `wat.core/+` is not… we are not
> attacking keywords here.. we are attacking angle brackets."*

**Illegal keywords are a SEPARATE strike and are out of scope here.** This stone does not touch the
`:wat::core::` spelling, does not rename a single head, and does not approach the 6,552 distinct
colon-quoted spellings the 255 seam counts. It changes ONE thing: how a parametric's type args are
grouped. Keeping these apart is what makes each survivable.

## ⊹⊹ RULED 2026-08-20 — THE TYPE VECTOR IS MANDATORY. NO INFERENCE.

> Builder: *"yes — the vec-of-types param is mandatory for any parametric type. no inference — no
> ambiguity — you say what it is. **the verbosity is our shield** — we are optimizing this for LLMs
> not humans — when ambiguity is annihilated there's no guess work… same reason why everything is
> fqdn all the time."*

This is the project's own standing doctrine, not a new one: `wat/rete.wat:661` refuses to collapse its
axis chain for the same reason — *"verbosity is our shield (R63/R65: the same exhaustiveness we pay
for in keystrokes is what lets us change meaning later)"* — and FQDN-everywhere is arc 109's thesis.

**Consequence, measured across `wat/` + `tests/` + `wat-scripts/`:**

```
2,138 constructor call sites
  851  already carry a leading type keyword   → pure mechanical bracket insertion
       Vector 702 · HashMap 149
  977  carry NO type at all                   → ★ T must be INFERRED to write the bracket
       PersistentVector 683 · Tuple 286 · PersistentMap 8
```

The 977 are a different kind of migration from the angle cut. `Head<K,V>` → `(Head [K V])` is a pure
shape rewrite — the types are already on the page. `(PersistentVector 1 2 3)` →
`(PersistentVector [:wat::core::i64] 1 2 3)` requires *inferring* `i64`. **A form-aware codemod cannot
do that.** wat-fix walks forms; it does not type-check.

## ★★ THE 977 BECOME MECHANICAL: THE CHECKER'S REFUSAL CARRIES THE TEXT TO INSERT

Do NOT build a checker-driven rewriter. The checker already holds the answer at the exact moment it
would refuse:

- `infer_persistentvector_constructor` computes `t_ty` from the elements. `infer_tuple_constructor`
  computes a type per position. `infer_persistentmap_constructor` computes K and V.
- `CheckErrorKind::MalformedForm` already carries a structured **`remedies`** field
  (`crate::remedy::Remedy`, `check.rs:378 remedies_for` / `type_error_remedies`) — errors in this
  substrate already ship writable fixes, not prose hints.

So at ③, a bracket-less constructor is an error **whose remedy is the literal text to write**:

```
:wat::core::PersistentVector requires a type vector; it was inferred as [:wat::core::i64]
  remedy → (:wat::core::PersistentVector [:wat::core::i64] 1 2 3)
```

★ That converts 977 judgment-bearing sites into 977 **mechanical** ones. The compiler does not merely
name the sites — it writes the fix. This is `SUBSTRATE-AS-TEACHER` at its sharpest: the fail-count is
the worklist, and each entry arrives pre-filled.

⚠ **Two things to measure before relying on it**, neither assumed here:
1. That the inferred type is always *writable* — a fresh unresolved type variable has no spelling, and
   an empty `(PersistentVector)` infers nothing. Those sites need a human or a declared default.
2. That the inferred type is the one the author would have written. An inferred concrete type where a
   supertype was intended ships silently, because the checker then accepts its own guess. This is the
   ONLY place in the whole migration where a wrong answer can pass green — treat it as the risk.

## The strike order — the cut is TOTAL at ③

```
①  the checker/reader ACCEPTS (Head [types])     both forms briefly legal
②  codemod the corpus                             wat-fix for .wat (R21) + a separate src/*.rs pass
③  the angle form becomes ILLEGAL                 lexer machinery deleted; every heretic screams
```

**Why ① and ② exist, and why that is not foot-dragging:** `wat/*.wat` IS the stdlib, loaded and
type-checked at startup. A checker that rejects `<>` before `wat/` is migrated means nothing loads —
including the wat-fix codemod that performs the migration. That is burning the farm while holding the
only tool inside it, and `wat/fix.wat`'s header BOOTSTRAP / STASH-DANCE note exists for precisely this
shape. The cut lands undiluted at ③.

⚠ **`.wat` migration is a wat-fix codemod. R21, non-negotiable** — never hand-edits, never python/sed.
`src/*.rs`'s 566 sites are string literals outside wat-fix's reach and are their own strike, with the
goldens riding on the rendering decision.

★ Step ③ is `SUBSTRATE-AS-TEACHER` (FM 15): the fail-count is the progress meter, each error names the
next site, and the waterfall runs to zero. The builder pre-expects the count and does not need
protection from it.

## Downstream — what this unblocks

- **The Clojure flip becomes a HEAD RENAME over an already-correct tree.** `(:wat::core::HashMap […])`
  → `(wat.type/HashMap […])` is one symbol swap, zero structural change. The seam's hazard — *"after
  the flip `(f HashMap<K,V>)` reads as VALID EDN and silently changes arity 2→3"* — **has no form**,
  because `(Head [types])` is always a 2-element list whatever N is. All 977 comma-bearing sites stop
  being a danger set. #110 still gates the corpus flip; it does not gate this stone.
- **The typed literal constructor lands with it** — `(HashMap [i64 Record] 0 n0 1 n1)` finally lets a
  literal declare a supertype for heterogeneous values, the gap
  `109/NOTE-typed-literal-constructors.md` filed as having no working form.
- **285's constructor parity falls out free.** `HashMap` requires leading `K V` today and
  `PersistentMap` refuses them — measured, both directions. One shape ends that fracture, which is
  285's honesty clause.
- **`300/NOTE-the-type-converter-emits-the-superseded-form.md`** — `edn_shim.rs:1249-1253` (the `TypeExpr::Parametric` arm) splices args flat
  and must emit the bracket. Named blocker for 300.1; closed by this stone.

## The four questions

- **Obvious?** YES — one shape for every parametric, at every depth, in both positions.
- **Simple?** YES — and it SUBTRACTS: the lexer's angle machinery and the depth-aware splitter both go.
- **Honest?** YES at ③, and only at ③ — while both forms are legal the language has two spellings for
  one thing, which is the state this stone exists to end. That is why ① and ② are steps, not a resting
  place.
- **Good UX?** YES — a delimited group the reader splits, instead of a string every consumer re-splits.
