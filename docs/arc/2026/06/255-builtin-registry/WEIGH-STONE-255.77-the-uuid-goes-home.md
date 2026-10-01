# WEIGH — STONE 255.77: cutover stone 3, the UUID type goes home — ACCEPTED

**Executor: a Sonnet subagent, commit `628260fd9`.** Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator

- **At `628260fd9`:** `.floor/2026-10-01T09-41-45Z` was **red**, 6338/6339:
  `rete::kernel::tests::harvest_cost::harvest_wrap_split` (`harvest_cost.rs:337`, a wall-clock apportionment). The same
  commit passed it on the executor's floor twelve minutes earlier (`.floor/2026-10-01T09-29-58Z`). Not re-run. The builder
  ruled T1 (no floor verdict reads a clock), done as 255.78, and 255.77 was held until that landed.
- **At `f12c10995` (255.77 + 255.78):** `.floor/2026-10-01T18-41-07Z`: **6349 passed / 24 skipped**, exit 0, with
  `harvest_wrap_split` passing on its new count witness. Clippy exit 0.

## What landed (read in the diff)

- The type's key is `:wat::uuid::UUID`, spelled `wat.uuid/UUID`. The recorded codemod `wat-scripts/fixes/uuid-type-goes-home.wat`
  converted 10 sites (idempotent, replay-fixtured). `Value::wat__core__Uuid` → `Value::wat__uuid__Uuid`; Rust literals renamed.
- `:wat::core::Uuid` is refused by name (`src/check.rs:1010-1025`, the `:wat::core::Char` arm's shape) with a retirement
  row naming `wat.uuid/UUID`. **The keyword-heresy ledger grew 148 → 149** for that arm: disclosed, same shape as its
  counted sibling, a refusal and not dual support.
- `framing-floor-of` compares field types through `:wat::core::type-equal?` instead of `ast-name` strings, all four
  branches; its frame floors (38, 42, 55, 24) were pinned before and are identical after.

## Finding for stone 6 (printers)

`type_expr_to_clojure_form` (`src/edn/render.rs`) renders any `wat::core::X` as `wat.type/X` without asking the closed set
of 24, so `Option`, `Result`, `Span` would print under `wat.type/`, against the cutover ruling. That is why
`field-types-of` printed `wat.type/Uuid` before this stone.

## Verdict

Accepted and pushed with 255.78.
