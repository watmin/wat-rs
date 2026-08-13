# NOTE — parametric type FORMS already parse; only the `[…]` bracket is missing

**Filed 2026-08-13, arc 278. MEASURED, not proposed.** Six probes, each with controls. Answers a
question three arc-109 notes left open and materially shrinks the type-syntax pivot.

Read first: `109/NOTE-generic-bracket-syntax-edn.md`, `109/NOTE-typed-form-and-type-namespace.md`,
`109/NOTE-typed-literal-constructors.md` — the three interlinked notes this measures against.

## What those notes assumed, and what is actually true

They framed parametrics-as-forms as a **destination to be built**: delete `lex_keyword`'s bracket
machinery, design a grammar, land it with the keyword-as-type retirement. The measurement says the
bare-arg form is **already standing and genuinely checked**, in every position tried:

```
:wat::core::Vector<wat::core::i64>                    exit 0  "3"   the existing angle form
(:wat::type::Vector :wat::core::i64)      PARAM       exit 0  "3"   ★ list form works
(:wat::type::Vector :wat::core::i64)      RETURN      exit 0  "3"
(:wat::type::Vector :wat::core::i64)      FIELD decl  exit 0  "2"
(:wat::type::Vector (:wat::type::Vector :wat::core::i64))  NESTED  exit 0  "1"
```

**NOT VACUOUS — two controls fired:**

```
(:totally::bogus::Head :wat::core::i64)   exit 3  UnresolvedReference   ← the head is RESOLVED
(:wat::type::Vector :wat::core::String)   exit 3  CheckErrors           ← the arg is TYPE-CHECKED
```

A bogus head is rejected and a *wrong element type* is rejected, so the greens above are real
acceptance, not a parser waving list forms through. Grounded further: `types.rs:4439`/`:4488`/`:4694`
already handle `:wat::type::Vector` post-normalize, and `wat.type/i64` (Symbol) normalizes to
`:wat::type::i64`.

## The ONE gap — the `[…]` type-param bracket

The 2026-07-24 refinement in `NOTE-typed-literal-constructors` brackets the type-params
(`(wat.type/HashMap [K V])`) so one head serves annotation AND construction. **That form is refused:**

```
(:wat::type::Vector [:wat::core::i64])
  → MalformedForm: "malformed :wat::core::fn form: invalid type keyword: malformed type expression"
```

So the work is not "build type-forms." It is **add the bracket** to a grammar that already parses,
resolves, checks, and nests.

## ⚠ AND A DIAGNOSTIC DEFECT FOUND WHILE MEASURING IT

The bracket's real error is **suppressed whenever the definition has a caller.** Same file, one line
changed:

| | reported |
|---|---|
| malformed `defn` **with** a caller | `UnresolvedReference :user::sum` — "call head — not a builtin, not a registered function", pointing at the **CALL SITE** |
| malformed `defn` **without** a caller | `MalformedForm … invalid type keyword` — **located, at the real cause** |

The located diagnostic EXISTS. Resolve runs before check and short-circuits, so a caller's unresolved
reference is reported *instead of* the cause. This is `109/NOTE-a-malformed-definition-must-not-vanish.md`
with a twist — the definition does not vanish, its **diagnostic** does — and it is the same shape as
24z's `defclause`-metadata finding (*"you learn at a CALL SITE as an unresolved reference, pointing at
the caller, not the cause"*).

