# Arc 167 — `fn` flat-shape signature with `<- / ->` arrow duality

**Status:** queued 2026-05-08; opens immediately.

**Gates:** none. Substrate has `:wat::core::fn` (arc 155) +
`:wat::core::defn` (arc 166); wat-edn parses `[...]` as `Value::Vector`
already (`crates/wat-edn/src/value.rs:50`). Arc 167 wires the wat
language layer to consume vector-shape function signatures.

## Background

User direction 2026-05-08:

> "i want to start working towards this... `[x <- :wat::core::i64 y <-
> :wat::core::i64] -> :wat::core::i64` ... `<-` consumes; `->`
> produces ... we have fn being the sole define of function forms and
> defn wrapping it."

Plus on retirement discipline:

> "i want the path that doesn't leave cruft... we thought we hard
> deprecated a bunch of forms only to later realized days later that
> we didn't and spent hours cleaning up stuff i thought was hard done."

The arrows-as-duals semantic: `<-` and `->` point FROM the type
TOWARD the named slot. Args have `<-` (slot consumes from a value
source); returns have `->` (slot produces to a value sink). Once
seen, reading any wat function definition becomes mechanical.

## End-state shape

```scheme
;; defn (composes def + fn)
(:wat::core::defn :user::add5-to-2-nums
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+ 5 x y))

