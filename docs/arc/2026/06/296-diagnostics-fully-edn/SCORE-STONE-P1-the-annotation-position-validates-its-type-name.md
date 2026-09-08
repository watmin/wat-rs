# SCORE — STONE P-1: an annotation may not name a type that does not exist

No commit. Floor and clippy left to the orchestrator (EXPECTATIONS: rider
never runs them). Lands on Q's membership union. That was not reverted.

A `TypeExpr::Path` that is a named type (`::` or `.`) must be
`TypeEnv::contains` ∪ `is_builtin_primitive`. Type variables stay
`is_type_var_path` — the registry is never asked about `:T`.

## Where

Not `parse_type_expr` (STOP-1). After types AND user functions are registered
(`freeze/env.rs`, same posture as `validate_aggregate_containment`) so forward
references resolve. The bound set is the one assembled at
`parse.rs:505-540` and stored on `Function.type_params` / `TypeDef`.

The walk is `walk_type_expr` in `typevar.rs` — the recursion
`walk_free_type_vars` used to own, now shared. `walk_free_type_vars` is a
visitor. `grep -c "fn walk_"` is 1 → 2; the new fn **is** that recursion.

Error: `TypeErrorKind::UnknownNamedType { path }` — sibling shape to
`BareLegacyPrimitive` (the name is a field; span rides on the outer error).
Span is `rust_caller_span` (TypeExpr carries none; containment does the same).

## Honest deltas

1. **Reserved-prefix declarations are skipped.** First firing was
   `:rust::sqlite::Connection` inside stdlib — a TypeEnv hole, not a phantom.
   Without the skip, every program dies. User annotations of that name still
   refuse. Not a prefix blanket on the annotation; a skip of `:wat::`/`:rust::`
   *declarations*.
2. **defenum variant fields** ride the same TypeDef walk as aggregates. No
   second site.
3. **No forward-reference failure** on the seven fixtures (types register
   before functions; the type walk is post-registration).

## Expectations

| # | result |
|---|---|
| 1 | `--check` phantom param+return: **EXIT=1**, path `:usr::TotallyMadeUp` |
| 2 | `--check` phantom field: **EXIT=1**, path `:usr::AlsoMadeUp` |
| 3 | generic `:- [T]`: **EXIT=0** |
| 4 | bare Uppercase no binder: **EXIT=0** |
| 5 | `:i64`: **EXIT=1**, stderr is `BareLegacyPrimitive`, not `UnknownNamedType` |
| 6 | declared `:usr::Point`: **EXIT=0** |
| 7 | `Option` + `i64`: **EXIT=0** |
| 8 | `cargo nextest run --release -E 'test(p1_annotation)'`: **7 passed, 0 skipped** |
| 9 | walk shared; `fn walk_` 1 → 2 (`walk_type_expr` is the recursion) |
| 10 | census: **845 files, 0 refusing, 0 distinct names** |

## STOP rows

| STOP | result |
|---|---|
| STOP-1 TypeEnv into `parse_type_expr` | **held.** Not touched. |
| STOP-2 a green control went red | **held.** Rows 3/4/5/6/7 EXIT as predicted. |
| STOP-3 census > ~40 names | **held.** 0. |
| STOP-4 missing/contradicting fixture | **held.** All seven fixtures existed. |

## Targeted checks

```
cargo nextest run --release -E 'test(p1_annotation)'   7 passed, 0 skipped
./target/release/wat --check …__phantom_param_and_return.wat     EXIT=1  :usr::TotallyMadeUp
./target/release/wat --check …__phantom_record_field.wat         EXIT=1  :usr::AlsoMadeUp
./target/release/wat --check …__generic_type_param.wat           EXIT=0
./target/release/wat --check …__phantom_bare_uppercase_is_a_var.wat  EXIT=0
./target/release/wat --check …__bare_legacy_primitive.wat        EXIT=1  BareLegacyPrimitive
./target/release/wat --check …__control_declared_type.wat        EXIT=0
./target/release/wat --check …__builtin_and_generic_instantiation.wat  EXIT=0
```

Floor **orchestrator**. Clippy **orchestrator**.

## Working tree

```
src/declare/typevar.rs     walk_type_expr + first_unknown_named_type
src/check.rs               validate_named_type_annotations
src/freeze/env.rs          call after register_defines
src/types/error.rs         TypeErrorKind::UnknownNamedType
tests/types/probe_arc296_p1_annotation_names_a_type.rs   un-ignore
docs/.../CENSUS-STONE-P1-unknown-named-types.md
```

Do not commit unless a later brief says to.
