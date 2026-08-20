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

## ⛔ THE ONE CONTRACT DECISION — does RENDERING follow the source syntax?

113 angle-form occurrences live in `tests/**/*.edn` goldens across 29 files (`Vector<T>` 18,
`Option<T>` 17, `Tuple<T…>` 12, `Result<T,E>` 12, `HashSet<T>` 12, `HashMap<K,V>` 12 …), mostly as
generic shape-prose inside error messages rather than concrete renderings.

- **Rendering follows** → diagnostics print `(:wat::core::Vector [:wat::core::i64])`. 113 golden
  occurrences move; every type-bearing message gets longer.
- **Rendering stays angled** → goldens barely move, and the compiler **prints a syntax it refuses to
  read**. That fails Honest outright.

**RECOMMENDATION: rendering follows.** A language that speaks a dialect it will not accept is exactly
the incoherence this arc exists to end, and the 113 are mechanical. ⚠ NOT YET RULED by the builder —
this stone does not proceed past step ① without that call.

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
