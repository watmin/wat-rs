# Arc 098 — `:wat::form::matches?` — Clara-style pattern matching — DESIGN

**Status:** READY — opened and settled 2026-04-29. All design
questions resolved by Q&A 2026-04-29 + Q2/Q12 research.

**Predecessor of:** [arc 093](../093-wat-telemetry-workquery/DESIGN.md)
slice 4 (the example scripts depend on this matcher).

**Lineage explicit:** Clara Rules (Clojure rule engine, RETE-based,
Ryan Brush et al., 2014–). Adopting Clara's pattern syntax —
field-keyed bindings + interleaved constraints — without RETE,
forward-chaining, multi-fact joins, or recursion. **Single-item
filter only**: takes one Value (or `Option<Value>`), returns
`:bool`. Per the user's scope (2026-04-29):

> "we are doing a stream filter - so we are just processing a
> single item at a time and measuring if the item is acceptable
> per our constraints... no rete here - just the query interface
> in a pleasant ux form"

---

## The shape

```scheme
(:wat::form::matches? SUBJECT
  (TYPE-NAME
    CLAUSE1
    CLAUSE2
    ...))
```

**Subject:** a Value or `Option<Value>`. `:None` and non-Struct
return `false`. Struct whose type doesn't match `TYPE-NAME`
returns `false`. Otherwise the clauses run.

**Clauses** are either bindings or constraints. The pattern walker
classifies by shape.

### Bindings

```scheme
(= ?var :field)
```

`?var` is a fresh logic variable (any symbol starting with `?`);
`:field` is a struct field name (a `:keyword`). The binding makes
the field's value available as `?var` to subsequent constraint
clauses.

### Constraints

Any other clause. Vocabulary recognized inside clauses (no
`:wat::core::` prefix needed):

```
=  <  >  <=  >=  not=
and  or  not
where
```

`=`, `<`, `>`, `<=`, `>=`, `not=` are standard comparisons over the
bound `?var`s and literals.

`and`, `or`, `not` are logical combinators — each takes more
clauses (recursively).

`where` is the escape hatch — `(where <wat-expr>)` evaluates an
arbitrary wat expression in the binding scope; must return `:bool`.

Anything else inside a clause is a parse error at expansion.

### Worked example

```scheme
(:wat::form::matches?
  (:wat::telemetry::Event::Log/data-value e)
  (:trading::PaperResolved
    (= ?outcome :outcome)
    (= ?grace-residue :grace-residue)
    (= ?outcome "Grace")
    (> ?grace-residue 5.0)))
```

Read as: "subject is a `:trading::PaperResolved`; bind `?outcome`
from the `:outcome` field; bind `?grace-residue` from the
`:grace-residue` field; predicate is `?outcome == "Grace" AND
?grace-residue > 5.0`."

---

## Architecture — special form, both sides walk the grammar

Per Q2 research (2026-04-29): user defmacros expand BEFORE
type-checking, so they can't query the struct registry at
expansion. The substrate handles this by recognizing certain
built-in forms (`:wat::core::let*`, `:wat::core::match`,
`:wat::core::if`, `:wat::core::quasiquote`, `:wat::core::struct`,
etc.) as **special forms** dispatched directly by both
`runtime.rs::eval_call` and `check.rs::infer_call`. They have full
access to the registry and to runtime values respectively; they
walk forms with substrate-internal logic, not via user-facing
quasiquote templates.

`:wat::form::matches?` is the same shape. Two parallel walkers:

```
Type-check side (check.rs::infer_form_matches)
  1. Type-check subject. Accept :Value, :Option<Value>, or :TYPE-NAME.
  2. Look up TYPE-NAME in the struct registry — get fields[name → type].
  3. Walk clauses; recognize bindings (= ?var :field):
     - Verify :field exists on the struct.
     - Push ?var → field-type into a local type scope.
  4. Walk constraint clauses in the local scope; require :bool.
  5. Form returns :bool.

Runtime side (runtime.rs::eval_form_matches)
  1. Eval subject. :None / non-Struct → return false.
  2. Compare runtime Struct.type_name to TYPE-NAME → mismatch → false.
  3. Walk clauses:
     - Binding (= ?var :field): extract field value; push ?var → value
       into local Value scope.
     - Constraint: eval as :bool in the scope.
  4. AND all constraint results. Return.
```

Both sides share the same pattern-classifier helper
(`classify_clause`) — given a clause AST, decides binding vs
constraint vs sub-combinator. The classifier lives in a shared
module to keep check and runtime semantically aligned.

No new macro infrastructure. No struct-registry plumbing into
the macro system. No quasiquote templates.

### `?var` syntax

