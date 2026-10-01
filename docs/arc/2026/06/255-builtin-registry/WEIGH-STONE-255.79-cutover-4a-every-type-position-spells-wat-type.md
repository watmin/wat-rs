# WEIGH — STONE 255.79: cutover 4a — every `.wat` type position spells `wat.type/` — ACCEPTED (STOP-1 open)

**Executor: a Sonnet subagent, commit `817e2003f`.** Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator at `817e2003f`

| row | result |
|---|---|
| release floor | `.floor/2026-10-01T20-18-57Z`: **6349 passed / 24 skipped**, exit 0 |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| the codemods were not their own input | the pristine copy the agent ran (`0374d9c78:wat-scripts/fixes/types-to-wat-type.wat`) is byte-identical to the draw's (`4b3f3194e`) |

## What landed

- **194 `.wat` files converted** by the recorded `types-to-wat-type.wat`: 122 recorded codemods (run by the pre-edit copy
  over `/tmp` mirrors), 72 elsewhere. Idempotent; census `--diff` no STOP-8; delta NEW 0 / RECOVERY 0.
- **Rule F** (`:nature` values) added with its own replay case: 79 `:nature :wat::core::Struct|Record` sites, stone 2's
  named residue.
- **A door bug the conversion exposed:** `Nature::from_root_keyword` (`src/types.rs`) normalized with `canonical_identity`,
  which does not fold `wat.type/`, so `:nature wat.type/Struct` was refused (the loader gate caught two scratch files).
  Swapped to `type_denotation`. ⚠ **For 4b:** that is the mapping 4b deletes; the nature-root match must then accept the
  new canonical key.
- Residue: 1,627 code-position tokens remain, classified (706 comment, 340 string, 308 head or slash-verb, 273 data);
  none a type position.

## STOP-1 (open, for the builder)

Wat programs embedded in Rust string literals: **1,093 type-position sites in 84 files** (`src/runtime.rs` 139,
`src/macros/tests.rs` 78, `src/check.rs` 61, …), over the brief's 600 bound. Not converted. 4b cannot refuse
`:wat::core::X` in type positions until these move.

## Verdict

Accepted and pushed; STOP-1 goes to the builder.
