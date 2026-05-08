# Arc 168 slice 1 — substrate consumer + walker + body implicit-do + tests

## Goal

Ship the substrate consumer changes that wire `:wat::core::let`'s
binding-vector position to `WatAST::Vector` (extending arc 167's
foundation), AND extend `let` / `fn` / `defn` body to accept 1+
trailing forms (implicit-do). Plus the `BareLegacyLetBindings`
walker for the legacy outer-List shape. Plus tests.

After this slice, the substrate accepts the new shapes; legacy
outer-List code fires the walker; existing single-body code keeps
working unchanged. Slice 2's input is the failure stream of let
callsites the walker fires on (~563 sites).

## Branch + commit policy

- **Active branch**: `arc-168-let-flat-shape` (slices 1+2+3+4 share
  this branch per atomic-merge discipline; slice branch carries
  all WIP commits; main untouched)
- Multiple WIP commits + pushes welcome on the branch for backup
- DO NOT push to main; orchestrator merges atomic to main as a
  single squash commit after slice 5 closure paperwork ships
- Use `./scripts/cargo-test-summary.sh` for progress checks
  (`passed: N failed: M` single-line; safe from awk-pipe denial)

## Background context (read these first)

- `docs/arc/2026/05/168-let-flat-shape/DESIGN.md` — full arc
  scope, settled answers, four-questions evaluation, dependency
  ordering
- Arc 167's INSCRIPTION (`docs/arc/2026/05/167-fn-flat-signature/INSCRIPTION.md`)
  — the foundation arc that minted `WatAST::Vector` + the fn
  flat-shape consumer pattern arc 168 mirrors. Especially the
  delta-A walker scoping in `freeze.rs:599-616` user-source
  pre-pass (your walker registers there, NOT in `check_program`).
- Arc 154 `BareLegacyLetStar` walker (`src/check.rs:261`,
  `:638`, `:920`) — the precedent for this slice's walker
  variant + Display + Diagnostic shape
- Arc 159 INSCRIPTION + walker — the typed `:T` retirement that
  brought let to its current `(name expr)` shape; you're
  evolving from `((name expr) (name expr))` outer-List to
  `[name expr name expr]` outer-Vector
- Arc 167 slice 2 SCORE (`docs/arc/2026/05/167-fn-flat-signature/SCORE-SLICE-2.md`)
  delta A + B — scoping + transitional parser pattern that
  applies here

## End-state shape

```scheme
;; Canonical
(:wat::core::let [x 1 y 2] (:wat::core::+ x y))

;; Empty bindings — Clojure-faithful
(:wat::core::let [] (:wat::core::+ 1 1))

;; Empty body — returns :wat::core::nil
(:wat::core::let [x 1])

;; Destructure binder — Vector of symbols inside binding Vector
(:wat::core::let [[a b c] some-tuple] (:wat::core::+ a b c))

;; Multi-form body — implicit-do
(:wat::core::let [x 1]
  (:my::log "computing")
  (:wat::core::+ x 1))           ;; ← let's value

;; Same body shape for fn
(:wat::core::fn
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:my::log "called")
  (:wat::core::+ x 1))           ;; ← fn's return value

;; Same body shape for defn (via macro forwarding)
(:wat::core::defn :user::add5
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:my::log "adding 5")
  (:wat::core::+ x 5))
```

## Substrate edits

### 1. `src/runtime.rs`

**`parse_let_binding`** (currently at `:4167`) — rewrite to consume
flat (binder, expr) chunks where binder is:
- `WatAST::Symbol` → `LetBinding::Single { name, rhs }` (canonical)
- `WatAST::Vector` of Symbols → `LetBinding::Destructure { names, rhs }`

The function takes a single chunk now (binder + rhs); the caller
(`eval_let`) iterates the outer Vector's elements 2-at-a-time.

The legacy List-shape branches (typed-single `((name :T) rhs)`,
nested-List destructure `((a b c) rhs)`) are NOT in this slice's
scope as canonical paths — they're the LEGACY shape the walker
fires on. Slice 3 deletes them. For slice 1, the canonical parser
accepts ONLY the new flat shape.

**`eval_let`** (currently at `:4013`) — rewrite to:
- Accept `args[0]` as `WatAST::Vector` (clean error if not)
- Vector contents must be even-length; iterate 2-at-a-time
  (binder, expr) into `parse_let_binding`
