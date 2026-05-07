# Arc 158 — Untyped `let` bindings (Clojure-faithful inference)

**Status:** opened 2026-05-07, after arc 157 closed.

## User direction (verbatim)

> *"do we need typed bindings or can we always infer the type of
> the value being bound?"*

> *"so... we can now support something like... (:wat::core::let
> [x 2] (:wat::core::+ x 2)) — we don't need to have x declared
> to be a :wat::core::i64 ?"*

> *"we do not support brackets yet - they are coming - let's
> remove the typed bindings first.. we'll get to clojure brackets
> soon - not now"*

## Goal

Drop the per-binding type annotation `:T` from `:wat::core::let`.
Each binding's type is inferred from its expression — same lesson
as arc 145 / `:wat::core::def` (memory
`feedback_substrate_already_typed.md`), applied to the
inner-binding slot.

**Out of scope (deferred to a future arc by user direction):**
Clojure-style square-bracket binding form `[name expr name expr]`.
Arc 158 keeps the existing paren-grouped binding shape; only the
type annotation is removed.

## Final shape

| Before | After |
|---|---|
| `(:wat::core::let (((name :T) expr) ...) body)` | `(:wat::core::let ((name expr) ...) body)` |

Each binding goes from a 2-element list-of-list (`((name :T) expr)`)
to a 2-element flat list (`(name expr)`). The outer bindings list
`(...)` and the body slot stay unchanged.

Concrete example:

```clojure
;; Before (arc 154 era)
(:wat::core::let
  (((floor :wat::core::f64)
    (:wat::holon::coincident-floor (:wat::config::dim-count))))
  (:wat::core::fn ((cos :wat::core::f64) -> :wat::core::bool)
    (:wat::core::< (:wat::core::- 1.0 cos) floor)))

;; After (arc 158)
(:wat::core::let
  ((floor (:wat::holon::coincident-floor (:wat::config::dim-count))))
  (:wat::core::fn ((cos :wat::core::f64) -> :wat::core::bool)
    (:wat::core::< (:wat::core::- 1.0 cos) floor)))
```

`floor`'s type is inferred as `:wat::core::f64` from
`coincident-floor`'s return type. No annotation needed.

## Why no per-binding type annotation

Substrate is statically typed via inference + recipient
unification (memory `feedback_substrate_already_typed.md`). Arc
145 (typed-let) was BACKED OUT for the form-level `-> :T` slot
for this exact reason. Arc 158 closes the parallel inner slot.

**Edge cases:**
- **Genuinely ambiguous expressions** (e.g., `[]` of unknown
  element type) — answer: hint inside the expression, not on
  the binding form. Same answer as `def`.
- **Subtype narrowing** — wat-rs has no subtypes; not relevant.
- **Mutual recursion** — `let` is sequential since arc 154; RHS
  sees only prior bindings; not relevant.
- **Documentation** — comments cover this; not a load-bearing
  reason for syntax noise.

The form-level `-> :T` (arc 145 backout) and per-binding `:T`
(this arc) are the same redundancy in two slots.

## Migration shape — clean break (Path A)

Per arc 154 (`let*` retirement) and arc 155 (`lambda`
retirement) precedent, plus user direction *"clojure is our
guiding light - we're just building a strongly typed clojure on
rust"*: substrate accepts new shape only; walker fires
`LegacyTypedLetBinding` CheckError on old shape. No transitional
alias.

Atomic substrate + consumer sweep per recovery doc § 7.

## Empirical scope

| Bucket | Count |
|---|---|
| `wat/` + `wat-tests/` + `crates/*/wat*/` + `examples/` | ~436 |
| `src/*.rs` + `tests/*.rs` (embedded wat strings) | ~515 |
| `holon-lab-trading/` (cross-repo) | ~965 |

Total: ~1916 occurrences (each with one or more bindings to
transform). Per user direction: count is irrelevant; the work
is the work.

## Slice plan

### Slice 1a — substrate

