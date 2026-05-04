# Arc 150 — Variadic `:wat::core::define` (`& rest` rest-params)

**Status:** drafted 2026-05-03 mid-arc-148-slice-4-unblock.
Arc 148's slice 4 cannot proceed without it.

User direction 2026-05-03 (this session):

> *"the substrate has variadic defmacros but not variadic defines —
> that's an arbitrary inconsistency. defmacros (which expand) can
> use `& rest`; defines (which evaluate) can't. No principled
> reason; just nobody needed it before."*

> *"sounds you like found the path — let's map it out and watch
> sonnet beat the dungeon"*

## What this arc fixes

`:wat::core::defmacro` has supported variadic rest-params (`& rest`)
since arc 029 — see `tests/wat_variadic_defmacro.rs` and the
`MacroDef.rest_param: Option<String>` field at `src/macros.rs:68`.

`:wat::core::define` has NOT. Today's `Function` struct at
`src/runtime.rs:499` has only `params: Vec<String>` + `param_types:
Vec<TypeExpr>` — no rest_param field. `apply_function` at
`src/runtime.rs:12865` enforces strict arity:
`if cur_args.len() != cur_func.params.len() { return ArityMismatch }`.

This is a foundation gap surfaced by arc 148 slice 4's attempt to
write:

```scheme
(:wat::core::define
  (:wat::core::+ & (xs :wat::core::Vector<numeric>) -> :numeric)
  (:wat::core::reduce :wat::core::+,2 (:first xs) (:rest xs)))
```

