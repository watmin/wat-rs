# BRIEF 2a1b — one membership door: `type-of` answers every name `is-type?` admits

**RULED 2026-09-13: C** (four questions in the main chat). Finding 14 is the measured ground; finding
11 named the defect. 2a2 needs it: once the codemods leave `eval-with-defs!`, their stdlib questions
reach `type-of` directly, and `eval-with-defs!` was the only thing catching its raise.

## Why — three verbs, three copies of "is this name a type?", three answers

Measured on `5edca1211` (`bootstrap/era/probe-R/defect-*.wat`, `subtype-marker.wat`):
- `:wat::core::Vector`: `is-type?` → true; `type-of` raises `unknown type ':wat::core::Vector'`.
- `(:wat::core::derive :t::A :t::Marker)`: `(is-type? :t::Marker)` → true; `(:wat::core::subtype? :t::A
  :t::Marker)` raises `unknown type name ':t::Marker' is not registered in the TypeEnv and is not a
  built-in primitive` — the one question `derive` exists to set up.
- A literal `(:wat::runtime::type-of :wat::core::i64)` is refused at CHECK (Doctrine 1: `src/check.rs:3674`
  infers the arg as a value); `(:wat::runtime::is-type? :wat::core::i64)` passes check
  (`src/check.rs:2839` reads it as a type position).

`is-type?` answers from `TypeEnv::is_known_type` (`src/types.rs:631`): `contains` (the `types` map and
the `builtin_names` set) ∪ `crate::runtime::is_builtin_primitive` (`src/runtime.rs:9995`) ∪
`is_subtype_parent` (`src/types.rs:956`). `type-of` answers from `get` only
(`src/reflect/verbs.rs`, `eval_type_of`). `subtype?` answers from `get` ∪ `is_builtin_primitive`
(`src/runtime.rs:10143`, `:10147`).

## The shape

1. **One classifier on `TypeEnv`**, beside `is_known_type`. For a keyword it returns exactly one of:
   - **Declared**(`&TypeDef`) — `get` is `Some`;
   - **Builtin** — in `builtin_names` or `is_builtin_primitive` (the two stores `is_known_type` unions);
   - **Marker**(children) — `is_subtype_parent`; children are the `subtype_edges` keys that list it,
     sorted;
   - **Unknown**.

   Checked in that order (a name that is a `TypeDef` answers Declared even if a builtin store also
   holds it — `:wat::core::Option`/`Result` are both `defenum`s and `BARE_CONTAINER_HEADS` entries). It
   carries `is_known_type`'s `:wat::type::` → `:wat::core::` canonicalization. **`is_known_type` becomes
   "not Unknown" of it** — one door. Membership only: builtins keep `get` = `None`; no `TypeDef` is
   fabricated (the `builtin_names` field doc, `src/types.rs:549-561`).
2. **`type-of` answers from the classifier.** Declared → `type_info_value` (unchanged). Builtin → kind
   `:Builtin`, body `:Builtin`. Marker → kind `:Marker`, body `:Marker` with its children. Unknown → the
   same raise as today. The Builtin and Marker rows are built in ONE function beside `type_info_value`.
3. **`wat/runtime-typeinfo.wat`**: `TypeKind` gains `:Builtin` and `:Marker`; `TypeBody` gains
   `:Builtin []` and `:Marker [children <- (:wat::core::Vector :- [:wat::core::keyword])]`. The row's doc
   says: a `:Builtin` row's `type-params` is empty because a builtin's structure, parameters included,
   is not declared anywhere (`builtin_names` holds names only). Rust registers these enums from the
   wat source (`wat_enum_register_from!`, `src/types.rs:2399-2407`); `type_kind_value` /
   `type_body_value` (`src/reflect/verbs.rs:1356`, `:1443`) build values by variant name.
4. **The checker reads `type-of`'s literal keyword arg as a type position**, as it does for `is-type?`
   (`src/check.rs:2839`): a literal `:wat::core::i64` passes check. A non-literal arg is inferred as
   today (the codemods pass computed keywords).
5. **`subtype?`'s known-check asks the classifier** (not Unknown), so a marker and every
   `builtin_names` entry are known. `(:wat::core::subtype? :t::A :t::Marker)` → true.

`:wat::runtime::declared-types` is unchanged: it reports the `TypeEnv` keys a program added.

## Pins — tracked ground only

- `tests/reflection/probe_arc296_type_of_six_kinds.wat`: its exhaustive `TypeKind` match (`:32-37`)
  gains the two arms. `:user::main` gains rows, and the runner's exact-stdout assertion
  (`tests/reflection/probe_arc296_reflection_answers_for_every_type_kind.rs:82`) gains their lines:
  - `(:wat::runtime::type-of :wat::core::Vector)` → `Builtin`;
  - `(:wat::runtime::type-of :wat::core::i64)`, a literal → `Builtin` (it passes check);
  - `(:wat::runtime::type-of (:wat::keyword::from-string "wat::core::HashMap"))` → `Builtin`;
  - with `(:wat::core::derive :probe::Rec :probe::Marker)` in the fixture: `type-of :probe::Marker` →
    `Marker`, children `[:probe::Rec]`.
- `(:wat::core::subtype? :probe::Rec :probe::Marker)` → true, in a tracked probe (name it).
- An unknown name still raises the same error (`unknown type ':nope::Nothing'`), pinned.
- **The agreement wall:** a test that walks EVERY name `is-type?` admits in the stdlib snapshot —
  every `types` key, every `builtin_names` entry, every `is_builtin_primitive` name, every subtype
  parent — enumerated from the stores themselves, never from a list in the test, and asserts `type-of`
  answers each without raising, with the kind the classifier gives. It must go RED once (e.g. make the
  Builtin arm raise) before it counts; paste that red.

## Measure and report (no change)

The set difference between `is_builtin_primitive`'s 37 names and `builtin_names`
(`register_builtin_types`, `src/types.rs:2544-2615`), both directions, as a table in the SCORE. The
classifier unions them, so `is-type?` and `type-of` agree whatever the difference is; reconciling the
two stores is its own stone.

## STOP triggers

- **STOP-1:** a name `is-type?` admits that none of the four answers fits. Report it verbatim.
- **STOP-2:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green (floor + clippy 0). **Do not push.** Yield with `SCORE-2a1b.md`: one row per shape
item (1-5) and per pin, plus the set-difference table.