- Body becomes implicit-do over `args[1..]`:
  - `args.len() == 1` (no body) → return `Value::wat__core__nil()`
  - `args.len() >= 2` → evaluate `args[1..N-1]` for side effects
    (discard values), evaluate `args[N-1]` and return its value

**`eval_fn`** (currently at `:3831`) — extend to:
- Accept `args.len() >= 3` (was strict `== 4`)
- args[0] = ARGS-VECTOR, args[1] = `->`, args[2] = :RET-TYPE
- Body is `args[3..]`:
  - `args.len() == 3` (empty body) → Function body materializes
    as no-op returning `:wat::core::nil` at call time
  - `args.len() == 4` (single body) → unchanged
  - `args.len() >= 5` (multi body) → implicit-do over args[3..]

**Implementation discretion**: Function::body is `Arc<WatAST>`
today. Multi-form body needs either:
- (a) Change Function::body to `Arc<Vec<WatAST>>` (or `Vec<...>`
  inline) — uniform iteration at call time
- (b) Synthesize a `(:wat::core::do ...)` AST when there are
  multiple body forms; pass through single body as-is; for empty
  body, synthesize `:wat::core::nil` keyword
- (c) Some other shape

You choose. The semantics + tests are load-bearing; the internal
representation is yours to pick. Document the choice in your
report.

**`try_parse_fn_shape_def`** (currently at `:1946`) — adjust the
arity check from "exactly 5 elements" to "5+ elements" (allow N
body forms). This helper recognizes `(:wat::core::def :name
(:wat::core::fn sig body...))` for arc 166's defn macro
pre-registration; the body slot now varies.

### 2. `src/check.rs`

**`infer_let`** (currently at `:6253`) — parallel update:
- Accept `WatAST::Vector` outer
- Iterate (binder, expr) pairs
- Each binding: infer rhs's type, bind name's type into the
  let-scope chain
- Body becomes implicit-do over trailing forms:
  - Each non-final form must type-check (we infer its type so
    it's well-formed; the inferred type is discarded for value
    purposes but the type-check happens)
  - Final form's type IS the let's type
  - Empty body → let's type is `:wat::core::nil`
- Sequential semantics preserved (each binding visible to next)

