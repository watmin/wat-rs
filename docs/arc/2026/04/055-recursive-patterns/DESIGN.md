# wat-rs arc 055 — Recursive patterns in `:wat::core::match`

**Status:** opened 2026-04-25. Tenth wat-rs arc post-known-good.

**Scope:** small-to-medium. Pattern matcher gains recursion. The
type checker, runtime evaluator, and exhaustiveness analyzer
each become structural-recursive over patterns instead of
one-level dispatchers.

Builder direction:

> "what did you just reach for - what is wat missing - this is
> indicative that the core lang is deficient - your instincts
> are often an indciator for a pivot and improvement"

> "new arc - yes"

The "what I reached for" was `(Some (a b c d e f))` — destructure
an Option of a 6-tuple in a single pattern, the way every ML-
family language does it. Today wat rejects this with *"binder
must be a bare symbol, got list"* (`src/check.rs:1370`).

---

## Motivation

Pattern matching is the algebra of structured-data destructure.
Wat already has the shapes that compose: `:Option<T>`,
`:Result<T,E>`, tuples, structs (058-049), enums (058-048).
The expected algebra is **recursive** — patterns mirror the
shape of values, at any depth. Today wat allows exactly one
level of destructure per `match` form.

The deficiency surfaced concretely on 2026-04-25: the lab's
first in-crate shim (parquet OHLCV via `:rust::lab::CandleStream`)
ships `next!` returning `:Option<(i64,f64,f64,f64,f64,f64)>` —
the natural shape of "iterator next" over OHLCV rows. The
consumer wants:

```scheme
(:wat::core::match row -> :()
  ((Some (ts open high low close volume))
    ;; use ts and close ...
    )
  (:None
    ;; end of stream
    ))
```

Today this is rejected. The workaround is a two-step decompose
— match the Option to a single binder, then match again to
destructure the tuple:

```scheme
(:wat::core::match row -> :()
  ((Some r)                                    ; bind r:tuple
    (:wat::core::match r -> :()                ; second match
      ((ts open high low close volume)
        ;; finally use ts and close ...
        )))
  (:None ...))
```

The structure of the data is buried under bureaucracy. And it
**compounds with depth.** A `:Result<:Option<(i64,f64)>,String>`
needs three levels of nested match. Same shape applies to every
substrate edge that yields composite values: every shim's `next`
/ `peek` / `try-recv`, every `parse-line` style helper, every
sandbox result that wraps an inner result.

This isn't a feature request — it's filling a hole in the
algebra surface. The grammar already says *"works on `:Option<T>`,
`:Result<T,E>`, and tuples"* (USER-GUIDE §"match"). It just doesn't
say "and you can compose them," because the implementation doesn't
yet.

**The gap isn't quirky — it's a finished-by-half pattern matcher.**
Every modern language with algebraic data types makes patterns
recursive. The `(Variant pat₁ pat₂ ...)` shape and the
`(pat₁ pat₂ ...)` shape are both already in the grammar; arc 055
makes their `pat`s themselves patterns, recursively.

---

## What ships

A single principle:

> A `match` pattern is, recursively, one of:
> - **Bare symbol** → bind matched value
> - **Wildcard `_`** → no bind
> - **Literal** (int, float, string, bool, keyword) → match by equality
> - **`(:Variant pat ...)`** → match constructor; recurse into each `pat`
> - **`(pat₁ pat₂ ... patₙ)`** → match tuple of arity n; recurse into each

The grammar itself is unchanged at the s-expr level. The change
is in the **interpretation**: anywhere the matcher sees a sub-
pattern, it now treats that sub-pattern recursively, rather than
demanding a bare symbol.

### Examples enabled

```scheme
;; Option<Tuple> — the immediate caller (CandleStream::next!)
(:wat::core::match row -> :()
  ((Some (ts open high low close volume))
    ...)
  (:None ...))

;; Nested Option
(:wat::core::match maybe-pair -> :i64
  ((Some (Some x)) x)
  ((Some :None) -1)
  (:None -2))

;; Result of Option of Tuple
(:wat::core::match resp -> :i64
  ((Ok (Some (k v))) v)
  ((Ok :None) 0)
  ((Err _) -1))

;; Wildcard at any depth
(:wat::core::match pair -> :i64
  ((Some (_ x _)) x))

;; Literal at any depth
(:wat::core::match outcome -> :String
  ((Ok 200) "success")
  ((Ok 404) "not found")
  ((Ok n) (:wat::core::string::concat "code: " (:wat::core::i64::to-string n)))
  ((Err msg) msg))
```