- Parser: accept new binding shape `(name expr)` alongside
  legacy `((name :T) expr)`. Both parse cleanly during
  migration window.
- `infer_let` recognizes new shape: name is bare keyword;
  expression is the second sibling; inferred type IS the
  binding's type.
- Mint `LegacyTypedLetBinding` CheckError variant + walker per
  substrate-as-teacher Pattern 3 (mirror arc 154's
  `BareLegacyLetStar` recipe). Walker fires per source-level
  legacy binding.
- 8-12 tests covering: new shape parses; type inference works;
  legacy shape fires walker; mixed bindings (multiple in one
  let, both new shape) work; sequential references work; etc.

`model: "sonnet"` explicit per FM 12.

DO NOT COMMIT (atomic with 1b per recovery doc § 7).

### Slice 1b — wat-rs consumer sweep (~951 sites)

Mechanical 1:1 transform across wat-rs:
- `((name :wat::core::T) expr)` → `(name expr)`
- Multi-binding lets: `(let (((a :T1) e1) ((b :T2) e2)) ...)`
  → `(let ((a e1) (b e2)) ...)`

Sweep order: `wat/*.wat` → `wat-tests/**/*.wat` →
`crates/*/wat-tests/**/*.wat` → `examples/**/*.wat` →
`tests/*.rs` (embedded) → `src/*.rs` (embedded).

Atomic commit with 1a when wat-rs workspace = 0-failed.

### Slice 1c — holon-lab-trading consumer sweep (~965 sites)

Cross-repo sweep mirroring 1b's transform pattern. Same
mechanical edit; lab repo separate atomic commit.

### Slice 2 — substrate retirement + closure paperwork

- Walker body retired per substrate-as-teacher § "Retire the
  hint" (arc 154 / 155 precedent)
- `LegacyTypedLetBinding` variant + Display retained as
  orphaned scaffolding (arc 113 precedent)
- INSCRIPTION + 058 changelog row + USER-GUIDE update +
  WAT-CHEATSHEET update
- Pre-INSCRIPTION grep mandatory per FM 11
- Orchestrator-side per `feedback_paperwork_orchestrator_side.md`

## Cross-references

- **Arc 145 (typed-let)** — back-out lesson on form-level
  `-> :T`; this arc closes the parallel inner-slot redundancy
- **Arc 154 (kill let*)** — closest precedent for let-related
  substrate change with walker + sweep
- **Arc 155 (fn rename)** — closest precedent for clean-break
  retirement (no transitional alias)
- **Arc 157 (def)** — `def` ships with no type annotation per
  same lesson; arc 158 makes `let` consistent
- **Memory `feedback_substrate_already_typed.md`** — paid-for
  lesson driving the no-annotation decision
- **Memory `feedback_stepping_stones_proactive.md`** — slicing
  framework
- **Recovery doc § 7** — atomic-commit-across-coordinated-sweeps
  (substrate + 1b + 1c coordinate)

## Four questions

- **Obvious?** YES — same lesson as arc 145 / def, applied to
  parallel slot.
- **Simple?** YES — drops one syntactic element; inference fills
  the gap.
- **Honest?** YES — eliminates redundant noise; expression's
  type IS binding's type.
- **Good UX?** YES — Clojure-faithful, less ceremony, consistent
  with `def`.

## Stepping-stones

- **Tractability of next steps?** Slight — `let` matches `def`
  in shape (no type annotation on bindings); sets up future
  Clojure-bracket form arc.
- **Dependencies?** None. Pure substrate change.
- **Composition?** Atomic. Substrate slice + consumer sweeps.

## Estimated effort

- Slice 1a: ~30-40 min Sonnet (substrate + 8-12 tests)
- Slice 1b: ~25-40 min Sonnet (~951 wat-rs sites; mechanical;
  atomic with 1a)
- Slice 1c: ~25-40 min Sonnet (~965 lab sites; cross-repo;
  separate atomic commit)
- Slice 2: ~25 min orchestrator (closure paperwork)
- Total: ~2-2.5 hours wall-clock if Mode A clean throughout
