# SCORE — Stone 237.7b-i — `:wat::core::empty?` reborn as a `∀T` intrinsic

## Probe result (verbatim)

```
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Green-gate result

```
green-gate: PASS (test-build clean + lib baseline green)
```

- `cargo build --release --tests --workspace` → Finished (0 errors, pre-existing warnings only)
- `cargo test --release --lib -p wat` → `test result: ok. 834 passed; 0 failed`

## Diff stat

```
 src/check.rs   | 16 ++++++++++++++++
 src/runtime.rs | 44 ++++++++++++++++++++++++++++++++++++++++++++
 wat/core.wat   |  7 +++----
 3 files changed, 63 insertions(+), 4 deletions(-)
```

## define-dispatch count

```
grep -c "define-dispatch :wat::core::empty?" wat/core.wat
0
```

## What was done

Three files touched, exactly as scoped:

1. **`src/check.rs`** — registered `:wat::core::empty?` in `register_builtins` immediately after
   the `:wat::core::length` registration (line ~19619). TypeScheme: `type_params: ["T"]`,
   `params: [t_var()]`, `ret: bool_ty()`, `rest_param_type: None`. Mirrors length exactly
   with `bool_ty()` in place of `i64_ty()`.

2. **`src/runtime.rs`** — added dispatch arm `":wat::core::empty?" => eval_empty(...)` next to
   the length arm (~line 5323). Added `eval_empty` function after `eval_length` (~line 16183):
   arity-check 1 → eval arg → match `Value::Vec(xs)` → `bool(xs.is_empty())`,
   `Value::wat__std__HashMap(m)` → `bool(m.is_empty())`,
   `Value::wat__std__HashSet(s)` → `bool(s.is_empty())`,
   else teaching `RuntimeError::TypeMismatch` (same shape as eval_length's other arm).

3. **`wat/core.wat`** — deleted the `(:wat::core::define-dispatch :wat::core::empty? ...)` decl
   (3 lines + surrounding blank line). Replaced with a tombstone comment matching the
   pattern established by Stone 237.7a for `:wat::core::length`.

## STOP triggers

None triggered. No hidden coupling, no List arm temptation, no registry or holon-rs touch.
The decl delete routed `empty?` call-site resolution cleanly to the new builtin via ordinary
scheme lookup.