All of these are rejected today; all should compile and run
after this arc.

### What stays the same

- Outer arm shape: still `(pattern body)` per arm.
- `-> :T` annotation: still required; arm bodies still checked
  against `T` independently.
- Variant constructors: still `Some`/`None`/`Ok`/`Err` for
  built-ins, plus user-defined enum variants per 058-048.
- Exhaustiveness checking: still required at startup; just
  generalizes to nested cases.
- `:None` keyword shape (no constructor parens): unchanged.

---

## Decisions resolved

### Q1 — Exhaustiveness with nested patterns

A nested pattern partially covers the parent's space. Two cases:

**Case A.** A `(Some <pat>)` arm where `<pat>` is itself
non-trivial (not a bare symbol or wildcard) — it covers some
`Some` values but not all. The match still needs another arm to
cover the rest of the `Some` space, or a wildcard:

```scheme
(:wat::core::match maybe-pair -> :i64
  ((Some (1 x)) x)           ; only matches Some((1, _))
  (_ 0))                     ; covers everything else
```

Without the wildcard, this is non-exhaustive — same logic that
applies to incomplete enum coverage today, just generalized
through one more level.

**Case B.** A nested pattern that's fully general — bare symbol
or wildcard — fully covers its position. `(Some (a b))` covers
**all** `Some(tuple of arity 2)` values. So the only remaining
arm needed is `:None`.

Implementation: extend the existing `MatchShape`/coverage
analysis (`check.rs:920-970`) to track coverage per pattern
position recursively. `(Some (_ _))` registers as "Some fully
covered" — same as `(Some _)` does today. `(Some (1 _))`
registers as "Some partially covered (only when first field is
1)" — which is a new state the analyzer needs to express.

For arc 055 v1: **cover only the "fully general subpattern fully
covers parent" case at the analyzer level.** Literal-narrowed
nested patterns (`(Some (1 _))`) are accepted by the matcher and
runtime, but exhaustiveness analysis treats them as "partial
coverage" and demands a fallback wildcard or `:None` arm.

This is a strict improvement over today (today rejects all
nested patterns); the literal-narrowing analyzer refinement can
ship as a follow-up if a real caller needs it.

### Q2 — Variable shadowing within a single pattern

Can the same name bind twice in one pattern? `(Some (x x))` —
binds first field as `x`, then **also** binds second field as
`x`?

Two semantics:

- **Linear**: each binder is fresh; second `x` shadows first.
- **Equality**: second occurrence asserts the field equals the
  first binding (Erlang/Prolog convention).

Equality semantics are surprising for a Lisp-ish; linear is the
expected default. **Decision: linear.** The second `x` shadows
the first. If the caller wants equality, they bind two distinct
names and use a guard (which itself is out-of-scope for this
arc; see Q5).

Test the chosen behavior. Don't leave it implicit.

### Q3 — Pattern over user-defined struct fields

`(Candle ts open high low close volume)` — destructure a struct
positionally? Or `(Candle :ts t :open o ...)` — by field name?

This arc commits **only to the shapes wat already has at the
match site today** — `Some`, `None`, `Ok`, `Err`, user enum
variants (058-048), tuples. **Struct destructure is out-of-
scope.** Structs use field-access today
(`(:Candle/close c)`); a pattern shape for them is its own
proposal (probably 058-NNN, with a real proposal-doc surface,
not just an arc).

### Q4 — Pattern over enum variants with payload tuples

058-048 introduced tagged enum variants. A variant constructed
as `(:MyEnum::Variant a b c)` should match as
`(:MyEnum::Variant a b c)` — recurse into each field, same as
the built-in variants. **Yes** — this is the same recursive
mechanism as `(Some pat)` and `(Ok pat)`. Variant-pattern
checker reuses the same recursive subroutine.

### Q5 — Pattern guards

Many ML languages allow `pat if <bool-expr>`. **Out of scope.**
Guards are a separate proposal — they affect arm-evaluation
order and exhaustiveness in ways that warrant their own design.
Arc 055 covers structural recursion only.

### Q6 — Or-patterns

`(Some 1 | Some 2)` — match either. **Out of scope.** Same
rationale as Q5 — exhaustiveness analysis interaction is
non-trivial. A separate arc when called for.

### Q7 — As-bindings

`(Some (a b)) as pair` — bind the entire matched value to `pair`
while ALSO destructuring it. **Out of scope.** Useful but not
required to close the immediate gap.

