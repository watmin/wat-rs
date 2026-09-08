# CENSUS — STONE P-1 RELAND-1: `--check` over `wat/` `wat-scripts/` `wat-tests/`

845 `.wat` files. Instrument: `./target/release/wat --check`, count
`UnknownNamedType` only (other `--check` reds are not this wall). Both
`is_reserved_prefix` continues removed; stores 3 and 4 added.

```
REFUSING_FILES=0
DISTINCT_NAMES=0
```

STOP-4 held. Predicted zero. Measured zero. The 838-file scream on
`:rust::sqlite::*` is gone: stdlib `use!` of `:rust::sqlite::Connection` /
`:rust::sqlite::ReadConnection` is collected before
`register_stdlib_defines` and asked via `UseDeclarations::covers`.

No top-five error blocks: there are no corpus hits.

## What this instrument can see

This is the same three-directory population as the first P-1 census.
That census was honest for what it could see and blind to rust-beside
fixtures (`call_beside_value`). The floor named three phantoms there;
they were rewritten against declarations, not found by this walk.

An extra scan of `tests/**/*.wat` (1056 files, not a STOP-4 population)
found 5 UnknownNamedType:

```
tests/types/probe_arc296_p1_annotation_names_a_type__phantom_param_and_return.wat   :usr::TotallyMadeUp     intentional
tests/types/probe_arc296_p1_annotation_names_a_type__phantom_record_field.wat       :usr::AlsoMadeUp         intentional
tests/types/probe_arc296_p1_annotation_names_a_type__rust_without_use.wat           :rust::test::Greeting    intentional
tests/kernel/wat_dispatch_e4_shared.wat                                            :rust::test::Greeting    wat binary has no wat_dispatch registry; the floor freezes this through the test binary
tests/types/probe_arc283_1_rename_typearg__renamed.wat                              :t::New                  include_str golden fragment, not a freezeable program
```
