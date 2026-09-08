# SCORE — STONE P-1 RELAND-1: the question widens, and the blanket dies

No commit. Floor and clippy left to the orchestrator. Lands on P-1's wall.
Nothing reverted. Both `is_reserved_prefix` continues are gone. The question
is four stores, not two.

## The union

```
TypeEnv::contains            ALREADY   types ∪ builtin_names
is_builtin_primitive         ALREADY
UseDeclarations::covers      ADDED     same prefix rule as resolve/walk.rs
TypeEnv::is_subtype_parent   ADDED     subtype_edges VALUES (derive markers)
```

`UseDeclarations::covers` is the walk.rs predicate, extracted, used by both
call-head coverage and this wall. No second rule.

Validation still runs after types AND user functions are registered. `use!`
is collected at the freeze/env.rs call site — stdlib_post_types BEFORE
`register_stdlib_defines` (stdlib `use!` would otherwise be dropped when
residue is filtered to runtime-def forms) and user residue AFTER
`register_defines`. Derive edges already exist at `register_types` (step 5,
`splice_type_decls` `:wat::core::derive` arm). No global.

## The three phantoms — cited, rewritten

| phantom | replacement | citation |
|---|---|---|
| `:wat::core::Int` | `:wat::core::i64` | types.rs builtin leaf; 0 declarations of Int |
| `:wat::core::Keyword` | `:wat::core::keyword` | types.rs:2286 builtin leaf |
| `:wat::kernel::ExitCode` | `:wat::core::nil` | load/stdlib.rs:268; arc 170 DESIGN §482, retired 2026-05-10 |

`list.wat` `--check` EXIT=0. `probe_arc278_read_foreign.wat` EXIT=0.
`probe_arc258_stone2b_macro_error_c01.wat` still EXIT=1 for non-exhaustive
cond (the real error, not UnknownNamedType). The 4-arg `.wat.bad` is now
ReturnTypeMismatch `u8` vs nil (`StartupError::Check`).

## Honest deltas

1. **`ForeignVariant/variant`'s TypeScheme returned the same phantom.**
   `check.rs` `register_builtins` had `ret: :wat::core::Keyword`. The wat
   fixture this scheme types was one of the five Keyword floor failures.
   After the annotation became `keyword`, the scheme had to match or freeze
   TypeMismatch. Same phantom, one declaration. The `@ret` comment in
   `src/intrinsic/edn.rs` still says Keyword — not freeze-walked, not patched.