— which is the natural shape for a polymorphic variadic arithmetic
surface. The wat-define syntax doesn't accept `& xs`; arc 148 was
about to fall back to either (a) a defmacro-with-runtime-if branching
shape (heavy; defmacros are pure templates) or (b) a Rust substrate
primitive (works but contradicts the user's "composition lives in
wat" preference).

The durable answer (per the user's "eliminate failure domains; don't
bridge" discipline + the foundation work for arc 109 wind-down):
**add rest-param support to `:wat::core::define`,** mirroring the
defmacro syntax exactly.

## Why this arc must land before arc 148 slice 4

Arc 148 slice 4's variadic arithmetic surface depends on the
`(:wat::core::define (verb & (xs :Vector<...>) -> ...) body)` pattern
being legal. Without arc 150, slice 4 must use a less-ergonomic
fallback:

- Defmacro-with-runtime-branching: requires `(:wat::core::if (empty? xs)
  identity ...)` + recompute the constructed Vec at every branch
  (defmacro bodies are pure templates; can't inspect arg count at
  expansion time).
- Rust substrate Primitive: variadic logic in Rust; works but the
  composition isn't in wat where it should live.

Both are bridges. Arc 150 is the foundation fix.

## Beyond arc 148

Arc 150 unlocks variadic surfaces for ANY future wat-defined function:
- Format / interpolation primitives (`(:format "x = {}, y = {}" x y)`)
- Logging / tracing surfaces (`(:log :info "event" k1 v1 k2 v2)`)
- Composition combinators (`(:pipe a b c d)` for value pipelines)
- Test harness extensions (`(:assert-all (== a b) (> b c) (< c d))`)

The surface gap closes; LLM-generated wat code can follow Lisp
conventions without hitting "but functions can't be variadic" walls.

## What ships

### Substrate addition

1. **`Function` struct extension** (`src/runtime.rs:499`)
   - Add `rest_param: Option<String>` field
   - Add `rest_param_type: Option<TypeExpr>` field
   - Mirrors `MacroDef`'s shape (`src/macros.rs:68`)

2. **`parse_define_signature`** (`src/runtime.rs:1736`)
   - Accept `& (name :Type)` pattern after fixed params
   - Validate: `&` followed by exactly one binder; type required
     (mirrors `parse_defmacro_signature` at `src/macros.rs:380-420`)
   - Populate `rest_param` + `rest_param_type` in the parsed Function

3. **`apply_function`** (`src/runtime.rs:12865`)
   - When `rest_param.is_some()`: accept `args.len() >= params.len()`
     (instead of strict equality)
   - Collect extra args into a `Vec<Value>` value
   - Bind to `rest_param` name in the call env
   - When `rest_param.is_none()`: existing strict-arity behavior unchanged

4. **`TypeScheme` + `derive_scheme_from_function`** (`src/check.rs:63`,
   `src/check.rs:8397`)
   - Extend `TypeScheme` to express variadic: a `rest_type:
     Option<TypeExpr>` field (or sibling shape)
   - Update `derive_scheme_from_function` to populate the new field
     when the underlying Function has a rest-param

5. **Call-site type checking** (`src/check.rs` — `infer_list` for
   `Function` callees)
   - When the resolved scheme has a rest_type: accept `args.len() >=
     params.len()`
   - Type-check fixed args against `params`; type-check each rest-arg
     against the element type extracted from `rest_type` (which is
     `Vector<T>` — the T is the per-arg expected type)

### Tests

New test file: `tests/wat_arc150_variadic_define.rs`. Coverage:

- Variadic define with zero rest-args (just fixed params)
- Variadic define with one rest-arg
- Variadic define with many rest-args
- Variadic define with NO fixed params (only rest)
- Arity error: caller passes fewer than `params.len()` args
- Type error: rest-arg's actual type doesn't match the declared
  element type
- Reflection: `signature-of` on a variadic define returns the rest
  shape correctly (carries the variadic info for arc 144 reflection
  consumers)
- Apply pattern: variadic define + reduce over rest args (the canonical
  pattern arc 148 slice 4 needs)

### What does NOT change

- `:wat::core::lambda` — lambdas stay fixed-arity for now (lambdas
  don't have signatures in the substrate; variadic lambda would be a
  separate substrate addition; out of scope)
- `:wat::core::defmacro` — already variadic; UNCHANGED
- Strict-arity defines — UNCHANGED behavior; rest_param is optional

## Slice plan

### Slice 1 — Implement variadic define + tests

One cohesive slice. Touches:
- `src/runtime.rs` (Function struct + parse_define_signature +
  apply_function)
- `src/check.rs` (TypeScheme + derive_scheme_from_function +
  call-site inference)
- `tests/wat_arc150_variadic_define.rs` (new test file)

Predicted MEDIUM substrate slice (~300-500 LOC + tests).

Mode A: ~50-80 min. Mode B-substrate-coupling-surprise: scope-related
edges in check.rs's call-site inference. Time-box 120 min.

### Slice 2 — Closure

INSCRIPTION + 058 row + USER-GUIDE entry showing variadic define
shape + cross-reference to arc 148 slice 4 unblock + cross-reference
to defmacro symmetry. Small.

## What this unlocks

- **Arc 148 slice 4** (numeric arithmetic migration) — the variadic
  wat function for `:wat::core::+` etc. now expressible per the
  locked DESIGN
- **Future variadic surfaces** across the substrate (format, log,
  pipe, etc.)
- **The "comma-typed funcs are crutches" rule** strengthens — variadic
  surfaces can be wat-level functions without falling back to
  comma-tagged binary direct-call leaves

## Cross-references

- arc 148 — arithmetic + comparison correction (slice 4 BLOCKED on
  this arc)
- arc 029 — variadic defmacro (the precedent; pattern to mirror)
- `src/macros.rs:380-420` — `parse_defmacro_signature` (template for
  parser change)
- `src/macros.rs:68` — `MacroDef.rest_param` (template for Function
  field shape)
- COMPACTION-AMNESIA-RECOVERY § 12 — foundation work; eliminate
  failure domains; don't bridge

## Status notes

- DESIGN drafted 2026-05-03.
- Arc 148 slice 4 ON HOLD pending arc 150 closure.
- Arc 109 v1 closure now waits on arc 144 + arc 130 + arc 145 +
  arc 146 + arc 147 + arc 148 + **arc 150**. The chain extends; the
  foundation strengthens with each.
