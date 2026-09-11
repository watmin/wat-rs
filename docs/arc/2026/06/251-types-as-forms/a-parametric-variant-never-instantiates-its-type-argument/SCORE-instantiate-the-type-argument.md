# SCORE — instantiate the type argument

Struck. Floor not run. Clippy not run. Not committed.

```
cargo build --release                                          BUILD_EXIT=0
cargo nextest run --release -E 'binary_id(wat::types)'         614 passed, 5 skipped, NEXTEST_EXIT=0
```

## Site ① — `{:keys}` (`src/check.rs` `process_let_binding`)

**Wrong.** `TypeExpr::Parametric { head, .. }` dropped the args, looked up the
TypeDef by head, and bound each requested field to the **declared** field
type. `(:u::Cell :- [:wat::core::i64])` therefore bound `x` as `:X`.

One arm serves record, struct, and variant (the A-2 singleton-Enum widening).
All three failing `{:keys}` rows went through it.

**Changed.** Keep the args. After the TypeDef lookup, `instantiate_field_types`
zips `type_params` with the supplied args and `rename`s them through the
declared field types — the inverse of `parametric_decl_type`. Empty params
or empty args is a no-op (monomorphic / bare Path). `assignable` untouched.

## Site ② — variant `:T/field` accessor (`src/declare/register.rs` `register_enum_methods`)

**Wrong, and a measurement delta.** The brief names "the VARIANT field
accessor's return type" as if a scheme existed and failed to instantiate.
Pre-fix `--check` of `(:u::Demo.Has/has d)` was **not**
`ReturnTypeMismatch` / "body produces :T":

```
#wat.resolve/UnresolvedReferences
  :path ":u::Demo.Has/has"
  :context "call head — not a builtin, not a registered function"
```

`register_aggregate_methods` mints `:Record/field` with `type_params` so
`instantiate` at the call site substitutes. `register_enum_methods` minted
the ctor (via `parametric_decl_type`) and **never minted accessors**.
That is the enum-path omission.

The NOTE's "body produces :T" for ACCESSOR did not reproduce on
`:Enum.Variant/field`. Bare `(:has d)` (keyword-accessor fall-through)
`--check`s EXIT 0 by returning a **fresh var** that unifies with i64 —
a false green, not `:T`. Different shape; see STOP-4.

**Changed.** After each tagged-variant ctor, mint `:Enum.Variant/field`
with the same scheme shape as the aggregate accessor: `type_params` = the
parent's list, param = `parametric_decl_type(constructor_path, …)`, ret =
the declared field type. Body is `(:field self)` — `struct-field` is
Aggregate-only; the runtime read path for Enum is `keyword_accessor_enum`.
`instantiate` at the call site now does the substitution. No new
`assignable` arm.

## Grid (real exit codes, this binary)

Pre-fix measured on `target/release/wat` before the edit; post-fix on the
same path after `cargo build --release`.

| row | pre-fix | post-fix |
|---|---|---|
| parametric RECORD  `{:keys}`    | EXIT 1, body produces `:X` | **EXIT 0** |
| parametric STRUCT  `{:keys}`    | EXIT 1, body produces `:X` | **EXIT 0** |
| parametric VARIANT `{:keys}`    | EXIT 1, body produces `:T` | **EXIT 0** |
| parametric VARIANT accessor     | EXIT 1, UnresolvedReference `:u::Demo.Has/has` | **EXIT 0** |
| parametric RECORD  accessor     | EXIT 0 | **EXIT 0** |
| parametric ENUM via match       | EXIT 0 | **EXIT 0** |
| NON-parametric record `{:keys}` | EXIT 0 | **EXIT 0** |
| NON-parametric variant `{:keys}`| EXIT 0 | **EXIT 0** |
| Demo.Has → Demo slot            | EXIT 0 | **EXIT 0** |
| Demo → Demo.Has slot            | EXIT 1, TypeMismatch | **EXIT 1**, TypeMismatch |

Probes: `tests/types/probe_arc251_instantiate_the_type_argument.rs` (11 tests,
all PASS inside the 614). Runtime twin `parametric_variant_accessor_runs`
prints `42`.

## STOP triggers — none fired

- **STOP-1** (subtyping / `assignable`) — not touched. `git diff` is
  `src/check.rs` keys-destructure + `instantiate_field_types`, and
  `src/declare/register.rs` accessor mint. Widening controls hold.
- **STOP-2** (same-head covariance) — not needed; substitution is
  param→arg at the use site, not a variance change.
- **STOP-3** (floor failures > 8) — floor not run (brief). `wat::types`
  614/5 skipped, no new red in that binary.
- **STOP-4** (third site, same shape) — none found. Remaining
  `Parametric { head, .. }` hits in `check.rs` are classifiers (is this
  a variant? is this Option?), not "lookup TypeDef, use declared field
  types". Reported below, not fixed.

## Related, not the same shape (not STOP-4)

Bare keyword accessor `(:has d)` on a parametric variant still goes
through the HARVEST placeholder (`check.rs` ~5831) and returns a fresh
var. Pre-fix it `--check`d EXIT 0 against an i64 return — it never had
the field type, so it could not print `:T`. Fixing that would be a
different stone (give the placeholder the instantiated field type, or
retire it now that `:Enum.Variant/field` exists). Four-line repro:

```wat
(:wat::core::defenum :u::Demo :- [T] :wat::enum::Pure :Has [has <- :T] :HasNot [])
(:wat::core::defn :u::fb [d <- (:u::Demo.Has :- [:wat::core::i64])] -> :wat::core::i64
  (:has d))
```

`--check` EXIT 0 both before and after this stone.

## Uncertain

- Accessor body is `(:field self)` rather than `struct-field`. Runtime
  confirmed on the run row. If a later stone wants a primitive Enum
  field-at, that is new work.
- Ctor still uses the **parent's** full `type_params` for
  `parametric_decl_type`; the singleton TypeDef stores only consumed
  params. Accessors match the ctor. Pre-existing split, not opened here.
