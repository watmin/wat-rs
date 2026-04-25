# wat-rs arc 055 — Recursive patterns — INSCRIPTION

**Status:** shipped 2026-04-25. Tenth wat-rs arc post-known-good.

The pattern matcher gains structural recursion. Patterns now mirror
the algebra (Option, Result, tuple, enum) at any depth. Bare symbols
bind, `_` discards, literals narrow. The grammar at the s-expr level
is unchanged; what changed is the **interpretation** — anywhere the
matcher saw a sub-pattern, it now treats that sub-pattern recursively
rather than demanding a bare symbol.

Builder direction:

> "what did you just reach for - what is wat missing - this is
> indicative that the core lang is deficient - your instincts are
> often an indicator for a pivot and improvement"

The reaching-for was `(Some (a b c d e f))` — destructure an Option of
a 6-tuple in one step. Pre-arc-055 wat rejected this with *"binder
must be a bare symbol, got list"*. Post-arc-055 it works, plus every
nesting that compounds with depth (`Result<Option<Tuple>>`,
`Some (Some x)`, etc.).

---

## What shipped

### Slice 1 — Recursive pattern checker

`src/check.rs` — extracted `check_subpattern(pat, expected_ty, env,
bindings, errors) -> Option<bool>` recursive helper. Returns
`Some(full)` on success (where `full` is true iff the sub-pattern is
bare-symbol-or-wildcard at every level), `None` on type/shape mismatch.

Disambiguation at list-position is by `expected_ty`:
- `Option<U>` — head Symbol "Some" is the variant constructor; recurse
  on field with U.
- `Result<T,E>` — head Symbol "Ok"/"Err" is the constructor; recurse
  on field with T or E.
- Enum — head Keyword `:enum::Variant` is the constructor; recurse
  on each field.
- Tuple `(T1,...,Tn)` — list is positional destructure; recurse on
  each element type. Head can be any sub-pattern.

The existing variant arms in `pattern_coverage` (Some/Ok/Err and the
user-enum tagged-variant case) replaced the bare-symbol check with a
`check_subpattern` call and propagated the `full` flag into the
returned Coverage variant.

`Coverage` enum updated:

```rust
enum Coverage {
    OptionNone,
    OptionSome { full: bool },
    ResultOk { full: bool },
    ResultErr { full: bool },
    EnumVariant { name: String, full: bool },
    Wildcard,
}
```

**Shape substitution at the top of the arm loop.** `detect_match_shape`
returns a `MatchShape::Option(fresh)` etc. with a fresh type variable.
The scrutinee unify call resolves the variable in the substitution but
doesn't mutate the shape. `pattern_coverage` would then pass the
unresolved fresh type to `check_subpattern`, breaking the type-shape
checks. Fix: apply the substitution to the shape's inner types
immediately after the scrutinee unify, then thread the resolved shape
through the arm loop.

### Slice 2 — Recursive pattern runtime

`src/runtime.rs` — `try_match_pattern` is now structurally recursive.
After dispatching on a variant tag (Option/Result/Enum), the inner
sub-pattern is matched against the variant's payload via a recursive
call. Tuple destructure added: when the value is `Value::Tuple` and
the pattern is a list whose head isn't a recognized variant
constructor, match positionally with arity check, recursing into each
element.

Literal patterns (IntLit/FloatLit/BoolLit/StringLit) added as new
match arms — compare for equality with the corresponding `Value::*`
variant.

**Linear-shadowing semantics** per Q2 in DESIGN: a name bound twice in
one pattern keeps the second binding (later recursion overwrites
earlier). Verified by `linear_shadowing` test.

The bare-symbol-only restriction in the user-enum tagged-variant arm
also dropped — fields use the same recursive match as Option's inner.

### Slice 3 — Exhaustiveness analyzer (v1 partial-coverage)

The arm-loop coverage tracking distinguishes `OptionSome { full: true }`
from `OptionSome { full: false }`. Only `full: true` satisfies
`covers_option_some` (and similarly for ResultOk/ResultErr/EnumVariant).
Partial coverage demands a fallback arm — either a top-level `_`
wildcard or another fully-general arm.

Diagnostic extended:

```
non-exhaustive: :Option<T> needs arms for both :None and (Some _),
or a wildcard. (Arc 055 — narrowing patterns like `(Some (1 _))` are
partial; add a fallback `_` arm.)
```

A more sophisticated literal-narrowing analyzer (one that knows
`(Some 1)`/`(Some 2)`/`(Some _)` collectively cover) is a follow-up
when a real caller needs it. Today's coarse rule plus the wildcard
escape hatch is enough.

### Carry-along — `:wat::core::string::concat`

Discovered mid-arc that wat-rs has had no string concatenation
primitive since inception. `string::join` exists (separator + Vec)
but is awkward at call sites where the goal is just stitching a few
strings. Added variadic `(:wat::core::string::concat s1 s2 ... sn)
-> :String`. Special-cased in the type-checker's variadic dispatch
(same shape as `tuple` and `vec`); runtime arity check rejects
zero-arg calls with a diagnostic. Five lines of runtime code, twenty
of type-checker registration. Pure ergonomic uplift; predates arc 055
but materialized while writing the test suite.

### Slice 5 — Docs

This INSCRIPTION + USER-GUIDE addendum + lab FOUNDATION-CHANGELOG row.

---

## Tests