**`infer_fn`** (currently at `:9427`) — parallel update for
multi-form body. The declared `-> :T` constrains the LAST body
form's type. Intermediate forms can be any well-typed type
(value discarded; same as `do`'s rule).

**`parse_fn_signature_for_check`** (currently at `:9494`) and
**`parse_fn_signature_for_check_diag`** (currently at `:9547`) —
arity check expansion mirroring the runtime side.

**Add `BareLegacyLetBindings { span: Span }`** variant to
`CheckError`:
- Variant declaration + Display impl + Diagnostic impl mirroring
  `BareLegacyLetStar`'s shape (see `:261`, `:636`, `:920`)
- Walker function `walk_for_legacy_let_bindings` that recurses
  through every form, detecting `(:wat::core::let LIST ...)`
  shape (List-as-bindings — legacy outer-List form). Fires fatal
  with the migration hint.
- Recurses into `WatAST::Vector` children (mirror the arc 167
  slice 3 substrate gap fix; if you find a place where Vector
  recursion is missing, pull the same 5-line fix pattern)

**Migration message** (load-bearing — verbatim text drives slice
2's mechanical translation):

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

### 3. `src/freeze.rs`

Register `validate_legacy_let_bindings` walker in the user-source
pre-pass region (around `:599-616`) alongside other arc-167-style
walkers. **NOT in `check_program`** — substrate-authored stdlib
forms must silently migrate without walker firing (mirrors arc
167 slice 2 delta A precedent). Slice 2 sweeps stdlib via the
same translation; the walker fires only on user-source forms
during the sweep window.

### 4. `wat/core.wat` defn macro

Verify the existing rest-binder `& (rest :AST<wat::core::Vector<wat::WatAST>>)`
forwards multi-form body cleanly. The macro splices `,@rest`
into the fn form; if rest collects N elements, splicing forwards
N elements. **Probably no edit needed** — but verify with a test
case that defn with multi-form body works through expansion.

## Tests (new file `tests/wat_arc168_let_flat_shape.rs`)

Mirror arc 167's test file structure
(`tests/wat_arc167_fn_flat_signature.rs`). Required cases:

1. **Single binding** — `(:wat::core::let [x 1] (:wat::core::+ x 1))`
   evaluates to 2
2. **Multiple bindings** — `(:wat::core::let [x 1 y 2] (:wat::core::+ x y))`
   evaluates to 3
3. **Sequential references** — `(:wat::core::let [x 1 y (:wat::core::+ x 1)] y)`
   evaluates to 2 (y sees x)
4. **Empty bindings** — `(:wat::core::let [] (:wat::core::+ 1 1))`
   evaluates to 2
5. **Empty body** — `(:wat::core::let [x 1])` evaluates to
   `:wat::core::nil`
6. **Destructure binding** — `(:wat::core::let [[a b]
   (:my::pair)] (:wat::core::+ a b))` evaluates correctly given
   a tuple-returning fn
7. **Legacy outer-List fires walker** — `(:wat::core::let ((x 1))
   (:wat::core::+ x 1))` produces `BareLegacyLetBindings` error
8. **Migration message text** — assert the migration hint text
   appears verbatim in the walker diagnostic
9. **Odd-count vector errors** — `(:wat::core::let [x] body)` and
   `(:wat::core::let [x 1 y] body)` produce a clear MalformedForm
10. **Multi-form let body** — `(:wat::core::let [x 1] body1 body2
    body3)`: each non-final form evaluated for side effect; last
    form is the value
11. **Multi-form let body type-check** — non-final form with
    type mismatch surfaces error at correct span
12. **Multi-form fn body** — fn with multiple body forms after
    `-> :T`: same semantics as let
13. **Multi-form defn body** — defn macro forwards N body forms
    cleanly through fn expansion
14. **Single-body let regression** — old-style single-form body
    keeps working unchanged
15. **Single-body fn regression** — old-style single-form body
    keeps working unchanged

## Verification

- `cargo build --release --workspace` green
- `./scripts/cargo-test-summary.sh` shows the workspace failure
  count (expected: ~563 let callsites in stdlib + tests fail
  walker; the slice-2 sweep input)
- New tests 1-6, 8-15 pass
- Test 7 fires walker (PASS the assertion)
- `cargo test --release --test wat_arc168_let_flat_shape` 15/15 pass
- `cargo test --release --test wat_arc167_fn_flat_signature` 9/9
  pass (regression — arc 167 tests still pass post-fn body
  multi-form change)
- Lib unit tests stay 793/0 (slice 4b precedent — substrate-internal
  fixtures use single-body throughout)

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify the new `walk_for_bare_primitives` Vector arm at
  `src/check.rs` — that's permanent infrastructure (arc 167 slice 3)
- DO NOT modify `parse_fn_signature` (the canonical fn-sig parser
  from arc 167) — extending it for multi-body is in
  `eval_fn`/`infer_fn`'s args-handling, not the sig parser
- DO NOT modify `wat/core.wat`'s defn macro shape (verify
  forwarding works; if it doesn't, STOP and report — that's a
  substrate gap to surface)
- USE `./scripts/cargo-test-summary.sh` for progress measurement
- DO NOT pipe `cargo test` through `awk` — use the scripts

## FM 5 GUARDRAIL — explicit

If a substrate quirk surfaces (Function::body shape choice
breaks something subtle, the existing `do` form's empty-arity
check conflicts with the implicit-do nil-return rule, etc.):

- STOP and report
- DO NOT bridge by special-casing the test
- DO NOT modify substrate code outside the listed scope to "make
  it work"
- The right answer is always: STOP, name the gap, let
  orchestrator decide

## Report shape

When complete, report:

1. Final cargo test summary via `./scripts/cargo-test-summary.sh`
2. Function::body representation choice (a/b/c/your-own) +
   one-paragraph rationale
3. Site count by file for substrate edits
4. Honest deltas — substrate quirks, hidden dependencies
5. Test 7 walker output sample (paste the actual diagnostic so
   we can verify migration text)
6. Branch state confirmation
7. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-1.md. If you exceed the upper bound still
iterating, STOP and report current state — orchestrator decides
on continuation.