**Load-bearing for the migration:** during a corpus-wide flip, most malformed definitions WILL have
callers. Every one of them will report the wrong location. FIXED 2026-08-13 in `5e8eeb84` — resolve is
deferred so `check_program` runs first and a located cause outranks the downstream symptom (narrowed:
an `UnknownCallee` IS the resolver's own finding restated, so only a DIFFERENT cause outranks).

## ⚠ CORRECTION — the `[…]` bracket is NOT "already claimed". There is NO collision.

The un-hidden diagnostic reads *"function-type bracket needs a `:->` arrow: `[arg… :-> ret]`"*, and this
note's author first read that as **the `[…]` bracket being taken by function types**, i.e. a collision
that would force the 07-24 `[type-params]` grammar to move. **That was wrong.** Measured:

```
[A :-> B]                    exit 0   bare bracket standing alone as the type = FUNCTION TYPE, works
[A B]                        exit 1   bare bracket, no arrow — correctly refused
(:wat::type::Vector [A])     exit 1   SAME error — the parser applies its ONLY bracket rule
```

The parser has exactly ONE rule for `[` in type position. Meeting a bracket as a PARAMETRIC HEAD'S
ARGUMENT, it applies that rule and fails. **Unimplemented case, not taken syntax.** The two are
distinguished by POSITION, exactly as annotation-vs-construction is:

```clojure
[A :-> B]                  ; bracket STANDING ALONE as the type   -> function type
(wat.type/HashMap [K V])   ; bracket as the ARGUMENT of a head    -> type-param list
```

**The proposed grammar needs no change.** The parser needs a rule for bracket-in-parametric-arg-position.

★ This is `[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]`, THIRD instance in one day
(after `LOST disconnected` and the `where`-fence's first-failing-axis). The message reported the
ASSUMPTION THE PARSER MADE — "I took this for a function type" — not a declaration that the bracket is
reserved. Corrected by the builder refusing the premise: *"what collision did we just encounter?"*

## The ambiguity the notes flagged is NOT a blocker

`NOTE-typed-literal-constructors`'s addendum names a "genuine open": `(wat.type/HashMap [K V])` being
ambiguous between a type annotation and an empty typed literal. **Position resolves it, and the
language already works this way** — a type expression appears after `:-`/`->`/in a field decl; a value
expression appears in argument/binding/body position. Those sites never overlap. Arc 242's Doctrine 1
(*"a `:type` keyword is not a value"*) is already a position rule; Typed Clojure's `(t/Vec t/Num)` is a
type only because an annotation site reads it.

It is also solved for the CODEMOD specifically: position is a **fact** (`parent`, `index`, parent-head),
which `rules-corpus-03` extracts from real source and `rules-corpus-01` joins on.

**The real residue is a diagnostics requirement, not a grammar ambiguity:** zero-arg construction and
annotation are textually identical, so anything that *reports* on such a form must carry position to say
which it means.

## What this does to the pivot's cost

| piece | state |
|---|---|
| parametric types as forms | **already works** — 4 positions + nesting, controls armed |
| `[…]` type-param bracket | **the gap** — one parser change |
| typed literal constructor | measured gap: `wat.type/` refs "recognized but non-functional", accepted without driving V (109 note) |
| `wat.type/` namespace population | rename cascade; UNMEASURED whether the namespace is populated or aliasing |
| dotted surface | last |
| `<>` deletion | at zero offenders |

It also cheapens the transition strategy the arc-109 notes worried about. They called an interim
"scaffolding we are about to delete" — but the new form is **not new**, so "accept both, migrate, then
hard-disable `<>`" is not scaffolding; it is one already-parsing grammar plus a bracket.

## Bounds on this measurement

- Tested `:wat::type::Vector`. **UNMEASURED:** whether `:wat::type::` is a populated namespace or that
  name happens to alias `:wat::core::Vector`. Check before relying on `wat.type/` as a namespace.
- Tested param / return / field / nested. **UNTESTED:** `ann-form` expression position, fn types
  (`[A -> B]`), user records, `defsurface` `:features`, and the constructor (value) position.
- The `[…]` refusal is characterised by its message, not by reading the parser. The mechanism behind
  "invalid type keyword" is unread.

## ★ FUNCTION TYPES — the replacement is ALREADY LIVE, and the two spellings are ONE type

Measured 2026-08-13. The long-symbol `Fn(A,B)->R` is not a syntax needing a designed replacement;
it is a DUPLICATE THAT LOST. Both spellings run today, same 3-arg function, all returning "7":

```clojure
[f <- :wat::core::Fn(wat::core::i64,wat::core::String,wat::core::bool)->wat::core::i64]  ; 107 sites
[f <- [:wat::core::i64 :wat::core::String :wat::core::bool :-> :wat::core::i64]]         ;  27 sites
[f <- [wat.type/i64 wat.type/String wat.type/bool :-> wat.type/i64]]                     ; ★ the DOTTED
```

The third line is the destination surface — dotted symbols, whitespace-separated, EDN-clean — and it
is **already in the corpus and already runs**. Nothing is waiting on anything.

### The `:->` is a PIVOT, not a separator — edges measured

```
[:-> R]              exit 0   ZERO args; the pivot may start the vector
[A :-> R]            exit 0
[A B C D E :-> R]    exit 0   arity is FREE — no separator to run out of
[A :->]              exit 1   no return — refused
[A :-> R :-> S]      exit 1   two pivots — refused
[A :-> R S]          exit 1   two returns — refused
```

Exactly one pivot, exactly one type after it, any arity before. That is why it scales where
`Fn(...)` cannot: adding an argument is a symbol and a space, versus cramming another
forbidden-whitespace comma INSIDE a single keyword token.

### ★ AND THE TWO SPELLINGS UNIFY — this is the load-bearing result

A parameter declared with the BRACKET form accepts a function passed through a parameter declared
with the `Fn(...)` form, and the reverse, in one program (`"5"`, both directions). **They are ONE
`TypeExpr` with two surfaces.**

Consequence, and it removes the risk this note was about to raise: **the function-type flip is a
codemod over TEXT, not a substrate migration.** No downstream Rust `match` arm can go quiet after
it, because there is no second representation for it to stop seeing. Arc 109's recorded trap —
`TypeExpr::Path` vs `Parametric`, 137 hand-matched sites, and a ruling-A lock that silently skipped
parametric responses because *"a match that only has one arm at all… produces no output
whatsoever"* — **does not apply here.**

So the function-type half of the migration is: 107 sites → the 27 sites' form, plus deleting
`lex_keyword`'s paren-balancing machinery. No grammar decision remains.

**Bound:** unification is proven in the CHECKER for a concrete monomorphic shape. UNTESTED: generic
type variables in the bracket form (the corpus has `Fn(T)->U` with free `T`/`U`), a function type
NESTED inside a parametric, and whether EDN printing / macro reflection normalise the two
identically. Unification is the load-bearing property and it holds; the rest is unmeasured.
