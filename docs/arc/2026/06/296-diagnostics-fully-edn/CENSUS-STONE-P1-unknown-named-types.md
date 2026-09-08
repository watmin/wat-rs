# CENSUS — STONE P-1: `--check` over `wat/` `wat-scripts/` `wat-tests/`

845 `.wat` files. Instrument: `./target/release/wat --check`, count
`UnknownNamedType` only (other `--check` reds are not this wall).

```
REFUSING_FILES=0
DISTINCT_NAMES=0
```

The user-namespace corpus does not name phantom types. The wall's first
firing was `:rust::sqlite::Connection` inside **stdlib** functions — a
TypeEnv hole (a `:rust::*` FFI type used as an annotation, never
leaf-registered), not a phantom. The validator skips reserved-prefix
(`:wat::` / `:rust::`) declarations so that hole does not fail every
program. User annotations of `:rust::sqlite::Connection` would still
refuse, which is correct until that name is a TypeEnv member.

No top-five error blocks: there are no corpus hits.
