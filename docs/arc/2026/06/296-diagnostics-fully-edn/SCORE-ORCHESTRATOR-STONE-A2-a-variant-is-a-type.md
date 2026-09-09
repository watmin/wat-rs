# SCORE (independent re-run) — STONE A-2: a variant is a type

Independent re-run of the rider's SCORE. Floor not run (ingest did not
ask for `scripts/floor.sh`). Clippy not run.

```
cargo build --release     Finished, 5 pre-existing dead_code warnings, no new ones
git diff --stat src/record/construct.rs    empty
```

## Rows 1–14, re-run

| # | command | rider | this re-run |
|---|---|---|---|
| 1 | `--check …__process_full_box.wat` | EXIT 0 | **EXIT 0** |
| 2 | `--check …__ctor_carries_the_variant.wat` | EXIT 0 | **EXIT 0** |
| 3 | `--check …__variant_widens_to_enum.wat` | EXIT 0 | **EXIT 0** |
| 4 | `--check …__match_still_works.wat` | EXIT 0 | **EXIT 0** |
| 5 | `--check …__nonexistent_variant.wat` | EXIT 1, `:usr::Box::Nope` | **EXIT 1**, path `:usr::Box::Nope` |
| 6 | `--check …__enum_does_not_narrow.wat` | EXIT 1, TypeMismatch, no UnknownNamedType | **EXIT 1**, `expects (:usr::Box::Full :- [:wat::core::i64]); got (:usr::Box :- [:wat::core::i64])` — no `UnknownNamedType` |
| 7 | `-E 'test(a2_a_variant)'` | 6/0 | **6 passed, 0 skipped** |
| 8 | `-E 'test(p1_annotation)'` | 10/0 | **10 passed, 0 skipped** |
| 9 | `-E 'test(p1b_a_parametric)'` | 4/0 | **4 passed, 0 skipped** |
| 10 | `-E 'test(p2prereq)'` | 4/0 | **4 passed, 0 skipped** |
| 11 | `-E 'test(p3_one_question)'` | 5/0 | **5 passed, 0 skipped** |
| 12 | `-E 'test(a1_one_rule)'` | 4/0 | **4 passed, 0 skipped** |
| 13 | `git diff --stat src/record/construct.rs` | empty | **empty** (not in the working-tree diff) |
| 14 | TypeDef choice | `TypeDef::Enum` singleton | **confirmed in source**: `register_variant_types` mints `TypeDef::Enum { variants: vec![that one variant] }` |

**14/14 match the rider on this re-run.**

## The user-only scope — confirmed in source, not re-measured at 1228

`register_variant_types` filters `!is_reserved_prefix(name)`. The rider's
1228-error first measurement was not reproduced here (that would mean
reverting the scope). The disclosure is load-bearing: stdlib enum
constructions still erase. Every EXPECTATIONS fixture is `:usr::Box`.

Floor and STOP-5 corpus scan remain the orchestrator's if a later turn
asks for them.

## Verbatim, rows 5 and 6

```
#wat.type/UnknownNamedType {:message "annotation names unknown type :usr::Box::Nope — not a declared type, not a type variable, and not a builtin" … :path ":usr::Box::Nope"}
```

```
#wat.check/TypeMismatch {:message ":user::takes-full: parameter #1 expects (:usr::Box::Full :- [:wat::core::i64]); got (:usr::Box :- [:wat::core::i64])" …}
```
