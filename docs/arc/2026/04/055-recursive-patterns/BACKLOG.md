# wat-rs arc 055 — recursive patterns — BACKLOG

**Shape:** four slices. Type-checker recursion, runtime evaluator
recursion, exhaustiveness analyzer (v1 partial-coverage rule),
INSCRIPTION + docs. Slices 1 and 2 ship as a pair (checker without
runtime is unhelpful; runtime without checker accepts ill-typed
programs). Slice 3 can lag if needed; slice 4 is doc.

---

## Slice 1 — Recursive pattern checker

**Status: ready.**

`src/check.rs` (around line 1361, the current
"binder must be a bare symbol" site):

- Extract the pattern-checking logic into a recursive helper
  `check_pattern(pat, expected_ty, bindings, errors) -> Coverage`
  per the conceptual sketch in DESIGN.md §"Implementation sketch".
- Handle each `WatAST` variant inside the pattern position:
  bare symbol (bind), wildcard `_`, literal (narrow + check
  type), list (variant or tuple — recurse).
- Disambiguate variant vs tuple at a list position via the
  expected type per sub-fog 5a.
- Replace the "bare symbol required" error with a `check_pattern`
  recursive call.

Tests under `tests/wat_recursive_patterns.rs` covering the
checker behavior:

1. **Option<Tuple> compiles.** `(Some (a b c))` against
   `:Option<(i64,i64,i64)>` → no errors, bindings are i64×3.
2. **Result<Tuple> compiles.** `(Ok (k v))` against
   `:Result<(String,i64),String>` → bindings k:String, v:i64.
3. **Type mismatch in nested position errors.** `(Some (1 x))`
   against `:Option<(String,i64)>` → mismatch on first field
   (literal int vs String type).
4. **Variant arity mismatch errors.** `(Some (a b c))` against
   `:Option<(i64,i64)>` → wrong tuple arity.
5. **Sub-pattern bare-symbol-only allowed.** `(Some x)` still
   works, binds x to the inner Tuple value.

**Estimated cost:** ~80 LOC + 5 tests. Day and a half.

---

## Slice 2 — Recursive pattern runtime

**Status: ready (after slice 1; checker must accept the patterns
before runtime can evaluate them).**

`src/runtime.rs` — pattern matching at `match` arm dispatch:

- Extract pattern-vs-value matching into `match_pattern(pat, val,
  bindings) -> bool`.
- Mirror the checker's recursion: literal compares for equality;
  symbol binds; wildcard accepts; list (variant or tuple)
  recurses.
- Variant dispatch via the value's tag (existing); tuple dispatch
  via arity (existing); the new bit is recursing on each sub-
  pattern after dispatch succeeds.
- Linear binding semantics per Q2 in DESIGN — second occurrence
  of a name shadows the first within a single pattern.

Tests:

6. **Option<Tuple> runs.** Match `(Some (1 2 3))` against
   `(Some (a b c)) -> a + b + c` → returns 6.
7. **Wildcard at depth.** `(Some (_ x _))` matches any 3-tuple
   inside Some, binds x to the middle.
8. **Literal at depth picks the right arm.** `((Ok 200) "ok")
   ((Ok n) (str n))` against `(Ok 200)` → "ok"; against
   `(Ok 404)` → "404".
9. **Nested Option three levels deep.** `(Some (Some x))` binds;
   `(Some :None)` and `:None` reach their arms.
10. **Linear shadowing.** `(Some (x x))` against `(Some (5 7))`
    → x is 7 in the body (second binding wins).

**Estimated cost:** ~60 LOC + 5 tests. One day.

---

## Slice 3 — Exhaustiveness analyzer (v1 partial-coverage)

**Status: ready (independent of slices 1 and 2 in shape, but
coverage rule needs its data path).**

`src/check.rs` (around lines 920-970, the existing exhaustiveness
machinery):

- A nested pattern that's neither a bare symbol nor `_` registers
  as **partial coverage** for its variant/tuple position.
- Existing `:Option<T>` analyzer demands `Some _` and `:None` (or
  wildcard); generalize: it now demands either `(Some <bare or
  wildcard>)`, OR all narrowing `Some(...)` arms must collectively
  cover with a fallback, OR a top-level wildcard `_`.
- Same generalization for `:Result<T,E>` and enum variants.

For v1, the simplest rule is: **a nested pattern with any non-
trivial sub-structure counts as partial coverage; a fallback arm
(top-level `_` or `:None`/`Err _` for Option/Result) must follow.**

Tests:

