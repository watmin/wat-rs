# Arc 168 — `let` flat-shape binding vector + body implicit-do

**Status:** queued 2026-05-08; opens immediately. Three open questions
settled by user 2026-05-08; scope expands to include implicit-do body
for `let` + `fn` + `defn`.

**Gates:** none. Substrate has `WatAST::Vector` first-class
(arc 167 slice 1 — `src/ast.rs` + `src/parser.rs` + `src/hash.rs`
TAG_VECTOR=0x18). Arc 168 wires `let`'s binding consumer to
`WatAST::Vector` instead of `WatAST::List` AND extends `let` /
`fn` / `defn` body to accept 1+ trailing forms (implicit-do).

## Background

User direction 2026-05-08:

> *"sounds good to me - this'll be the first time we accept
> square brackets - we'll give let the same treatment next..."*

> *"let's work on 168 - let forms in brackets like clojure has"*

The arc 167 precedent settled the substrate gain (`WatAST::Vector`
as first-class node) and the consumer pattern (fn signature
consumes Vector with arrow-duality). Arc 168 extends the legal
positions of `WatAST::Vector` to the let bindings.

## End-state shape

```scheme
;; Canonical single binding, single body form
(:wat::core::let
  [x 1]
  (:wat::core::+ x 2))

;; Multiple bindings — flat alternation
(:wat::core::let
  [x 1
   y 2
   z (:wat::core::+ x y)]
  (:wat::core::* x y z))

;; Empty bindings — Clojure-faithful (degenerate but legal)
(:wat::core::let [] (:wat::core::+ 1 1))

;; Destructure binding — binder is a Vector of symbols
(:wat::core::let
  [[a b c] some-tuple]
  (:wat::core::+ a b c))

;; Mixed
(:wat::core::let
  [x 1
   [a b] (:my::pair-fn)
   y 2]
  body)

;; Implicit-do body — multi-form trailing block
(:wat::core::let
  [x 1
   y 2]
  (:my::log "computing")
  (:my::trace x y)
  (:wat::core::+ x y))    ;; the let's value
```

The OUTER bindings collection is a `WatAST::Vector` (instead of
List). Inside the outer Vector, elements ALTERNATE (binder,
expression) — `2N` elements total, must be even. Empty Vector
`[]` is legal.