Per Q12 research (2026-04-29): wat's lexer accepts symbols
starting with `?` natively. `?outcome` lexes as a bare symbol;
the type checker treats it as a normal identifier. The pattern
walker recognizes `?`-prefix symbols as logic-variable
placeholders within the matcher's grammar; bindings push them
into a local scope.

No lexer changes needed.

---

## What ships in this arc

**One special form** — `:wat::form::matches?` recognized by both
sides:

```
(:wat::form::matches?
  subject :: :Value or :Option<Value> or :TYPE-NAME
  pattern :: (:TYPE-NAME clause ...))
  -> :bool
```

**Pattern grammar** — single shared classifier in
`src/form_match.rs` (new module). Recognizes:

- Binding: `(= ?var :field)` where `?var` is fresh and `:field`
  is a struct field keyword.
- Comparison constraint: `(= ?var lit)`, `(< ?var lit)`,
  `(> ?var lit)`, `(<= ?var lit)`, `(>= ?var lit)`,
  `(not= ?var lit)`. Either side of the comparison can be a
  bound `?var` or a literal.
- Logical combinator: `(and clause ...)`, `(or clause ...)`,
  `(not clause)`.
- Escape: `(where <wat-expr>)` — body is arbitrary wat
  evaluated in the binding scope; must be `:bool`.

**Errors at expansion (type-check time):**

- Pattern head isn't a `:keyword` referring to a registered
  struct → error (`unknown struct type :Foo`).
- Field doesn't exist on the struct → error
  (`struct :Type has no field :unknown-field`).
- Binding LHS isn't a fresh `?var` → error
  (`(= 5 :field) — left side must be a logic variable`).
- Binding RHS isn't a `:keyword` matching a field → error.
- Constraint clause head isn't in the recognized vocabulary →
  error (`unknown matcher head: :foo`).
- `?var` referenced in constraint but not bound → error.
- `where` body fails to type-check or doesn't return `:bool` →
  error.

**Errors at runtime:**

- Subject is `:None` or non-Struct or wrong type → return `false`
  (Clara semantic — no error).

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
- **Bindings extraction.** A future `match` (not `matches?`)
  returning `:Option<:HashMap<:Symbol, :Value>>` for callers
  that want the bound values back. Out of v1 — `where` lets
  callers compute over bindings without surfacing them.

---

## Slice plan

**Slice 1** — pattern grammar + classifier module + type-check side.

- New module `src/form_match.rs` with the shared classifier.
- `infer_form_matches` in `src/check.rs` — full type-check pipeline:
  subject type, struct lookup, clause walk, scope handling.
- Dispatch arm in `infer_call`'s special-form match for
  `:wat::form::matches?`.
- Tests: pattern type-check passes for valid patterns; rejects
  invalid patterns at expansion time with each error class
  surfaced.
- The runtime arm panics with "not yet implemented" — slice 1
  is type-check only.

**Slice 2** — runtime walker.

- `eval_form_matches` in `src/runtime.rs` — full runtime pipeline:
  subject eval, struct match, clause walk, AND of constraints.
- Dispatch arm in `eval_call`'s special-form match.
- Reuses the slice-1 classifier.
- Tests: end-to-end via `wat-tests/std/form/matches.wat` —
  worked example from arc 093 (PaperResolved Grace > 5.0).
  Cover all clause types: bindings, comparisons, and/or/not,
  where-escape, struct-of mismatch returns false, Option-None
  returns false, non-Struct subject returns false.

**Slice 3** — INSCRIPTION + USER-GUIDE + arc 093 unblock.

- INSCRIPTION.md sealing the arc.
- USER-GUIDE.md appendix forms-table additions.
- Arc 093's slice 4 example scripts now runnable; mark its
  Clara-matcher dependency resolved.
- 058 FOUNDATION-CHANGELOG row in lab repo.

---

## Open questions — none

All resolved by chat + Q2/Q12 research. Decisions captured
inline above.

---

## Predecessors / dependencies

**Shipped:**
- Arc 048 — user-defined enums (struct/enum infrastructure
  `eval_call` dispatches against).
- Arc 057 — closed algebra under itself (HolonAST as primary
  representation; struct field accessors auto-generated).
- Arc 091 slice 8 — `struct->form` / runtime quasiquote
  (precedent for substrate-recognized form-walking primitives).
- Arc 097 — Duration helpers (sibling of this arc; they ship
  ahead per the time → forms → clara dep order).

**Depends on:** nothing else. Pure substrate add.

## What this enables

- Arc 093 slice 4's worked-example queries — the pry/gdb
  debugging UX for telemetry interrogation.
- Reusable beyond telemetry: any wat code that wants Clara-style
  filter predicates over Value::Struct.
- Foundation for a future `:wat::form::match` (bindings-extraction
  variant) if a real consumer demands it.

**PERSEVERARE.**
