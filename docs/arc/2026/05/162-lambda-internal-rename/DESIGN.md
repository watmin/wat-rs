# Arc 162 — `lambda` internal-identifier rename to `fn`

**Status:** queued 2026-05-07 by orchestrator. Not yet started.

**Gates:** none (independent housekeeping). Open this arc after the
arc 159 / 160 / 161 chain closes.

## Background

Arc 155 retired the user-facing `:wat::core::lambda` keyword in favor
of `:wat::core::fn`. The retirement was complete at the surface:

- The wat keyword fires `BareLegacyLambda` / `BareLegacyLowercaseFn`
  at check time
- All consumer wat code uses `:wat::core::fn`
- Walker bodies + dispatch arms recognize only `:wat::core::fn`

Arc 155's scope was the user-visible surface. The Rust-level
identifier naming (Value variant, helper functions, debug strings,
test file naming) was NOT swept — arc 155 deliberately scoped to
the surface change.

User audit 2026-05-07 surfaced the gap: *"did we kill off lambda
in favor of fn?... i see a bunch of lambda refs still"*. Honest
split: surface dead; internals carry the legacy name. Arc 162
closes the internal rename so the substrate's naming matches
its surface.

## Scope

Mechanical rename across Rust source. ~15-20 sites total.

### Categories

1. **Value variant** (load-bearing):
   `Value::wat__core__lambda(Arc<Function>)` →
   `Value::wat__core__fn(Arc<Function>)` in `src/runtime.rs:159`.
   Every match arm where Value is destructured updates
   (`runtime.rs`, `freeze.rs`, `edn_shim.rs`, `check.rs`).

2. **Type-name string**:
   `"wat::core::lambda"` (returned by `Value::type_name()`) →
   `"wat::core::fn"` in `src/runtime.rs:465`. Affects error
   message surface; check error-message tests don't snapshot
   this string (verify pre-flight).

3. **Helper functions**:
   - `parse_lambda_signature` → `parse_fn_signature` (`runtime.rs:3798`)
   - `parse_lambda_signature_for_check` → `parse_fn_signature_for_check` (`check.rs:9492`)

4. **Walker helpers** (arc 134 family):
   - `spawn_thread_lambda_body_has_no_recv` → `spawn_thread_fn_body_has_no_recv`
   - `rhs_spawn_lambda_has_no_recv` → `rhs_spawn_fn_has_no_recv`
   - Local variable names (`lambda_call`, `lambda_head`, `body_forms`
     in `rhs_spawn_lambda_has_no_recv`) update accordingly

5. **Debug-display strings**:
   - `<lambda@span>` → `<fn@span>` in `freeze.rs:162,191` and `check.rs:9474`
   - `":wat::kernel::spawn <lambda>"` → `":wat::kernel::spawn <fn>"` in `check.rs:7622`

6. **Error-message text**:
   - `"used outside any function or lambda body..."` → drop
     "or lambda" remnant in `check.rs:6823,6930` (only `fn` bodies
     exist post-arc-155)

7. **Test naming**:
   - `tests/wat_spawn_lambda.rs` → `tests/wat_spawn_fn.rs`
   - `fn arc_134_no_recv_in_lambda_body_does_not_fire` → `..._in_fn_body_...`
   - `fn typed_let_binding_with_lambda_value` → `..._with_fn_value`
   - test-internal string assertions update if they reference
     `"wat::core::lambda"` (verify pre-flight)

### Out of scope

- Comment text containing the word "lambda" in historical
  documentation or arc artifacts (immutable record per
  `feedback_inscription_immutable.md`)
- The `BareLegacyLambda` / `BareLegacyLowercaseFn` CheckError
  variants — these are arc-155's retirement diagnostics, named
  for the legacy form they reject; they intentionally carry the
  legacy name (precedent: arc 113 orphaned scaffolding)

## Slice plan

### Slice 1 — substrate sweep (sonnet, mechanical)

Rename in src/runtime.rs (Value variant + helper) → cargo build
(should fail at every match arm) → fix each arm → cargo build
clean → cargo test --release --workspace clean.

Estimated ~15-20 edited lines across ~6-8 files.

### Slice 2 — tests + closure paperwork (orchestrator)

Test file/fn renames (`tests/wat_spawn_lambda.rs` →
`tests/wat_spawn_fn.rs` + fn renames). Then INSCRIPTION + 058
changelog row.

## Why arc 162 is the right shape

Four questions:
- Obvious — naming the internal identifier matches the surface
  identifier; one consistent vocabulary
- Simple — mechanical rename; cargo's compile errors guide every
  edit; no design freedom
- Honest — closes a real internal-vs-surface mismatch the user
  audit surfaced
- Good UX — error messages improve (no more "wat::core::lambda"
  type-name surfacing in internals)

## Cross-references

- **Arc 155** — retired the surface; arc 162 closes the internal
  identifier rename arc 155 deliberately scoped out
- **Arc 109 § L** (task #253) — sibling internal-rename housekeeping
  (typealias/defmacro/newtype hyphenation); pattern is similar
  (mechanical rename, mass-edit, low risk)
- **Arc 113** — orphaned scaffolding precedent (`BareLegacyLambda`
  variant retains legacy name; rename does NOT touch it)
- **User audit 2026-05-07** — *"did we kill off lambda in favor
  of fn?... i see a bunch of lambda refs still"*

## When this opens

After arc 161 ships and arcs 159 / 160 / 161 close cleanly. The
current course (arc 159 binding-shape change + arc 160 / 161
substrate inference fixes) takes priority.
