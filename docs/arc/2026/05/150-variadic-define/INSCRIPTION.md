# Arc 150 — Variadic `:wat::core::define` — INSCRIPTION

## Status

Shipped 2026-05-03. One sonnet sweep (slice 1, ~19 min wall-clock)
+ one orchestrator-driven cleanup (sibling-map → inline TypeScheme
field, ~5 min). Slice 2 is this paperwork.

The arbitrary asymmetry between `:wat::core::defmacro` (variadic-
capable since arc 029) and `:wat::core::define` (strict-arity only)
is closed. Defines now accept `& (rest :Vector<T>)` rest-params
mirroring the defmacro syntax exactly.

## Why this arc existed

Arc 148 slice 4 attempted to write the polymorphic variadic
arithmetic surface as:

```scheme
(:wat::core::define
  (:wat::core::+ & (xs :wat::core::Vector<numeric>) -> :numeric)
  (:wat::core::reduce :wat::core::+,2 (:first xs) (:rest xs)))
```

The wat parser rejected `& xs` on `:wat::core::define`. Defmacros
supported variadic; defines didn't. Per the user's discipline
("eliminate failure domains; don't bridge"), arc 150 closes the gap
at the substrate layer rather than working around it.

The substrate-as-teacher cascade — arc 144 made entities
reflectable; arc 146 made polymorphism a first-class entity; arc
148's slice 4 attempt surfaced a 30-arc-old foundational gap that
nobody had needed to fill until variadic user functions became a
real use case.

## What this arc adds

### `Function` struct extension (`src/runtime.rs:499`)

Two new fields, mirroring `MacroDef`'s shape:

```rust
pub struct Function {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub type_params: Vec<String>,
    pub param_types: Vec<TypeExpr>,
    pub ret_type: TypeExpr,
    pub body: Arc<WatAST>,
    pub closed_env: Option<Environment>,
    // Arc 150 additions:
    pub rest_param: Option<String>,
    pub rest_param_type: Option<TypeExpr>,
}
```

Both default to `None` for existing strict-arity defines (no
behavior change to current tests).

### `parse_define_signature` (`src/runtime.rs:1736`)

Mirrors `parse_defmacro_signature` (`src/macros.rs:380-440`):

```scheme
(:wat::core::define
  (:user::main (a :i64) (b :i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  body)
```

Validations:
- `&` followed by exactly one binder `(name :Type)`
- Type required (rest-param without type is parse error)
- Type must be `Vector<T>` shape (`Vec<T>`, `Vector<T>`, or
  `wat::core::Vector<T>`)
- `&` may appear at most once
- Rest-binder is the LAST element (no fixed param after rest)

### `apply_function` (`src/runtime.rs:12865`)

Variadic arity check:
- `rest_param.is_none()` → strict-arity equality (existing behavior)
- `rest_param.is_some()` → `args.len() >= params.len()`

Variadic arg binding:
- Bind the first `params.len()` args positionally
- Collect remainder into `Value::Vec(Arc::new(rest))`
- Bind to `rest_param` name in the call env

Tail-call loop integrity preserved (cur_args rebuilt fresh per
iteration).

### `function_to_signature_ast` extension

The signature-AST renderer (consumed by arc 144's `signature-of`
reflection primitive) emits `& (rest :Vector<T>)` for variadic
defines, mirroring `macrodef_to_signature_ast`. Reflection round-
trips correctly.

### `TypeScheme.rest_param_type` (`src/check.rs:75`)

```rust
pub struct TypeScheme {
    pub type_params: Vec<String>,
    pub params: Vec<TypeExpr>,
    pub ret: TypeExpr,
    pub rest_param_type: Option<TypeExpr>,  // arc 150
}
```

Inline field. All 215 existing substrate-primitive struct literals
carry `rest_param_type: None,` (mechanically populated 2026-05-03
via brace-depth-tracking python script; see commit `6416b76`).

### `derive_scheme_from_function` extension

Populates `rest_param_type: func.rest_param_type.clone()` so
variadic info propagates from runtime Function → check-side
TypeScheme uniformly.

### Call-site type checking (`src/check.rs` — `infer_list`)

When the resolved `scheme.rest_param_type` is `Some(Vector<T>)`:
- Accepts `args.len() >= params.len()`
- Type-checks the first `params.len()` args against `params`
- Type-checks each rest-arg against T extracted from `Vector<T>`
- Returns the scheme's `ret` unchanged

### `check_function_body` rest-name binding

Variadic function bodies bind the rest-name to its `Vector<T>`
type in body-locals so intra-body uses (`(:length xs)`,
`(:foldl xs ...)`, etc.) resolve correctly. Required for the
canonical foldl-over-rest pattern to type-check.

## Tests

`tests/wat_arc150_variadic_define.rs` (405 LOC, 16 tests):
- Variadic with zero/one/many rest-args
- Variadic with no fixed params
- Arity error: caller passes fewer than `params.len()` args
- Type error: rest-arg's actual type doesn't unify with declared T
- Reflection: `signature-of` round-trips the variadic shape
- Apply pattern: variadic + reduce-over-rest (the arc 148 slice 4 shape)
- Negative parse tests: double `&`, `&` without binder, fixed-after-rest, non-Vector rest type
- Regression guards: strict-arity defines unchanged

