# Arc 098 — `:wat::form::matches?` — Clara-style pattern matching — INSCRIPTION

**Status:** shipped 2026-04-29.

The substrate gained a Clara-flavored single-item pattern matcher.
Across two implementation slices the lab's debugging UX picked up
the predicate-shaped query it had been waiting on:

```scheme
(:wat::form::matches?
  (:wat::telemetry::Event::Log/data-value e)
  (:trading::PaperResolved
    (= ?outcome :outcome)
    (= ?grace-residue :grace-residue)
    (= ?outcome "Grace")
    (> ?grace-residue 5.0)))
```

Reads as: "subject is a `:trading::PaperResolved`; bind
`?outcome` from the `:outcome` field; bind `?grace-residue` from
the `:grace-residue` field; predicate is `?outcome == "Grace" AND
?grace-residue > 5.0`." Returns `:bool`. Matches Clara's
[field-keyed binding syntax](https://github.com/cerner/clara-rules)
1:1 — no RETE, no joins, no recursion, no rule-firing. Single-item
filter. Sibling arc to [arc 097](../097-wat-time-duration/INSCRIPTION.md)
(time helpers); both unblock [arc 093](../093-wat-telemetry-workquery/DESIGN.md)
slice 4 (telemetry interrogation example scripts).

**Predecessors:**
- Arc 048 — user-defined enums + structs (the type registry the
  matcher walks).
- Arc 057 — closed algebra under itself (HolonAST as primary
  representation; struct field accessors auto-generated).
- Arc 091 slice 8 — `struct->form` / runtime quasiquote
  (precedent for substrate-recognized form-walking primitives).
- Arc 097 — Duration helpers (sibling; ships ahead per the time
  → forms → clara dep order).

**Surfaced by:** user direction 2026-04-29 mid-arc-093 design:

> "i hvae a such strong bias for clara style - this is well
> understood? this is an amazing UX"

> "I don't like the underscore... do you know of clojure's
> clara?... they just use ?prefix to do it... i /really like/
> clojure's clara library"

The arc 093 telemetry-interrogation arc needed a predicate shape
the user could write naturally — Clara's syntax was the explicit
target. Across the design Q&A the user kept pulling toward
"single-item stream filter — no RETE — just the query interface
in a pleasant ux form." The arc closed when slice 2's runtime
walker landed and the worked example evaluated `true` for the
expected subject and `false` for every documented mismatch
category.

---

## What shipped

### Slice 1 — Pattern grammar + classifier + type-check side

New module `src/form_match.rs` ships the shared pattern
classifier consumed by both walkers. Pure structural
classification: `classify_clause` returns a `RawClause<'a>` enum
naming the shape (binding vs comparison vs and/or/not vs where);
the walker that owns the local scope handles binding-vs-comparison
disambiguation and field-existence checks.

The classifier is intentionally neutral on semantics. It doesn't
know about types, struct registries, or runtime values — it just
recognizes the recognized vocabulary heads (`=`, `<`, `>`, `<=`,
`>=`, `not=`, `and`, `or`, `not`, `where`) and dispatches accordingly.
That keeps the classifier unit-testable without freeze
infrastructure (9 unit tests landed alongside the module) and
guarantees the type checker and runtime can never disagree on
what a clause IS — only on what it MEANS.

`infer_form_matches` in `src/check.rs` walks the pattern, pushes
bindings into a `HashMap<String, TypeExpr>` scope, recurses into
and/or/not sub-clauses with the same scope, and type-checks
`where`-bodies against `:bool`. Surfaces every error class the
DESIGN enumerated:

- Unknown struct type
- Field doesn't exist on the struct
- Binding LHS isn't a `?var`
- Binding RHS isn't a field keyword
- Unrecognized constraint head
- `?var` referenced but not bound
- `where`-body fails to type-check or returns non-`:bool`

All as `MalformedForm` diagnostics naming the offending shape.

### Slice 2 — Runtime walker

`eval_form_matches` in `src/runtime.rs` walks the same grammar
with values instead of types. Bindings push `?var → field-value`
into a child `Environment` that subsequent clauses see;
constraints AND-short-circuit. Per Clara semantics, every "this
doesn't fit" path returns `false` rather than erroring:

- Subject is `:Option<T>::None` → `false`
- Subject is `(Some <non-struct>)` → `false`
- Subject is non-Struct → `false`
- Subject is `Struct` of the wrong type → `false`

`Option<Some(struct)>` auto-unwraps one level so callers can pass
nullable telemetry rows directly without unwrapping in user code.

The walker uses `walk_match_clause` which threads the environment
through bindings while keeping `or` and `not` branches' bindings
local (they're ambiguous past the combinator boundary — which
branch's binding wins?). Comparison operators promote i64↔f64
the same way arc 050's `eval_compare` does, so mixed-numeric
constraints work without ceremony.

---

## Tests

24 new tests across the substrate:

- 9 in `src/form_match.rs::tests` covering the classifier directly
  (every clause kind, every error class, `?var` detection).
- 9 in `tests/wat_arc098_form_matches_typecheck.rs` covering the
  type-check side end-to-end via `startup_from_source` (3 valid
  patterns, 6 rejection categories).
- 15 in `tests/wat_arc098_form_matches_runtime.rs` covering the
  runtime walker end-to-end: the worked example + each clause
  kind + each comparison op + each negative path.

`cargo test --workspace`: every existing test still green; 24 new
tests pass. The substrate is ready for arc 093's slice 4 example
scripts to consume `:wat::form::matches?` as their predicate
language.

---

## What's NOT in this arc

Per the user's scope (single-item stream filter, no RETE):

- **Multi-fact joins.** Clara's `[A ...] [B ...]` with cross-fact
  bindings. Out, period.
- **Forward-chaining rule firing.** Clara's `defrule X => action`.
  Out.
- **Recursive patterns.** `(= ?paper (:Paper (= ?dir :direction)))`
  where the bound `?paper` is itself pattern-matched. Single-level
  patterns only for v1.
- **Multiple solutions / backtracking.** First match wins; no
  enumeration.
- **Reified patterns.** `Value::Pattern` runtime variant or
  pattern-as-data. Patterns are inline literals only; defmacros
  can construct them at expansion time but they don't survive as
  runtime values.
- **Custom predicates as special heads.** `member`, `between`,
  etc. Available via `where` until a real script wants them
  promoted.
- **Bindings extraction.** A future `:wat::form::match` (not
  `matches?`) returning `:Option<:HashMap<:Symbol, :Value>>` for
  callers that want the bound values back. `where` covers the
  predicate use case for now.

---

## Lessons

1. **Substrate-recognized special forms vs. user defmacros.** The
   first design pass proposed `:wat::form::matches?` as a user-
   defmacro that quasiquoted into `(if cond ...)` shape. Q2
   research turned up the actual constraint: macros expand BEFORE
   type-checking, so they can't query the struct registry to
   validate field names at expansion time. The substrate already
   ships several special forms (`:wat::core::let*`, `match`, `if`,
   `quasiquote`, `struct->form`) that both `runtime.rs::eval_call`
   and `check.rs::infer_call` dispatch directly. `:wat::form::matches?`
   is the same shape — both walkers share the slice-1 classifier,
   so grammar drift is impossible by construction.

2. **Pure-structural classifier as a forcing function.** Putting
   the classifier in its own module (`src/form_match.rs`) with
   no dependency on the type checker or runtime forced both
   walkers to agree on what a clause IS before deciding what it
   MEANS. The walker that owns the local scope handles
   semantic disambiguation. This pattern generalizes — any
   substrate form whose grammar lives across check-and-runtime
   should ship its classifier this way.

3. **Disambiguating `(= ?var X)` by scope.** A binding and a
   comparison have the same surface shape — `(= ?var :field)`
   binds when `?var` is fresh; `(= ?var "Grace")` compares when
   it's already bound. The walker handles this with a single
   `lookup-then-classify` rule: if `?var` is fresh AND RHS is a
   keyword that names a field, it's a binding; otherwise it's a
   comparison. The user's `(= ?outcome :outcome) ... (= ?outcome
   "Grace")` worked example exercises both interpretations of
   `=` in the same pattern, and the rule resolves them
   unambiguously.

4. **Clara semantics: "doesn't fit" is `false`, not error.** Every
   negative path — `:None`, non-Struct, wrong-type-Struct — returns
   `false` rather than raising. Matches Clara's "the predicate
   said no" reading. Errors are reserved for grammar-level
   problems caught at expansion (type-check time). At runtime
   the only path to a thrown error is a malformed pattern that
   skipped check, which is a substrate bug.

5. **`?var` syntax dropped in for free.** Q12 research confirmed
   wat's lexer accepts `?`-prefixed symbols as bare symbols
   natively. No tokenizer changes needed. The pattern walker
   recognizes the `?` prefix as a logic-variable convention
   within the matcher's grammar; outside the matcher, `?outcome`
   is just an unbound identifier that fails to resolve.

---

## Surfaced by (verbatim)

User direction 2026-04-29:

> "the first real caller is us debugging the program... we'll do
> some run and then we'll interrogate the system - we'll build up
> script templates and treat wat like a ruby of sorts"

> "I don't like the underscore... do you know of clojure's
> clara?... they just use ?prefix to do it... i /really like/
> clojure's clara library"

> "i hvae a such strong bias for clara style - this is well
> understood? this is an amazing UX"

> "i think :wat::form::* is a good place... we are doing a stream
> filter - so we are just processing a single item at a time and
> measuring if the item is acceptable per our constraints... no
> rete here - just the query interface in a pleasant ux form"

> "alright - get an arc going and let's do it"

The arc closed when slice 2's runtime walker landed + `cargo
test --workspace` came back green for the second time in a row.
The substrate is what the user said it should be when he named
it.

**PERSEVERARE.**