A binder is one of:
- `Symbol` (single canonical binding — name's type inferred from RHS)
- `Vector` of `Symbol` (destructure — RHS must be tuple of matching arity)

Body is **1+ trailing forms** — implicit-do semantics. All but
last evaluated for side effects (values discarded but types
checked); last form's value IS the let's value. Mirror semantics
of `:wat::core::do`.

Legacy nested-pair-list outer shape `((name expr) (name expr) ...)`
fires `BareLegacyLetBindings` walker during the sweep window;
hard-retires in slice 3.

## Implicit-do body extends to `fn` and `defn` (arc 167 symmetry)

User direction 2026-05-08 (multi-form body answer):
> "Trailing forms after the binding vector treated as implicit do.
> Touches fn/defn body symmetry too."

So arc 168 also extends `:wat::core::fn` and `:wat::core::defn`
body to accept 1+ trailing forms after the `-> :T` arrow:

```scheme
;; fn — multi-form body
(:wat::core::fn
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:my::log "computing")
  (:wat::core::+ x 1))

;; defn — same pattern (defn is a macro on fn; rest-binder
;; already forwards trailing forms to fn)
(:wat::core::defn :user::add5
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:my::log "adding 5 to" x)
  (:wat::core::+ x 5))
```

The fn form's structural arity grows from "exactly 5 elements"
to "5+ elements" — `head + [args] + -> + :ret + body1 + body2 + ...`.

**Implicit-do is purely additive.** Old single-body code keeps
working unchanged; multi-body code is enabled. No walker needed
for the body change — the new shape SUBSUMES the old.

The defn macro in `wat/core.wat` already uses a rest-binder for
the body slot; passing N forms through rather than 1 is a tiny
edit (or no edit if the rest-binder already varadic-collects).

## Scope

### In scope (arc 168)

- **`let` bindings consume `WatAST::Vector`.** Outer is Vector;
  contents alternate (binder, expr). Empty Vector `[]` is legal.
- **Destructure binder is `WatAST::Vector` of symbols.** Recursive
  Vector-at-substrate-consumed-position (Vector inside the outer
  binding Vector). Still NOT a Vector value literal.
- **`let` body becomes implicit-do.** 1+ trailing forms; all but
  last evaluated for side effect; last is the let's value.
- **`fn` body becomes implicit-do.** 1+ trailing forms after the
  `-> :T` arrow. Same semantics as let body.
- **`defn` macro forwards multi-form body.** Already uses
  rest-binder for the body slot from arc 167; verify forwarding
  works for N forms (or extend if needed).
- **Walker `BareLegacyLetBindings` is a SWEEP-WINDOW DIAGNOSTIC.**
  Fires fatal on legacy outer-List shape with a verbose migration
  hint. Hard-retires in slice 3 per "doesn't leave cruft."
- **Sweep all let callsites.** ~563 sites across `wat/`, `wat-tests/`,
  `tests/wat_*.rs`, plus src/ lib unit-test fixtures (arc 167 slice
  4b precedent for substrate-internal `mod tests`).
- **No body sweep.** Implicit-do is purely additive. Existing
  single-body callsites continue to work; no migration needed.

### Out of arc 168's scope (affirmative)

- **Vector literals as values.** Same as arc 167's boundary —
  Vector is consumed at substrate positions only. `[1 2 3]` as
  an expression continues to error.
- **Typed legacy single-binding shape `((name :T) rhs)`.** Already
  retired in user code per arc 159. Arc 168's slice 3 deletes
  the parser arm that still accepts it (the legacy carrier the
  walker fires on covers this case via the outer-List walker).
- **`define` form's binding shape.** Define is a separate form
  with its own signature shape; arc 168 leaves it alone. (The
  user has signaled `define` will be killed in a coming arc; this
  is its own arc, not 168.)
- **Cosmetic `(do ...)` wrapper sweep.** With implicit-do support,
  many existing `(let [...] (do A B))` sites become equivalent to
  `(let [...] A B)`. Arc 168 does NOT sweep redundant `do`
  wrappers — the wrappers continue to work and authors can opt
  into the shorter shape lazily as code is touched. If/when
  redundant-`do` cleanup surfaces as friction, a separate arc
  closes it.

## Substrate work

### Layer 1 — parser

`parse_let_binding` in `src/runtime.rs:4167` currently expects
the OUTER bindings to be a `WatAST::List` (via `eval_let:4032`).
Inside, each binding is a `List` of 2 elements.

Post-arc-168:
- `eval_let` accepts `args[0]` as `WatAST::Vector`
- Vector contents alternate (binder, expr); even count required
- Binder is `Symbol` (single) OR `Vector` (destructure of Symbols)
- Legacy `WatAST::List` outer triggers walker (during slice 2-3
  window) or `MalformedForm` error (post-slice-4)

`infer_let` in `src/check.rs:6253` parallel update.

### Layer 2 — walker

`BareLegacyLetBindings { span: Span }` in `src/check.rs`:
- Variant on `CheckError`
- Display impl with the migration message (verbatim text load-bearing
  for sonnet's sweep)
- Walker fires when `(:wat::core::let LIST ...)` appears (List-as-
  bindings — legacy shape)
- Wired in `freeze.rs:599-616` user-source pre-pass alongside
  `BareLegacyPrimitive` etc., so substrate-authored stdlib forms
  silently migrate without walker firing (mirrors arc 167 slice 2
  delta A precedent)

**Migration message** (load-bearing — clear and unambiguous):

```
let bindings must be a vector `[name expr name expr ...]`.
Got legacy nested-pair-list `((name expr) (name expr) ...)`.

Migration:
  - Outer brackets change from `(...)` to `[...]`.
  - Inner pair-lists `(name expr)` flatten to alternating
    `name expr` inside the outer vector.
  - Destructure binders stay as a vector of symbols:
    `((a b c) rhs)` becomes `[[a b c] rhs]`.

Example:
  Before:  (:wat::core::let ((x 1) (y 2)) (+ x y))
  After:   (:wat::core::let [x 1 y 2] (+ x y))

  Destructure:
  Before:  (:wat::core::let (((a b c) tup)) ...)
  After:   (:wat::core::let [[a b c] tup] ...)
```

### Layer 3 — eval

Update `eval_let` (`src/runtime.rs:4013`) to:
- Accept `WatAST::Vector` outer
- Iterate alternating (binder, expr) pairs
- Dispatch binder kind: Symbol → Single; Vector → Destructure
- Sequential semantics preserved (each binding commits to scope
  chain before next RHS evaluates)

### Layer 4 — typed-single legacy shape

The current parser still accepts `((name :T) rhs)` via the
`is_typed_single` branch (`src/runtime.rs:4209-4229`). Arc 159
swept user code; this branch survives as a parse fallback. Arc
168 slice 4 deletes that branch — substrate has zero trace of
the typed-single shape post-arc-168. Walker covers any residual
sites in slice 2-3 window.

## Slice plan

Five slices total.

### Slice 1 — substrate consumer + walker + body extension + tests

Combine arc 167's slice 1+2 into a single slice — the foundation
(`WatAST::Vector`) is already there from arc 167.

**Substrate edits**:
- `src/runtime.rs`:
  - `eval_let` consumes Vector outer
  - `eval_let` body becomes implicit-do over 1+ trailing forms
  - `parse_let_binding` accepts `(binder, expr)` chunks where
    binder is Symbol or Vector
  - `eval_fn` body becomes implicit-do (form arity 5 → 5+);
    iterate trailing forms, evaluate all but last, return last
  - `try_parse_fn_shape_def` updated for the variable-arity
    fn-form shape
- `src/check.rs`:
  - `infer_let` parallel update (Vector outer + implicit-do body)
  - `infer_fn` parallel update (implicit-do body)
  - Add `BareLegacyLetBindings` variant + Display + walker +
    migration text
  - Body type-checking: each non-final form must type-check (we
    catch malformed code) but its inferred type is discarded;
    final form's type is the form's type
- `src/freeze.rs`: register walker in user-source pre-pass
- `wat/core.wat`: verify defn macro's rest-binder forwards N
  body forms cleanly (probably already does; if not, tiny edit)

**Tests** (new `tests/wat_arc168_let_flat_shape.rs`):
1. Single binding `[x 1]` evaluates correctly
2. Multiple bindings `[x 1 y 2]` evaluate sequentially
3. Sequential references `[x 1 y (+ x 1)]`
4. Destructure binding `[[a b] (some-pair)]`
5. Empty bindings `[]` allowed (degenerate but consistent)
6. Legacy outer-List `((x 1))` fires `BareLegacyLetBindings` walker
7. Type-mismatch in body surfaces correct error span
8. Migration message text test (verbatim assertion)
9. Odd-count vector errors clearly (`[x]` or `[x 1 y]`)
10. Multi-form let body — non-final forms evaluated for side
    effects; final form is the value
11. Multi-form let body type-check — non-final form with type
    mismatch surfaces error
12. Multi-form fn body — non-final forms evaluated for side
    effects; final form is the return value
13. Multi-form defn body — same semantics through macro expansion
14. Single-body let still works unchanged (regression check)
15. Single-body fn still works unchanged (regression check)

**Verification**:
- New tests 1-5, 7-15 pass
- Test 6 fires walker
- Workspace tests fail elsewhere on legacy let callers — that
  failure stream IS slice 2's input

### Slice 2 — sweep all let callsites (substrate-as-teacher)

Per FM 15. ~563 sites across `wat/`, `wat-tests/`, `tests/`.
Sonnet runs `cargo test`, reads `BareLegacyLetBindings`
diagnostics, applies the mechanical translation, re-runs,
repeats. No upfront enumeration.

The recipe:
- `((name expr) (name expr))` → `[name expr name expr]`
- `(((name :T) expr))` → `[name expr]` (typed-single legacy
  retires too; type inferred)
- `(((a b c) expr))` → `[[a b c] expr]` (destructure)

This is the first real sonnet sweep using the new
`.claude/settings.json` permission discipline (arc 167 slice 4b
shipped the fix; slice 4b ran on opus because the gap surfaced
THERE; arc 168 slice 2 is calibration).

### Slice 3 — substrate retirement

DELETE:
- `BareLegacyLetBindings` variant + Display + walker body
- `freeze.rs` walker registration
- `is_typed_single` branch in `parse_let_binding`
- The List-outer arm in `eval_let` / `infer_let` (now requires
  Vector outer; returns clean MalformedForm if not)

### Slice 4 — sweep src/ lib unit-test fixtures

Per arc 167 slice 4b precedent — substrate-internal `mod tests`
fixtures hidden by walker scoping. Slice 3's parser deletion
surfaces them; slice 4 sweeps mechanically.

### Slice 5 — closure paperwork

SCORE-1 through SCORE-4 + INSCRIPTION + 058 row + USER-GUIDE
update + atomic squash-merge.

## Settled answers (user direction 2026-05-08)

1. **Empty bindings `[]` allowed?** — YES.
   > "huh.. i think we follow clojure here.. i didn't expect this
   > to work but it makes sense — `user=> (let [] (+ 1 1))` 2"
2. **Destructure shape `[[a b c] expr]`?** — YES, Vector binder
   directly inside binding Vector.
3. **Implicit-do body?** — YES, absorbed into arc 168. Touches
   `let` + `fn` + `defn` body symmetry. Purely additive change
   (single-body code keeps working).
4. **Empty body legal?** — YES. Clojure precedent:
   > `user=> (let [])` → `nil`
   > `user=> (let [x 1])` → `nil`
   > `user=> (defn f [])` → `#'user/f`
   > `user=> (f)` → `nil`
   > `user=> ((fn []))` → `nil`

   Empty body is legal across `let` / `fn` / `defn`; the form's
   value is `:wat::core::nil`. For `fn` / `defn` the explicit
   `-> :T` arrow constrains: empty body type-checks only when
   `:T` is `:wat::core::nil` (substrate's existing return-type
   rule; no new behavior). Substrate allows it; idiom doesn't
   encourage it.

   > User direction 2026-05-08: *"a user... for whatever reason...
   > would need to express `(:wat::core::defn f [] -> :wat::core::nil)`
   > to achieve identical behavior - we allow it, we don't
   > encourage it"*

## Four-questions evaluation

Run on the bundled scope (let binding-shape + implicit-do body
across let/fn/defn) per Section 5 of `docs/COMPACTION-AMNESIA-RECOVERY.md`:

1. **Obvious?** YES. Clojure-faithful syntax; reader who knows
   Clojure recognizes `[x 1 y 2]` and implicit-do body
   immediately. Wat's arrow-duality from arc 167 stays
   unchanged.
2. **Simple?** YES. Each piece atomic (single eval/check arm
   change; single walker; mechanical sweep recipe). The bundle
   composes uniform changes — let consumer, fn consumer, defn
   macro, walker, sweep, retirement, lib-fixture sweep, closure
   — each piece doing one thing.
3. **Honest?** YES. Scope cuts named (Vector value literals,
   `define`, cosmetic `do`-wrapper sweep). Walker scoping
   rationale documented (user-source pre-pass per arc 167 slice
   2 delta A precedent). Empty-body semantics surfaced.
4. **Good UX?** YES. Denser binding shape; no explicit `do`
   wrapper at body position; uniform body rule across body-
   bearing forms. Sweep cost: walker fires once during the
   migration window; mechanical recipe.

Obvious + Simple + Honest hold; Good UX follows.

## Dependency ordering (stepping-stone analysis)

Components and their dependency relations:

| Component | Depends on |
|---|---|
| C1: `WatAST::Vector` foundation | (already shipped — arc 167) |
| C2: `let` consumer for Vector outer | C1 |
| C3: `let` body implicit-do | (independent) |
| C4: `fn` body implicit-do | (independent) |
| C5: `defn` macro forwards multi-form body | C4 |
| C6: walker for legacy let outer-List | C2 |
| C7: sweep all let callsites | C2 + C6 |
| C8: substrate retirement (walker + legacy parser arms) | C7 |
| C9: src/ lib unit-test fixture sweep | C8 |
| C10: closure paperwork | C9 |

C3 + C4 + C5 (implicit-do body) are independent of C2 + C6 + C7
(let binding-shape sweep). The stepping-stone analysis says: the
ordering between A-cluster (C2/C6/C7/C8) and B-cluster (C3/C4/C5)
is judgment, not dependency. We ship them in one arc per user
direction "this is a single unit"; we do NOT split into separate
arcs.

Within the arc, slice ordering uses the dependency graph:

- C1 already shipped (arc 167)
- Slice 1 ships C2 + C3 + C4 + C5 + C6 + tests (substrate
  consumer changes + walker — all touch substrate; coherent
  edit window)
- Slice 2 ships C7 (sweep — driven by walker diagnostics)
- Slice 3 ships C8 (substrate retirement — clean after sweep)
- Slice 4 ships C9 (lib unit-test fixture sweep, surfaced by
  C8's parser deletion per arc 167 slice 4b precedent)
- Slice 5 ships C10 (closure paperwork)

Five slices total — same as arc 167. The amount of slices is
whatever amount is necessary; here, five is necessary because
the dependency chain has five settled points (consumer → sweep
→ retirement → fixture-sweep → closure) and slice 1 bundles the
substrate consumer work coherently.

## Why arc 168 is the right shape

Four questions on the canonical Clojure `[name expr ...]` flat shape:
- **Obvious?** ✓ One canonical binding shape; matches Clojure
  exactly; instant reading once seen
- **Simple?** ✓ Single substrate change building on arc 167's
  foundation; sweep is mechanical
- **Honest?** ✓ Hard retirement of legacy outer-List + typed-single
  parser arms; "let bindings are `[name expr ...]`, period"
- **Good UX?** ✓ Cleaner reading; brackets visually separate from
  call/grouping parens; alternating elements are dense but
  scannable

All four favor the chosen path.

## Cross-references

- **Arc 154** — killed `let*`; let adopted sequential semantics
- **Arc 158a** — walker pattern-matches RHS for new-shape let
  bindings (the precedent for the binding-shape walker)
- **Arc 159** — dropped per-binding `:T` annotation in user code;
  arc 168's slice 3 retires the legacy parser arm that still
  accepts it
- **Arc 167** — `WatAST::Vector` foundation + fn flat-shape;
  arc 168 mirrors the slice pattern at smaller substrate scope
- **Arc 167's INSCRIPTION** — names arc 168 as the queued next-arc
  number; this DESIGN closes that promise

## Calibration prediction

Sonnet sweep similar in shape to arc 159 (951 sites for typed-let
retirement). ~563 sites for outer-List → Vector. Predict:
- Slice 1 (substrate + tests): 30-60 min opus
- Slice 2 (sweep): 60-120 min sonnet (post-`.claude/settings.json`
  fix; first real sonnet sweep on the new permission discipline)
- Slice 3 (retirement): 20-40 min opus
- Slice 4 (src/ unit-test fixtures): 15-30 min sonnet
- Slice 5 (closure): 30-45 min orchestrator

Total: ~3-5 hours.
