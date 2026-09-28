# WEIGH — STONE 255.70: constructor brackets through the type door; `List` takes `:-` — ACCEPTED

**Executor: a Sonnet subagent, commits `c389233af` … `627342cf8`.** Weighed by the orchestrator on 2026-09-28.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor at `627342cf8` | **6225 passed / 24 skipped**, exit 0 |
| `parse_bracket_type_keyword` | gone from `src/check.rs` (retired, its three callers on `parse_param_spec_slot` → `parse_type_node`) |
| the one red | `.floor/2026-09-28T22-03-50Z`, 1 failed, `unused_span_justified`: the stone's own two new `_…span` params in `src/intrinsic/list.rs`. Cured with justified runes (read: `list.rs:55`), and a **new** floor ran green. **The first stone under the revised red rule, used as intended.** |
| agents' gates | census `no STOP-8`; delta on the 48 files (pre-conversion restored) `NEW 0 RECOVERY 0`; idempotent; clippy rc 0 |

## What landed

- A constructor's type bracket is read by the one type door, so compound element types check (`(wat.type/PersistentVector
  :- [(wat.type/Tuple :- [A B])])`). No other keyword-only bracket path exists (the agent checked each candidate the brief
  named).
- `List` takes `:-` in the checker and at run time. `:wat::core::List` moved from the ALGEBRA registry (whose auto-generated
  AST door evaluated the bracket as a value) to BINDING, with `apply`'s value door kept byte for byte. **The bracket is
  optional for now**: a bracket-less `List` still checks until the wall.
- **109 / 109** sites converted by the unchanged codemod, fed a fresh type table.

## What is left for the wall

**177 untyped constructor calls remain** (List 23, PersistentMap 27, PersistentVector 44, Tuple 73, Vector 10): 255.68's
unresolved (33) and not-checked (141) classes, one new bracket-less regression fixture, and one attributed to corpus
growth. These are the heretics the wall will name.