All pass.

## Honest deltas (recorded for the arc record)

### Slice 1 Delta 1 — TypeScheme sibling-map (since folded back)

Sonnet's slice-1 implementation initially used a sibling registry
(`CheckEnv.variadic_rest_types: HashMap<String, TypeExpr>`) instead
of the inline `TypeScheme.rest_param_type` field. Driven by a
mistaken assumption that mass-edit tooling (sed/perl/python) wasn't
available in the harness. Functionally equivalent but architecturally
non-ideal.

Per user direction ("if there is something we deferred we do it
now"), the orchestrator verified tool availability empirically
(`which sed perl python3` returns paths) and folded the sibling map
back into the TypeScheme inline field. Mechanical execution: ~5
minutes via a 24-line python state-tracking script that walked
brace-depth from each `TypeScheme {` opening and inserted
`rest_param_type: None,` at the matching closing `},` (215/215
sites updated cleanly; zero false positives; multi-line `ret:`
cases all caught).

Lesson recorded: future briefs include "verify sed/perl/python
access if mass-edit is needed" as a default. Sonnet should
empirically test tool availability before claiming a constraint.
Cost of testing: ~2 seconds. Cost of wrong assumption: a follow-up
arc to clean up.

### Slice 1 Delta 2 — `check_function_body` rest-binding (within scope)

Body must bind the rest-name in locals or `(:length xs)`-style
intra-body uses fail with `<unresolved>`. One-line addition;
identified on first body-uses-rest test failure; fixed within the
slice.

### Slice 1 Delta 3 — `function_to_signature_ast` rendering (within scope)

Signature renderer extended to emit `& (rest :Vector<T>)` for
variadic defines, mirroring macrodef equivalent. Required by
reflection round-trip test.

## What this arc unlocks

### Arc 148 slice 4 — RESUMES

The polymorphic variadic arithmetic surface is now expressible:

```scheme
(:wat::core::define
  (:wat::core::+ & (xs :wat::core::Vector<numeric>) -> :numeric)
  (:wat::core::reduce :wat::core::+,2 (:first xs) (:rest xs)))
```

Slice 4 ships per the locked DESIGN.

### Arc 141 — docstrings (DESIGN locked, impl pending)

The "extend Function carrier" pattern arc 150 established is now
exercised + structurally clean. Arc 141's docstring field
(`pub docstring: Option<String>`) follows the same shape:
- Add field to Function struct
- Add field to TypeScheme (for tooling)
- Extend parse_define_signature (and parse_defmacro/typealias/etc.)
- Mass-update existing struct literals via the same python script
  pattern

Arc 141 becomes a pattern-application slice rather than a
substrate-architecture slice.

### Future variadic surfaces

Lisp-natural call shapes are now expressible at the wat level:
- Format / interpolation: `(:format "x={}, y={}" x y)`
- Logging: `(:log :info "event" k1 v1 k2 v2)`
- Composition: `(:pipe a b c d)`
- Test harness: `(:assert-all (== a b) (> b c) (< c d))`

LLM-generated wat code follows Lisp variadic conventions without
hitting "but functions can't be variadic" walls.

## What's NOT in this arc

- `:wat::core::lambda` stays fixed-arity (lambdas don't have
  signatures in the substrate; variadic lambda would be a separate
  substrate addition; out of scope)
- `:wat::core::defmacro` UNCHANGED (already variadic since arc 029)
- The 215-site mechanical update touched only TypeScheme literals;
  no behavior change in any existing substrate primitive

## Cross-references

- arc 029 — variadic defmacro (the precedent; pattern mirrored)
- arc 144 — Binding enum + lookup-form (reflection consumer)
- arc 148 — arithmetic + comparison correction (slice 4 was the
  motivating use case; resumes here)
- arc 141 — core form docstrings (future arc; pattern beneficiary)
- COMPACTION-AMNESIA-RECOVERY § 12 — foundation discipline
  (eliminate failure domains; don't bridge)

## Calibration record

- **Slice 1 wall clock**: ~19 min (vs predicted 50-80 min Mode A;
  16% of 120-min time-box)
- **Sibling-map cleanup wall clock**: ~5 min (orchestrator-driven;
  mechanical sed + python + 4 small Edits)
- **Total arc 150 substrate work**: ~24 min from spawn to closure
- **LOC**: 405 (test file) + ~350 substrate (after cleanup) ≈ 755
- **Honest deltas**: 3 surfaced; all addressed; one (Delta 1) led
  to a follow-up cleanup that strengthened the architecture

## Status

Arc 150 closes here. Arc 148 slice 4 unblocked. Arc 109 v1 closure
queue continues:

```
arc 148 slice 4 (RESUMES NEXT) — numeric arithmetic migration
arc 148 slice 6 (closure)
arc 146 slice 5 (closure)
arc 144 slice 4 + closure
arc 130 reland + closure
arc 145 (slices 1-2)
arc 147 (slices 1-N+2)
arc 141 (docstrings — pattern-application slice atop arc 150)
arc 151 (wat-macros wrapper honest message)
arc 109 v1 closure
```

The chain extends. Each arc compounds. The foundation strengthens.
