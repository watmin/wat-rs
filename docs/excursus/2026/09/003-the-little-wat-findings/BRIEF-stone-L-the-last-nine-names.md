# BRIEF — STONE L: the last nine Value variants that break the naming rule

**Drawn 2026-09-24.** Builder: *"get the names uniform"* (stone J). Stone K's whole-enum census
found these; the orchestrator spot-checked the `wat__std__` and case rows. Rule: a `Value` variant is
its wat type path with `::` as `__` (and `-` as `_`, the only spelling Rust allows).

## The renames (type paths from the `type_name` table in `src/value/value.rs`)

| today | type path | becomes | `Value::` refs |
|---|---|---|---|
| `wat__std__HashMap` | `:wat::core::HashMap` | `wat__core__HashMap` | 65 |
| `wat__std__HashSet` | `:wat::core::HashSet` | `wat__core__HashSet` | 44 |
| `wat__core__Char` | `:wat::core::char` | `wat__core__char` | 24 |
| `wat__core__Rational` | `:wat::core::rational` | `wat__core__rational` | 50 |
| `wat__core__BigInt` | `:wat::core::bigint` | `wat__core__bigint` | 56 |
| `Instant` | `:wat::time::Instant` | `wat__time__Instant` | 35 |
| `Duration` | `:wat::time::Duration` | `wat__time__Duration` | 27 |
| `ForeignRecord` | `:wat::edn::ForeignRecord` | `wat__edn__ForeignRecord` | 23 |
| `ForeignVariant` | `:wat::edn::ForeignVariant` | `wat__edn__ForeignVariant` | 17 |

~341 references. `wat__core__extend_def` (`:wat::core::extend-def`) already complies — do not touch.
Core primitives (`i64`, `String`, `Vec`, `Option`, `Tuple`, `Aggregate`, `RustOpaque`, …) — out.

## ⛔ Collision map — the dangerous words

- `Duration` / `Instant`: `std::time::Duration` appears 13× in `src/`, and `chrono` types may too.
  Rename ONLY `Value::Duration` / `Value::Instant` and the declarations.
- `HashMap` / `HashSet`: `std::collections::HashMap` is everywhere; only `Value::wat__std__HashMap`
  moves.
- `wat_edn::Value` is a DIFFERENT enum (stone J's grep counted it by mistake) — scope every grep.
- Lowercase variant names (`wat__core__char`) need the same `#[allow(non_camel_case_types)]` the enum
  already carries for `wat__core__fn` / `wat__core__keyword` — confirm it applies.
- EDN tag strings and type-path strings do NOT change. Messages/comments/Debug expectations that name
  a variant DO, as ruled (`RULING-a-childs-stdout-is-a-wire.md` §2).

## Prove it

Old names in code roots → 0 (scoped to the wat `Value`); the undo-the-rename-on-`+`-lines check that
stone K used, showing every changed line differs only by the token; no EDN/type-path string count
moved; floor 0 failed; clippy clean. The compiler is the gate — no mutation proof.

## STOP triggers

1. A reference you cannot classify as the wat `Value` variant vs a colliding name — report it.
2. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- Neither `cargo fmt` nor `rustfmt <file>` — touched files already fail `rustfmt --check` for
  unrelated reasons (stone K). Stage explicit paths; delete any throwaway script.