11. **Non-exhaustive nested errors at startup.**
    `((Some (1 _)) "matched-1")` alone (no fallback) → startup
    error with the existing "non-exhaustive" wording, mentioning
    that the partial nested pattern needs a fallback.
12. **Wildcard-fallback compiles.**
    `((Some (1 _)) "matched-1") (_ "other")` → compiles, runs.

**Estimated cost:** ~30 LOC + 2 tests. Half a day.

A more sophisticated literal-narrowing analyzer
(`(Some (1 _))` covers `Some(tuple starting with 1)`,
`(Some (2 _))` covers `Some(tuple starting with 2)`, etc.,
collectively covering when the literal space is finite) is a
follow-up arc when a real caller needs it.

---

## Slice 4 — INSCRIPTION + USER-GUIDE addendum

**Status: blocked on slices 1-3 shipping.**

- **INSCRIPTION.md** — three sites (check.rs, runtime.rs,
  exhaustiveness analyzer), test count, LOC delta, the v1
  partial-coverage rule's bound. Standard inscription shape.
- **USER-GUIDE addendum.** Add to §"match — pattern destructure":

  ```scheme
  ;; Patterns recurse — destructure Option<Tuple> in one step:
  (:wat::core::match row -> :i64
    ((Some (ts open high low close volume))
      (:wat::core::i64::+ ts (:wat::core::f64::to-i64 close)))
    (:None 0))

  ;; Wildcard at any depth, literal at any depth, nested variants
  ;; — all compose:
  (:wat::core::match resp -> :String
    ((Ok (Some 200)) "ok")
    ((Ok (Some n))   (:wat::core::string::concat "code:" (:wat::core::i64::to-string n)))
    ((Ok :None)      "no-content")
    ((Err msg)       msg))
  ```

- **FOUNDATION-CHANGELOG row** documenting "patterns are
  recursive over the algebra (Option/Result/tuple/enum-variant)"
  as the language-level rule, with v1's partial-coverage caveat.

**Estimated cost:** ~1 hour. Doc only.

---

## Verification end-to-end

After slices 1-3 land, `holon-lab-trading/wat-tests/io/CandleStream.wat`
collapses from:

```scheme
(:wat::core::match row -> :()
  ((Some r)
    (:wat::core::match r -> :()
      ((ts open high low close volume) ...)))
  (:None ...))
```

to:

```scheme
(:wat::core::match row -> :()
  ((Some (ts open high low close volume)) ...)
  (:None ...))
```

— and the rest of the lab's iterator-shaped consumers inherit the
property.

---

## Out of scope

- **Pattern guards** (`pat if <bool-expr>`). Separate proposal.
- **Or-patterns** (`pat₁ | pat₂`). Separate proposal.
- **As-bindings** (`pat as name`). Separate proposal.
- **Struct field-name patterns** (`(Candle :ts t ...)`). Separate
  proposal, probably 058-NNN with a real surface design.
- **Decision-tree compilation** (Maranget-style). Performance
  optimization; ship the simple recursive form and revisit if
  hot paths need it.
- **Literal-narrowing exhaustiveness** (`(Some 1)`/`(Some 2)`/
  `(Some _)` collectively covering `:Option<i64>`). Follow-up
  refinement; v1's coarse rule (any non-trivial nested = partial
  coverage, demand fallback) is enough to ship.

---

## Risks

**Type-checker complexity.** The recursion is genuinely simple
in shape (mirror the type), but the disambiguation between
"variant at list head" vs "tuple of arity 1+ where head is a
symbol" needs care (sub-fog 5a). Mitigate by making the type
context the disambiguator — the expected type at this position
determines which interpretation applies. Same logic the existing
checker already uses at the top level; just generalize it to
sub-positions.

**Exhaustiveness false positives.** v1's partial-coverage rule
might reject legitimately-exhaustive matches (e.g.,
`(Some 1)`/`(Some 2)`/`(Some _)`/`:None` — collectively
exhaustive over `:Option<i64>` if the analyzer were smart enough,
but v1 will reject as non-exhaustive). Mitigate with the
escape hatch: a top-level `_` fallback is always accepted. The
caller adds `_` if v1 rejects what they consider exhaustive;
the smarter analyzer ships when called for.

**Linear vs equality binding semantics** (Q2 in DESIGN). Choosing
linear shadowing is the expected default and matches every
current pattern checker in wat. No real risk; just document.

---

## Total estimate

- Slice 1: 1.5 days
- Slice 2: 1 day
- Slice 3: 0.5 day
- Slice 4: 1 hour

**Three days end-to-end.** Slices 1 and 2 should ship together
(at minimum); slice 3 can ship same day or follow-up. Slice 4 is
hours, drop in when the code's done.