---

## Implementation sketch

Three sites change. All three are already structural-recursive
in shape over `WatAST`; they currently bottom out at "must be
bare symbol" and need to keep recursing.

### 1. Type checker — `src/check.rs`

The current site (line ~1361) demands `WatAST::Symbol(...)` for
the inside of `(Variant _)`. Replace with a recursive helper:

```rust
// Conceptual shape:
fn check_pattern(
    pat: &WatAST,
    expected_ty: &TypeExpr,
    bindings: &mut HashMap<String, TypeExpr>,
    errors: &mut Vec<CheckError>,
) -> Coverage {
    match pat {
        WatAST::Symbol("_", _) => Coverage::Full,
        WatAST::Symbol(b, _) => {
            bindings.insert(b.clone(), expected_ty.clone());
            Coverage::Full
        }
        WatAST::IntLit(_,_) | WatAST::StringLit(_,_) | ... => {
            // literal — type-check, record narrowed coverage
            check_literal_matches_type(pat, expected_ty, errors);
            Coverage::Partial
        }
        WatAST::List(items, _) => {
            // (Variant pat...) — variant constructor with sub-patterns
            // OR (pat...) — tuple destructure with sub-patterns
            match items[0] {
                WatAST::Symbol(s, _) if is_variant_constructor(s) => {
                    let field_tys = field_types_of_variant(s, expected_ty);
                    for (sub_pat, sub_ty) in items[1..].zip(field_tys) {
                        check_pattern(sub_pat, sub_ty, bindings, errors);
                    }
                }
                _ => {
                    // tuple destructure
                    let elem_tys = tuple_element_types(expected_ty);
                    if items.len() != elem_tys.len() { errors.push(...); return Coverage::Empty; }
                    for (sub_pat, sub_ty) in items.iter().zip(elem_tys) {
                        check_pattern(sub_pat, sub_ty, bindings, errors);
                    }
                }
            }
            // Coverage analysis combines sub-coverages
        }
        ...
    }
}
```

Today's check function has a chunk of the recursive logic
already — it checks `(Variant <bare-symbol>)` correctly. The
work is replacing the "bare symbol" leaf with another
`check_pattern` call.

### 2. Runtime evaluator — `src/runtime.rs`

The matcher today picks an arm by checking the variant tag,
then binds the inner value to the named symbol. With recursive
patterns, after the variant tag matches, **recursively** match
the inner value against the sub-pattern, populating the
let-binding scope. Same shape as the type checker, just
returning a binding map instead of accumulating errors.

```rust
fn match_pattern(pat: &WatAST, val: &Value, bindings: &mut HashMap<String, Value>) -> bool {
    match (pat, val) {
        (WatAST::Symbol("_", _), _) => true,
        (WatAST::Symbol(b, _), v) => { bindings.insert(b.clone(), v.clone()); true }
        (WatAST::IntLit(n, _), Value::I64(v)) => *n == *v,
        (WatAST::List(pats, _), v) => {
            // dispatch on Variant constructor or tuple shape
            // recurse into each sub-pattern
        }
        ...
    }
}
```

### 3. Exhaustiveness analyzer

Per Q1 above, the v1 analyzer treats any nested pattern that
isn't a bare symbol or `_` as "partial coverage." This is a
small extension to the existing `MatchShape` machinery
(`check.rs:996-1100`) — same recognized shapes, plus a
"partial-coverage" marker per arm.

A fully-recursive coverage analyzer (one that knows
`(Some (_ _))` covers `Some` of any 2-tuple, but `(Some (1 _))`
does not) is a follow-up refinement. Doesn't block this arc.

---

## Tests

`tests/wat_recursive_patterns.rs` — six cases minimum:

1. **Option<Tuple> — single-level nesting works.**
   `(Some (a b c)) → use a, b, c`. Verify the variables bind to
   the right tuple positions.

2. **Result<Tuple> — same shape, different variant.**
   `((Ok (k v)) ...) ((Err msg) ...)`. Bindings work; both arms
   reachable.

3. **Nested Options — three-level recursion.**
   `(Some (Some x))` binds x; `(Some :None)` matches inner-None;
   `:None` matches outer-None.

4. **Wildcard at any depth.**
   `(Some (_ x _))` matches any 3-tuple inside Some, binds
   middle field to x.

5. **Literal at any depth.**
   `((Ok 200) ...) ((Ok n) ...)`. The literal narrows; the bare
   symbol catches the rest.

