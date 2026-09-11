# SCORE — type the polymorphic accessor

Struck. Floor not run. Clippy not run. Not committed.

```
cargo build --release                                          BUILD_EXIT=0
cargo nextest run --release -E 'binary_id(wat::types)'         627 passed, 5 skipped, NEXTEST_EXIT=0
```

(was 614 before this stone's 13 probes.)

## The block

`src/check.rs` unknown-scheme 1-arg fall-through. `acceptable` looked up the
TypeDef, confirmed named fields, then discarded the answer and returned
`fresh.fresh()`. Parametric **Aggregate** was absent from the match (A-2
RELAND-5 added only the singleton Enum).

**Changed.** `keyword_accessor_fields` reuses `instantiate_field_types`.
Known named-field receiver → the field's instantiated type, or a located
`MalformedForm` naming the missing field and the declared set. Fresh var
kept only for unresolved, HashMap (STOP-2), and the zero-field Record
umbrellas. `assignable` untouched.

## Cascade (not a fourth read path)

Typing `(:field self)` type-checks the **bodies** of the previous stone's
synthesized `:Enum.Variant/field` functions. First build could not start
the substrate (35 `ReturnTypeMismatch` on stdlib accessors,
`register.rs:1473`):

1. `parametric_decl_type` emits `Path("T")`; declared fields / `ret_type`
   store `Path(":T")`. Body inferred `T`, scheme declared `:T`.
2. Accessor schemes used the **parent's** full `type_params` as the
   receiver args; the singleton TypeDef stores only consumed params.
   `Result.Err` (`[E]`) zipped against `(:Result.Err :- [T E])` bound
   `E` to `T`.

Fixes, both required for the substrate to load:

- `colonize_type_var_paths` on `instantiate_field_types` output.
- Accessor mint uses the singleton's `type_params` for `type_params` and
  `parametric_decl_type` of the receiver.

Not STOP-4: this is the named-accessor **implementation** (already a listed
path), now checked because its body is the bare-keyword path this stone types.

## Grid (real exit codes)

| row | pre-fix | post-fix |
|---|---|---|
| RECORD mono `(:x c)` → i64 | EXIT 0 | **EXIT 0** |
| RECORD mono `(:x c)` → String | EXIT 0 (lie accepted) | **EXIT 1** ReturnTypeMismatch body i64 / sig String |
| RECORD param `(:x c)` → i64 | EXIT 1 UnknownCallee `:x` | **EXIT 0** |
| RECORD param `(:x c)` → String | EXIT 1 UnknownCallee (wrong reason) | **EXIT 1** ReturnTypeMismatch |
| VARIANT param `(:has d)` → i64 | EXIT 0 (fresh var) | **EXIT 0** (now i64) |
| VARIANT param `(:has d)` → String | EXIT 0 (lie accepted) | **EXIT 1** ReturnTypeMismatch |
| `(:nonexistent r)` known receiver | EXIT 0, eval UnknownField | **EXIT 1** MalformedForm, names `nonexistent` and declared `x` |
| HashMap `(:port m)` | EXIT 0 | **EXIT 0** |
| named RECORD/VARIANT accessor lie | EXIT 1 | **EXIT 1** |
| VARIANT `{:keys}` lie | EXIT 1 | **EXIT 1** |
| named VARIANT accessor truth | EXIT 0 | **EXIT 0** |
| runtime twin | — | **RUN_EXIT=0**, stdout `42` |

Probes: `tests/types/probe_arc251_type_the_polymorphic_accessor.rs` (13 tests).

`probe_3` in `probe_arc234_stone3c_keyword_accessor` lived in the shared
beside file; a check error there would have blocked probes 1/2/4/5/6.
Removed that defn; the test now `--check`s the new unknown-field fixture.

## STOP triggers — none fired

- **STOP-1** `assignable` / variance — not in the diff.
- **STOP-2** HashMap — still the placeholder; `hashmap_receiver` EXIT 0;
  existing probes 4/5 still in the 627.
- **STOP-3** floor / controls — floor not run; `wat::types` 627/5; every
  named-accessor / `{:keys}` / HashMap control holds.
- **STOP-4** fourth read path — none found. The cascade is the previous
  stone's named-accessor body, not a new path.

## Uncertain

- Record umbrellas (`:wat::core::Record`, `:wat::holon::Record`) still
  placeholder: they are Aggregates with zero fields, so a lookup would
  refuse every `(:x r)` on a generic Record parameter. Left as the old
  behaviour; a later stone can decide.
- `colonize_type_var_paths` is a spelling normalizer, not a unify change.
  Path("T") vs Path(":T") still fail unify if they meet elsewhere.