;; fn (anonymous)
(:wat::core::fn
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+ 5 x y))
```

Five elements at the form level: `head + name? + [args-vector] +
-> :ret + body`. Defn carries a name keyword between head and
arg-vector; fn omits it.

## Scope

### In scope (arc 167)

- **Mint `WatAST::Vector` proper.** Substrate gain: the language
  layer now has a Vector node distinct from List. Parser handles
  `[...]` brackets producing `WatAST::Vector`. wat-edn already
  parses `[...]` as `Value::Vector`; arc 167 wires the conversion
  + parser path so wat sources can write `[...]` and have it
  reach the AST as a Vector node.
- **Fn signature accepts ONLY the new vector shape.** Legacy
  nested-sig `((x :T) (y :T) -> :T)` removed from the canonical
  parser path.
- **Defn macro accepts the new shape.** Macro body in `wat/core.wat`
  passes the sig through to fn unchanged (fn handles the new shape
  natively).
- **Sweep all fn/defn callsites.** Substrate-as-teacher discipline:
  cargo test reveals every legacy site as a fail; iterate from the
  diagnostic stream until 0 failed. Sites span `wat/`, `wat-tests/`,
  `tests/wat_*.rs` embedded wat strings, `examples/`, `crates/*/`.
- **Walker `BareLegacyFnSignature` is a SWEEP-WINDOW DIAGNOSTIC.**
  Fires fatal during the sweep with a verbose migration hint that
  sonnet can mechanically translate. After sweep clears, the walker
  hard-retires (variant + Display + walker body all DELETED — no
  orphaned scaffolding per user direction "doesn't leave cruft").

User direction 2026-05-08:
> "which option is the best long term solution - no bandaids nor
> half measures. adding vec proper feels like the correct move ...
> arcs can be as complex as they need to be - we just add more
> slices as we need."

This locks in `WatAST::Vector` as a first-class substrate node
rather than a marker-flag-on-List bandaid. The arc grows to cover
the full vertical: substrate plumbing → consumer surface → sweep
→ retirement.

### Out of arc 167's scope (affirmative)

- **`let` flat-shape `[name expr name expr]`.** User direction
  2026-05-08: *"we'll give let the same treatment next."* Arc 168
  (next arc, opens after 167 closes) ships let consuming the new
  `WatAST::Vector` foundation arc 167 establishes.
- **`define` form.** User direction 2026-05-08: *"define will keep
  working - fn and defn will be broken."* `:wat::core::define`'s
  signature shape `(:name (p :T) ... -> :R)` is structurally distinct
  from fn's; arc 167 leaves it alone.
- **Other future Vector consumers** (struct field declarations,
  enum variant payloads, etc.). Each gets its own arc when a
  caller surfaces.
- **Short-form opt-in crate** (`(defn add5 [x <- i64] -> i64 ...)`
  without FQDN). Memory `project_short_form_crate_future.md` keeps
  this on the future-crate-pivot ladder; ships much later.

## Substrate work

### Layer 1 — lexer

`<-` is two characters, both already in the symbol charset (`<` and
`-` are legal symbol bodies). The lexer should already accept `<-`
as a `WatAST::Symbol`. **Verify in slice 1 with a probe before
assuming.**

### Layer 2 — parser

`parse_fn_signature` (`src/runtime.rs`) and
`parse_fn_signature_for_check` (`src/check.rs`) currently expect a
list-shape sig: `((p1 :T1) (p2 :T2) ... -> :R)`. Replace with
vector-shape parsing:

- The fn form's args-position element MUST be a `WatAST::Vector` (or
  whatever the parser produces from `[...]` — confirm in slice 1
  audit).
- Vector body is a flat sequence of (name, `<-`, type) triples.
- The `-> :T` arrow + return type are SIBLING positions of the
  vector inside the fn form, NOT inside the vector.

Form structure:
```
(:wat::core::fn ARG-VECTOR ARROW RET-TYPE BODY)
```
That's 5 elements (vs current fn's 3 elements: sig + body — wait,
let me re-check). Actually current fn is `(fn sig body)` = 3
elements where sig contains the arrow + ret. New fn is
`(fn [args] -> :T body)` = 5 elements where the vector contains
just the args.

Validate in slice 1 — the parser shape for new fn vs old fn.

### Layer 3 — walker `BareLegacyFnSignature`

Mirror arc 154's `BareLegacyLetStar` walker shape (`src/check.rs`):

- Variant: `BareLegacyFnSignature { span: Span }` on `CheckError`
- Display impl with the migration message
- `walk_for_legacy_fn_signature` recurses through every form,
  detecting `(:wat::core::fn ((... )) ...)` shape (list-as-first-arg
  to fn) — fires fatal with the migration hint
- Wired into `check_program` via `validate_legacy_fn_signature`

**Migration message** (the load-bearing piece for sonnet's sweep —
clear and unambiguous):

```
fn signature must be a vector binding form `[name <- :T name <- :T ...] -> :Ret`.
Got legacy nested-sig list `((x :T) (y :T) -> :T)`.

Migration:
  - Each `(p :T)` pair becomes `p <- :T` inside a `[...]` vector.
  - The `-> :R` arrow + return type remain siblings of the vector
    (NOT inside the vector).
  - The new shape arrows-as-duals: `<-` consumes (input type),
    `->` produces (output type).

Example:
  Before:  (:wat::core::fn ((x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64) body)
  After:   (:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64 body)

  Defn equivalent (same migration; defn is a wat macro composing def + fn):
  Before:  (:wat::core::defn :name ((x :T) -> :T) body)
  After:   (:wat::core::defn :name [x <- :T] -> :T body)
```

### Layer 4 — defn macro

`wat/core.wat`'s defn defmacro currently expands:
```
(:wat::core::defn :name :sig :body)
  → (:wat::core::def :name (:wat::core::fn :sig :body))
```

Post-arc-167 the macro shape changes to accept 4 args:
```
(:wat::core::defn :name :sig-vector :arrow-and-ret :body)
  → (:wat::core::def :name (:wat::core::fn :sig-vector :arrow-and-ret :body))
```

Or equivalent shape that aligns with the new fn surface.
Confirm sonnet's macro shape choice in the slice 1 BRIEF.

## Substrate-as-teacher discipline

This arc IS substrate-as-teacher applied. Three failure modes from
the recovery doc apply:

- **FM 15** — failures are the work, not a crisis. Expect a high
  fail-count after substrate change; that count IS the migration
  brief.
- **FM 16** — don't preempt sonnet with tool-availability preamble in
  briefs.
- **FM 14** — sweep internals same arc; no leftovers.

## Slice plan

Five slices total. The first lands the substrate foundation
(`WatAST::Vector`); the next consumes it at the fn-sig position;
the third sweeps; the fourth retires the migration walker; the
fifth is closure paperwork. Each slice ships independently
verifiable.

### Slice 1 — `WatAST::Vector` foundation (substrate plumbing only)

**Goal**: substrate accepts `[...]` in wat source and produces
`WatAST::Vector(Vec<WatAST>, Span)`. No fn/defn consumer changes
yet. Existing wat code unaffected.

**Critical scope constraint** (user direction 2026-05-08): vectors
are **expressions in binding-syntax positions only**. Not value
literals yet.

> "i'm not ready to support vec literals as values... just vecs as
> exprs... e.g. fn's args. `(fn [x <- i64] -> i64 (+ 0 x))` is
> what i want to support now. `(conj [0 1] y)` — not this... we
> don't know how to entertain this yet."

This means:
- Parser produces `WatAST::Vector` whenever it encounters `[...]`
- `eval` arm on `WatAST::Vector` errors with a clear message
  ("vector literals at value position are not supported in arc
  167; vectors are currently consumed only in fn/defn signatures")
- `check`/`infer` arm on `WatAST::Vector` similarly errors at
  value/expression positions
- The fn-sig consumer (slice 2) is the only legal Vector-consumer
  position in arc 167. let-binding consumer (arc 168) follows.

This keeps the substrate honest about what's supported. Future arcs
extend the legal positions (vector literals as Value::Vec; let
binding-vector; etc.) deliberately.

**Substrate edits**:
- `src/ast.rs`: add `Vector(Vec<WatAST>, Span)` variant to
  `WatAST`. Update `span()` accessor. Update `Display` impl if
  applicable.
- `src/parser.rs`: extend the parser to accept `[...]` brackets
  alongside `(...)` lists, producing `WatAST::Vector`. wat-edn
  parses `[...]` as `Value::Vector` already; conversion path
  needs the new variant.
- `src/runtime.rs` `eval` keyword arm: add `WatAST::Vector(_, span)
  => Err(MalformedForm { reason: "vector literals at value
  position are not supported (arc 167 scope: vectors consumed only
  in fn/defn signatures); a future arc enables vector literals as
  Value::Vec values.", span })`
- `src/check.rs` `infer` arm: parallel error
- Match-arm sweep: cargo build will name every site that needs an
  explicit `WatAST::Vector` arm. Most sites will just need
  `_ => ...` fallback or a no-op `WatAST::Vector(_, _) => ...`
  that passes the AST through. ~889 WatAST references workspace-
  wide; expect 5-15 sites needing explicit arms (eval, check,
  walkers, span helpers).

**Tests**:
- New `tests/wat_arc167_vector_ast.rs`:
  1. `[1 2 3]` parses as `WatAST::Vector` (probe via debug
     formatting or a substrate-level test).
  2. `[]` parses as empty Vector.
  3. Vector inside other forms (`(:wat::core::define ... [1 2 3])`)
     parses cleanly at the parser layer.
  4. Vector at value position errors with the
     "vector-literals-not-supported" message (test asserts the
     error text appears).
  5. Empty top-level Vector also errors clearly.

**Verification**:
- `cargo build --release --workspace` green
- `cargo test --release --workspace --no-fail-fast` 0 failed
  (existing tests unaffected — they don't use `[...]` syntax yet;
  the new error arm only fires on programs that include vectors at
  value positions, which existing tests don't)

### Slice 2 — fn-sig vector consumer + walker + defn macro

**Goal**: fn accepts ONLY the new vector-shape signature. Legacy
nested-sig `((x :T) (y :T) -> :T)` triggers the migration walker.

**Substrate edits**:
- `src/runtime.rs`: rewrite `parse_fn_signature` for vector shape:
  - Sig argument must be `WatAST::Vector` (the `[name <- :T name <- :T]` body)
  - Vector body is a flat sequence of (name :Symbol, `<-` :Symbol, type :Keyword) triples
  - The `-> :T` arrow + return type are SIBLING positions of the vector, NOT inside it
  - Form structure: `(:wat::core::fn [args-vec] -> :T body)` — 5 elements
- `src/check.rs`: rewrite `parse_fn_signature_for_check` similarly
- `src/check.rs`: add `BareLegacyFnSignature { span: Span }` variant
  + Display impl + walker `walk_for_legacy_fn_signature` + registration
  in `check_program`. Walker fires fatal on
  `(:wat::core::fn (...) ...)` shape (List-as-first-arg to fn).
  The migration message:

  ```
  fn signature must be a vector binding form `[name <- :T name <- :T ...] -> :Ret`.
  Got legacy nested-sig list `((x :T) (y :T) -> :T)`.

  Migration:
    - Each `(p :T)` pair becomes `p <- :T` inside a `[...]` vector.
    - The `-> :R` arrow + return type remain siblings of the vector
      (NOT inside the vector).
    - The new shape arrows-as-duals: `<-` consumes (input type),
      `->` produces (output type).

  Example:
    Before:  (:wat::core::fn ((x :wat::core::i64) (y :wat::core::i64) -> :wat::core::i64) body)
    After:   (:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64 body)

    Defn equivalent (defn is a wat macro composing def + fn):
    Before:  (:wat::core::defn :name ((x :T) -> :T) body)
    After:   (:wat::core::defn :name [x <- :T] -> :T body)
  ```

- `wat/core.wat`: defn macro shape changes to accept 4 args
  (name, sig-vector, arrow-and-ret, body) and emits the canonical
  fn form.

**Tests**:
- New `tests/wat_arc167_fn_flat_signature.rs`:
  1. fn with flat shape compiles + evaluates
  2. defn with flat shape compiles + evaluates (composition test)
  3. recursive defn with flat shape (arc 166's recursive binding
     survives the macro shape change)
  4. legacy nested-sig fn fires `BareLegacyFnSignature` walker
  5. legacy nested-sig defn also fires (through macro expansion)
  6. type-mismatch in body surfaces ReturnTypeMismatch with correct
     span pointing at the body
  7. zero-arg fn `[] -> :T body`
  8. reflection on flat-defn: `lookup-define` resolves
  9. error message text test — assert the migration hint text
     appears verbatim in the diagnostic so future regressions in
     the message clarity are caught

**Verification**:
- New tests 1, 2, 3, 6, 7, 8, 9 pass
- Tests 4, 5 fire walker (PASS the assertion)
- Workspace tests fail elsewhere on legacy fn/defn callers — that
  failure stream IS slice 3's input

### Slice 3 — sweep all fn/defn callsites (substrate-as-teacher)

**Goal**: zero legacy nested-sig sites remain in the workspace.
cargo test green throughout.

**Discipline**: per FM 15, the failures from slice 2 ARE the
migration brief. Sonnet runs cargo test, reads the
`BareLegacyFnSignature` diagnostics, applies the mechanical
translation the diagnostic describes, re-runs, repeats. No
pre-enumeration of categories.

**Sites expected** (from arcs 155 / 159 precedent):
- `wat/*.wat` sources
- `wat-tests/*.wat` test fixtures
- `tests/wat_*.rs` embedded wat strings (notably
  `tests/wat_arc166_defn.rs`)
- `examples/*/`
- `crates/*/wat-tests/`
- Possibly `crates/wat-macros/` codegen

**Verification**:
- `cargo test --release --workspace --no-fail-fast`: 0 failed
- `grep -rn "((:wat::core::fn (" src/ wat/ tests/ ...`: 0 hits
  (verifies no legacy callsites remain)

### Slice 4 — walker hard-retirement (no cruft per user direction)

**Goal**: zero substrate trace of the legacy nested-sig migration
machinery.

**Edits**:
- DELETE `BareLegacyFnSignature` variant from `CheckError`
- DELETE its `Display` impl
- DELETE `walk_for_legacy_fn_signature` walker body
- DELETE `validate_legacy_fn_signature` registration in
  `check_program`
- DELETE the migration-hint string constants
- RETIRE slice 2's tests #4 + #5 + #9 (the walker no longer fires;
  the assertions become vacuous). Replace with assertions that
  legacy nested-sig fires whatever generic parser error remains
  (the standard `MalformedForm` from the rewritten parser).

**Verification**:
- `cargo test --release --workspace --no-fail-fast`: 0 failed
- `grep -rn "BareLegacyFnSignature" src/ tests/`: 0 hits
- `grep -rn "legacy nested-sig" src/ docs/`: 0 hits in src/, only
  permitted in this DESIGN.md and the historical INSCRIPTION

### Slice 5 — closure

- SCORE-SLICE-{1..4} aggregated (or per-slice; orchestrator
  decides at writing time)
- INSCRIPTION with pre-INSCRIPTION grep clean
- 058 changelog row in trading-lab repo
- USER-GUIDE update: defn + fn sections show the new flat shape;
  legacy nested-sig examples removed (no breadcrumbs per user
  direction "doesn't leave cruft")
- WAT-CHEATSHEET update (if it documents fn-sig shape)

## Why arc 167 is the right shape

Four questions on Path-A-clean-break + full-walker-retirement:
- **Obvious?** ✓ One canonical fn-sig shape; no mid-flight ambiguity
- **Simple?** ✓ Single substrate change; sweep is mechanical
- **Honest?** ✓ Hard retirement leaves zero scaffolding; "fn shape
  is `[...] -> :T body`, period"
- **Good UX?** ✓ Cleaner reading; arrows-as-duals teaches itself

All four favor the chosen path. The "doesn't leave cruft" constraint
forces the walker hard-retirement (slice 2) — without it, arc 167
ends like arc 162's leftover problem.

## Cross-references

- **Arc 154** — `let*` retirement / sequential-let. Pattern: clean
  break, walker fires during sweep window. Arc 167 mirrors the
  walker shape.
- **Arc 155** — `lambda → fn` rename, `:fn(...) → :wat::core::Fn(...)`
  parametric type. Pattern: clean break, walker + sweep. Arc 162
  closed the leftover internal-identifier residue (arc 167 avoids
  that pattern via slice 2's hard retirement).
- **Arc 159** — `let` per-binding `:T` retired. Closest precedent:
  same shape (substrate accepts new shape, walker fires legacy,
  sweep across many sites, walker retires after sweep).
- **Arc 162** — internal-identifier sweep that arc 155 left open.
  ANTI-PATTERN: arc 167 explicitly avoids leaving internals dirty
  by deleting walker scaffolding in slice 2.
- **Arc 166** — defn shipped; arc 167 evolves defn's surface. The
  recursive-name-binding fix (Gap A) carries through.
- **`wat-rs/docs/arc/2026/05/166-core-defn-form/FUTURE-ITERATION.md`**
  — captured the arrows-as-duals vision; this arc is its first
  rung on the iteration ladder.

## Calibration prediction

Sonnet sweep similar in shape to arc 155 (476 sites) or arc 159
(~951 sites for let). fn is less common than let in user code but
more common in test fixtures. Predict ~150-400 sites. Sonnet
runtime predicted 60-120 min for slice 1 (substrate + sweep);
slice 2 mechanical retirement ~15-30 min. Total arc ~90-150 min.
