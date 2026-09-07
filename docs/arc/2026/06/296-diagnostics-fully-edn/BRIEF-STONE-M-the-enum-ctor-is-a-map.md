# BRIEF — STONE M: the enum ctor is a map

> Read `DESIGN-STONE-M-the-enum-ctor-is-a-map.md` first. **The probe is committed and RED**:
> `tests/types/probe_arc296_enum_map_ctor.rs`, 5 rows + 5 co-located `.wat` fixtures.

## THE WORK, IN ONE PARAGRAPH

A variant is constructed by a map that NAMES its declared fields — `(:probe::Box::Full {:payload 7})`,
`(:probe::Box::Empty {})` — and the positional form `(:probe::Box::Full 7)` is REFUSED. `defrecord`
already works this way; enum variants are the last positional constructor in the language. Field
names come from the declaration in declaration order, which `:wat::runtime::type-of` already answers.
**The map names fields; it is never the payload** — that is the whole stone, and it is invisible
unless the construction sits in a TYPED slot.

## READ IN ORDER — the rooms

```
src/declare/register.rs:1270   header of register_enum_methods — read BEFORE the fn
src/declare/register.rs:1296   register_enum_methods — mints the ctors, positionally, today.
                               Unit variants -> sym.register_unit_variant. Payload variants ->
                               a fn whose return type is parametric_decl_type(:833).
src/declare/register.rs:833    parametric_decl_type — ★ the ctor returns the ENUM type, never a
                               variant type. Do not try to change that here (see OUT OF SCOPE).
src/check.rs:13726 · :13833    the kwargs-construction lowering + the note stating where it
                               deliberately does NOT mirror its sibling. Read both before :13951.
src/check.rs:13951             infer_kwargs_construct_check — THE PRECEDENT to mirror. Same
                               mechanism, different arg shape: a MAP, not flat kwargs.
src/check.rs:4690              the `:wat::core::kwargs-construct` dispatch arm
src/match_arm.rs:159           builtin_variant — the four hardcoded Option/Result exceptions.
                               Read it to see why Option behaves unlike a user enum. NOT yours
                               to delete.
wat-scripts/fixes/positional-to-kwargs.wat    the recorded arc-294 codemod for the RECORD half of
                               this same move — the shape a later corpus stone copies. Not run here.
```

## THE ACCEPTANCE ROWS ARE THE PROBE

`cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'` — un-ignore the four and make
them green without touching row 1:

```
the_control_program_checks_clean                    GREEN now, must STAY green
a_payload_variant_is_built_from_a_map_naming_its_field
a_unit_variant_is_built_from_an_empty_map
option_is_an_ordinary_enum_and_takes_the_same_ctor
the_retired_positional_ctor_is_refused              red in the OPPOSITE direction
```

Every bar is `check("control")`, run in the same test. Do not replace one with a literal.

## STOP TRIGGERS

- **STOP-1 — a fixture is moved into an UNTYPED slot to make a row pass.** In an untyped position
  the map form already "works" by being swallowed as the payload (`Option<HashMap<keyword,i64>>`).
  Every fixture is typed on purpose; changing that turns the probe green and proves nothing.
- **STOP-2 — Option/Result get a special case.** They are ordinary parametric enums declared in
  `wat/core.wat:2125`. If the general path cannot serve them, that is a finding about the general
  path, not a licence for a fifth exception.
- **STOP-3 — the positional form is left accepted "for compatibility".** Row 5 is the row that
  proves the migration finished. Two conventions at the end is a migration that never happened.
- **STOP-4 — field names are hand-listed anywhere.** They come from the declaration via `type-of` /
  the enum's own `TypeDef`. A hand-list is the defect 296 J and K exist to have deleted.
- **STOP-5 — the corpus is migrated.** NOT this stone. 2,209 sites move later, by wat-fix, once
  the form they are moving ONTO exists.
- **STOP-6 — `parametric_decl_type` is changed to return a variant type.** That is the
  variant-as-a-type question, a separate ruled-open stone. Construction names its variant
  explicitly, so this stone is sound without it.