6. **Exhaustiveness — non-exhaustive nested rejected.**
   `((Some (1 x)) ...) ((None) ...)` — missing wildcard for
   non-1 case → startup error per Q1's v1 rule. Test asserts
   the diagnostic.

Plus an integration test that exercises the original
`CandleStream::next!` use case end-to-end (the case that
provoked this arc).

---

## Sub-fogs

### 5a — Variant-constructor disambiguation in pattern position

In a pattern, `(Foo a b)` could mean:
- Match the `Foo` variant of an enum, with two field sub-patterns
- Match a 3-tuple where the first element is `Foo` (which can't
  happen at the type level — `Foo` would be an unbound symbol
  binding the first tuple position to a value named `Foo`)

The disambiguator is the **type expected at this position**. If
the position type is an enum/variant type and `Foo` is one of
its variants, treat as variant; if it's a tuple type, treat as
tuple destructure (and `Foo` becomes a bare-symbol binding for
the first position). The type checker has this info.

Edge: a tuple of arity ≥1 whose first position is itself an
enum. Patterns like `((Some x) y)` — outer is tuple of arity 2,
first sub-pattern is `(Some x)` (variant), second is `y`
(symbol). The current grammar already handles this in principle;
verify the recursion path doesn't trip over it.

### 5b — Variant arity vs payload arity

`(Some (a b c))` — the **outer** arity is 1 (Some takes one
field), the **inner** arity is 3 (the tuple has three fields).
Don't confuse the two when validating.

### 5c — `:None` keyword vs `(None)` list shape

Today `:None` matches the None variant via a keyword token. With
recursive patterns, should `(None)` also work? The user-guide
shows `:None` (line 477). **Decision: keep `:None` as the
canonical shape.** No new spelling. The recursion concerns
*sub-pattern depth*, not the surface spelling of nullary
variants.

### 5d — Bindings scope across the body

After a match arm fires, all of that arm's bindings are in scope
for the body. This is unchanged. `((Some (a b))  body)` — both
`a` and `b` are visible in `body`. Same scoping rule as `let*`.

---

## What this arc does NOT add

- **Pattern guards** (`pat if <bool-expr>`). Q5.
- **Or-patterns** (`pat₁ | pat₂`). Q6.
- **As-bindings** (`pat as name`). Q7.
- **Struct field-pattern destructuring** (`(Candle :ts t ...)`).
  Q3 — separate proposal, probably 058-NNN.
- **Active patterns / view patterns** (Haskell-style). Out of
  scope.
- **Pattern synonym macros** — could later wrap recurring
  patterns under a name; not now.
- **Literal-narrowing exhaustiveness analyzer.** Q1 — v1 treats
  non-symbol/wildcard nested patterns as "partial coverage" and
  demands a fallback. A more sophisticated analyzer ships when
  needed.

---

## Non-goals

- **Backwards-incompatible grammar changes.** The s-expr grammar
  is unchanged. Anything that parses today still parses; what
  *checks* and what runs is what expands.
- **Performance optimization of pattern compilation.** Recursive
  matching is straightforward in interpretation; arc 055 ships
  the simple form. Decision-tree compilation (Maranget-style)
  is a possible follow-up if hot paths emerge — won't on
  config-pass match arms.
- **058-NNN INSCRIPTION addendum.** This arc is a pattern-
  matcher expansion, not a new substrate form. The grammar
  already specifies what the arms look like; the implementation
  catches up. FOUNDATION-CHANGELOG entry only.
- **USER-GUIDE rewrite.** A small section addition under
  §"match — pattern destructure" (lines ~474-495) showing
  recursive patterns alongside the existing flat examples.

---

## What this unblocks

- **Every shim that returns `Option<Tuple>`** — `CandleStream::
  next!` is the first; sqlite-row iterators, websocket-message
  pulls, recv-from-channel-with-payload all share the shape.
- **Every shim that returns `Result<Tuple>`** — analogous.
- **Sandbox-result inspection.** A `RunResult` wrapping inner
  state that itself has shape can be destructured cleanly
  instead of through nested matches.
- **Tagged-enum payload destructure.** 058-048's enum variants
  with tuple payloads become as ergonomic as `Option`/`Result`.
- **Test ergonomics.** Today's test helpers often write
  intermediate let-bindings just to peel one Option layer
  before destructuring; arc 055 collapses those.
- **Future arcs** — guards (Q5), or-patterns (Q6), and
  struct-field patterns (Q3) all build on the recursive-pattern
  foundation. Easier to add later if recursion is in first.
