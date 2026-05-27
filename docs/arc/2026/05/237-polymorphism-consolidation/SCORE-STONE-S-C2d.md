# SCORE — Stone S-C.2d — mint `:wat::Record/same-data?` (type-BLIND record data equality)

**Status:** COMPLETE. All targets met. Tree dirty (no commit — orchestrator commits).

## Scorecard

- [x] `:wat::Record/same-data?` minted (dispatch arm + eval fn + checker scheme).
- [x] `record_field_map` helper factored from `eval_record_to_map`; `eval_record_to_map` behavior unchanged.
- [x] `probe_arc237_sC2d_same_data` 6/6 (comp_* stayed green; samedata_* flipped RED → GREEN).
- [x] lib baseline preserved: 834/0 (no change).
- [x] arc 238 eq completeness: 8/8 (unaffected).
- [x] arc 227 defrecord/record->map regression: 35/35 (unaffected).

## Test result lines (exact)

```
cargo build --release -p wat 2>&1 | grep "^error"
  (no output — 0 errors)

cargo test --release --test probe_arc237_sC2d_same_data 2>&1 | grep "test result"
  test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --release --lib -p wat 2>&1 | grep "test result"
  test result: ok. 834 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.22s

cargo test --release --test probe_arc238_eq_completeness 2>&1 | grep "test result"
  test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

cargo test --release --test probe_arc227_stone2_defrecord 2>&1 | grep "test result"
  test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

## What was added

### 1. Dispatch arm — `src/runtime.rs` (next to `:wat::Record/assoc`, ~line 5349)

```rust
// Arc 237 Stone S-C.2d — type-BLIND record data equality.
// same-data? :: :wat::Record × :wat::Record -> :wat::core::bool
// Compares field-name→value maps (record->map); type-blind and flavor-blind.
// Distinct from `=` (type-strict, arc 238): Pt[0,0] same-data? Coord[0,0] → true.
":wat::Record/same-data?" => eval_record_same_data(args, list_span, env, sym),
```

### 2. `record_field_map` helper — `src/runtime.rs` (new fn before `eval_record_to_map`)

Factored from `eval_record_to_map`'s match body. Signature:

```rust
#[allow(clippy::mutable_key_type)]
fn record_field_map(
    v: Value,
    op: &str,
    span: &Span,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError>
```

Accepts an already-evaluated `Value`; handles the or-pattern `wat__holon__Record | wat__Record`
(Stone S-C.2c); returns `Value::wat__std__HashMap`. TypeMismatch on non-record input.
`eval_record_to_map` now: evals arg, then calls `record_field_map(v, OP, list_span, sym)`.
Observable behavior of `eval_record_to_map` is IDENTICAL — all 35 defrecord/record->map probes green.

### 3. `eval_record_same_data` — `src/runtime.rs` (new fn after `eval_record_to_map`)

```rust
fn eval_record_same_data(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError>
```

Arity 2; evals both args; calls `record_field_map` on each; delegates to
`values_equal(&map_a, &map_b) == Some(true)` — reuses arc 238's total HashMap equality,
never re-implements it. Returns `Value::bool`.

### 4. Checker scheme — `src/check.rs` (after `:wat::Record/assoc` registration, ~line 19384)

```rust
env.register(
    ":wat::Record/same-data?".into(),
    TypeScheme {
        type_params: vec![],
        params: vec![record_ty(), record_ty()],
        ret: bool_ty(),
        rest_param_type: None,
    },
);
```

`:wat::Record` umbrella accepts any record (base or holonic, any class). Fixed 2-arity;
returns `:wat::core::bool`. No type params needed (no polymorphic T — both inputs and output
are fully determined).

## Honest deltas

- `src/runtime.rs`: +~60 lines net (new `record_field_map` helper ~35 lines; new
  `eval_record_same_data` ~25 lines; `eval_record_to_map` body shortened by ~30 lines to a
  single `record_field_map` call; dispatch arm +5 lines with comment).
- `src/check.rs`: +14 lines (scheme registration with comment block).
- `tests/probe_arc237_sC2d_same_data.rs`: pre-existing on disk (untracked); unchanged.

No changes to `values_equal`, holon-rs, or any other file.

## Refactor shape

`eval_record_to_map` previously contained the entire match body inline. The refactor extracts
that body into `record_field_map(v, op, span, sym)` — a value-in / value-out helper that takes
`op: &str` and `span: &Span` for error messages. `eval_record_to_map` retains its arity check
and `eval_inner` call, then delegates. `eval_record_same_data` reuses the same helper on both
args, making the composition `record_field_map(a) + record_field_map(b) + values_equal` explicit
and zero-duplication.

## git status --short

```
 M src/check.rs
 M src/runtime.rs
?? tests/probe_arc237_sC2d_same_data.rs
```