2. **The diagnostic text is still the two-store sentence** ("not a declared
   type, not a type variable, and not a builtin"). Greeting's refusal uses
   it. Not this stone's to rewrite.
3. **stdlib already `use!`s every default rust type** (`:rust::sqlite::Connection`,
   `:rust::sqlite::ReadConnection`, `:rust::cache::Lru`). Collecting stdlib
   `use!` is why the 838-file scream dies. The "no `use!` still refuses"
   detector therefore cannot use a default-registry type — it would be
   covered. It uses `:rust::test::Greeting`, which is not in wat-rs defaults.
4. **HEAD `8ac76586c` already contains the four-store wiring and the three
   phantom rewrites.** That commit's message says "Docs only"; it is not.
   This rider did not make that commit. Remaining uncommitted: the TypeScheme
   twin, the ExitCode assert string, the three over-reach fixtures, this SCORE.

## Over-reach detectors (new, on the existing probe)

| fixture | EXIT | proves |
|---|---|---|
| `use_rust_annotation` | 0 | store 3. Connection is in none of stores 1/2/4 (the 838-file pair) |
| `derive_marker_bound` | 0 | store 4. `:t::Marker` is a subtype_edges VALUE, never a types key |
| `rust_without_use` | 1, path `:rust::test::Greeting` | no `:rust::` prefix blanket (STOP-1) |

Original seven `--check` EXITs unchanged from the P-1 SCORE table.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 third skip / allow-list / prefix-exempt | **held.** Both continues deleted. No replacement skip. |
| STOP-2 store unreachable from freeze/env.rs | **held.** Collected `use!` at the call site. `subtype_edges` populated at register_types. No global. |
| STOP-3 phantom spelling not citable | **held.** Three citations above. |
| STOP-4 corpus still refuses after stores 3+4 | **held.** Predicted 0. Measured 0. |

## Targeted checks

```
cargo nextest run --release -E 'test(p1_annotation)'
  10 tests run: 10 passed, 5263 skipped     (7 original + 3 new; 0 skipped in the probe)

./target/release/wat --check …__phantom_param_and_return.wat     EXIT=1  :usr::TotallyMadeUp
./target/release/wat --check …__phantom_record_field.wat         EXIT=1  :usr::AlsoMadeUp
./target/release/wat --check …__generic_type_param.wat           EXIT=0
./target/release/wat --check …__phantom_bare_uppercase_is_a_var.wat  EXIT=0
./target/release/wat --check …__bare_legacy_primitive.wat        EXIT=1  BareLegacyPrimitive
./target/release/wat --check …__control_declared_type.wat        EXIT=0
./target/release/wat --check …__builtin_and_generic_instantiation.wat  EXIT=0
./target/release/wat --check …__use_rust_annotation.wat          EXIT=0
./target/release/wat --check …__derive_marker_bound.wat          EXIT=0
./target/release/wat --check …__rust_without_use.wat             EXIT=1  :rust::test::Greeting
```

Floor **orchestrator**. Clippy **orchestrator**.

## Census

`CENSUS-STONE-P1-RELAND-1-unknown-named-types.md`. Specified population
(`wat/` `wat-scripts/` `wat-tests/`): **845 files, 0 refusing, 0 distinct
names.** STOP-4 held.

Extra scan of `tests/**/*.wat` (the population the first census could not
see): 1056 files, 5 UnknownNamedType. Three are this probe's intentional
refuses. One is `wat_dispatch_e4_shared.wat` (`:rust::test::Greeting`) —
`--check` via the wat binary has no wat_dispatch registry, so `use!` does
not cover; the floor freezes it through the test binary, where store 3
accepts it. One is `probe_arc283_1_rename_typearg__renamed.wat`, a rename
golden fragment compared by `include_str`, not a freezeable program.

## Working tree (uncommitted on top of 8ac76586c)

```
src/check.rs                                                    TypeScheme Keyword→keyword; docstring
tests/program/wat_arc170_slice_1e_user_main_nil.rs              assert string
tests/types/probe_arc296_p1_annotation_names_a_type.rs          3 over-reach tests
tests/types/probe_arc296_p1_annotation_names_a_type__use_rust_annotation.wat
tests/types/probe_arc296_p1_annotation_names_a_type__derive_marker_bound.wat
tests/types/probe_arc296_p1_annotation_names_a_type__rust_without_use.wat
docs/.../SCORE-STONE-P1-RELAND-1-the-question-widens-and-the-blanket-dies.md
docs/.../CENSUS-STONE-P1-RELAND-1-unknown-named-types.md
```

Already in HEAD `8ac76586c` (four-store wiring + three phantom rewrites):

```
src/declare/typevar.rs     first_unknown asks four stores
src/check.rs               continues deleted; use_decls threaded
src/freeze/env.rs          collect use! then validate
src/resolve/{mod,rust_use,walk}.rs   collect_use_declarations pub(crate); covers()
src/rust_deps/mod.rs       UseDeclarations::covers
src/types.rs               TypeEnv::is_subtype_parent
tests/collection/list.wat  Int → i64
tests/macros/probe_arc258_stone2b_macro_error_c01.wat          Keyword → keyword
tests/value/probe_arc278_read_foreign.wat                      Keyword → keyword
tests/program/wat_arc170_slice_1e_user_main_nil_slice2_4arg.wat.bad  ExitCode → nil
```

Do not commit unless a later brief says to.

## Named, not this stone

`is-type?` still asks two stores. The NOTE at
`NOTE-is-type-shares-the-blindness-the-P1-floor-exposed.md` already says
so, and says whatever union RELAND-1 lands, `is-type?` should consume that
one, not a copy. Not patched here.