`tests/wat_recursive_patterns.rs` — 10 integration tests:

1. `option_tuple_single_level_works` — `(Some (a b c))` against
   `:Option<(i64,i64,i64)>` binds and sums.
2. `result_tuple_destructure` — `(Ok (k v))` and `(Err msg)` arms.
3. `nested_options_three_levels` — `(Some (Some x))` / `(Some :None)` /
   `:None` / `_` fallback.
4. `wildcard_at_depth` — `(Some (_ x _))` middle-element extraction.
5. `literal_at_depth_picks_arm` — `((Ok 200) "ok")` matches; `(Ok n)`
   catches the rest.
6. `literal_fallback_to_general_arm` — same shape, value 418 falls
   through to the `(Ok n)` general arm.
7. `linear_shadowing` — `(Some (x x))` against `(5,7)` — `x` is 7.
8. `nonexhaustive_partial_pattern_rejected` — startup error when
   `(Some (1 x))` has no fallback arm.
9. `wildcard_fallback_compiles_and_runs` — same partial pattern with
   `(_ 0)` fallback succeeds.
10. `candlestream_next_shape_destructures_in_one_step` — the
    motivating case: `Option<(i64,f64,f64,f64,f64,f64)>` destructured
    in a single `(Some (ts open high low close volume))` arm.

Plus 1 implicit test of `string::concat` via tests #2, #5, #6, #10
which use the new primitive.

---

## Architecture decisions resolved

### Disambiguation by expected type (Q1 / sub-fog 5a)

The type checker uses `expected_ty` to decide whether a list pattern
is a variant constructor or a tuple destructure. At runtime, the
matcher dispatches on the *value*'s shape (Option/Result/Enum/Tuple)
since types aren't carried at runtime. Both routes converge on the
same matched bindings.

Edge: `(Some y)` against expected type `(i64, i64)` (tuple of arity 2).
Type checker treats this as tuple destructure with first elem bound to
"Some" and second to "y" (since the first position type is i64, not
Option). At runtime, the value is a Tuple, and the matcher takes the
tuple-destructure arm. The two layers stay coherent because both use
the structural property at their own level — type at check-time, value
at run-time.

### Linear shadowing (Q2)

Second occurrence of a name in one pattern overwrites the first — the
expected default for a Lisp-ish. Equality semantics (Erlang/Prolog
"second occurrence asserts equal") were rejected as surprising. Test
`linear_shadowing` pins the choice.

### Sub-pattern keyword shape

`:None` works as a sub-pattern only when expected_ty is `Option<U>` —
returns `Some(false)` (partial). User-enum unit variants
`:enum::Variant` work as sub-patterns when expected_ty is the enum.

### What stayed out of scope per BACKLOG

- Pattern guards (`pat if <bool-expr>`).
- Or-patterns (`pat₁ | pat₂`).
- As-bindings (`pat as name`).
- Struct field-name patterns (`(Candle :ts t :open o ...)`).
- Decision-tree compilation (Maranget-style).
- Literal-narrowing exhaustiveness analyzer (the smarter v2 of the
  partial-coverage rule).

These all build on the recursive-pattern foundation; easier to add
later now that recursion is in.

---

## Count

- Sites changed: **3** (`src/check.rs` pattern checker + exhaustiveness
  + variadic dispatch; `src/runtime.rs` `try_match_pattern`;
  `src/string_ops.rs` new `eval_string_concat`).
- New helpers: **2** in check.rs (`check_subpattern`,
  `infer_string_concat`), **1** in runtime/string_ops
  (`eval_string_concat`).
- Coverage enum redesigned: **5 variants**, three carry `full` flag.
- New runtime arms in `try_match_pattern`: **4** literal-pattern arms
  + **1** tuple-destructure arm + **1** new dispatch keyword
  (`string::concat`).
- Lines added: ~430 in check.rs (mostly the new `check_subpattern`
  helper), ~60 in runtime.rs, ~50 in string_ops.rs.
- New integration test crate: **1** (10 cases).
- Lib tests: **611 → 611** (no lib-level changes).
- Total tests passing: **943 → 953** (+10).
- Clippy: **0** warnings.

---

## What this unblocks

- **Every shim that returns `Option<Tuple>`** —
  `CandleStream::next!` is the first; sqlite-row iterators,
  websocket-message pulls, and recv-from-channel-with-payload all
  share the shape.
- **Every shim that returns `Result<Tuple>`** — analogous.
- **Sandbox-result inspection** — `RunResult` wrapping inner state can
  be destructured cleanly instead of through nested matches.
- **Tagged-enum payload destructure** — 058-048's enum variants with
  tuple payloads become as ergonomic as `Option`/`Result`.
- **Test ergonomics** — peel-then-destructure intermediate let-bindings
  collapse.
- **String stitching** — `string::concat` removes the `join "" vec`
  ceremony for fixed-arity concat at call sites.
- **Future arcs** — guards, or-patterns, struct-field patterns all
  build on this foundation.

---

## Commits

- `<wat-rs>` arc 055 — slices 1 + 2 + 3 + tests + INSCRIPTION +
  USER-GUIDE + `string::concat` (this commit).
- `<lab>` FOUNDATION-CHANGELOG row.

---

*"The grammar already says works on Option<T>, Result<T,E>, and tuples. It just didn't say and you can compose them, because the implementation didn't yet."*

**PERSEVERARE.**
