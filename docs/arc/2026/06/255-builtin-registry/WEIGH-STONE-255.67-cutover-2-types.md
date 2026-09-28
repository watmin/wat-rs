# WEIGH — STONE 255.67: cutover 2/7, the corpus spells its hard primitives `wat.type/` — ACCEPTED (with a recorded residue)

**Executors: grok could not (credits out); four Sonnet subagents, across the brief and three amendments.** Commits
`029ae9f2c` … `ed5d7749b` (15 local commits, pushed with this WEIGH). Weighed by the orchestrator on 2026-09-28.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor at `ed5d7749b` | **6213 passed / 24 skipped**, exit 0 |
| K1's own floor (the agents') | `.floor/2026-09-28T05-45-45Z` 6213/6213; the two red floors before it (`05-09-49Z` 103, `06-00-51Z` 57) were kept and classed, never re-run to green |
| `wat.type/` tokens in `.wat` | 22,922 |
| clippy / census / delta / idempotence | the agents': clippy rc 0; census `no STOP-8`; delta NEW 2 / RECOVERY 0 (same two files); a re-run of the codemod changes 0 files |

## What landed

- **The recorded codemod** `wat-scripts/fixes/types-to-wat-type.wat` (a single left-to-right walk; rules A–E), over the
  stdlib and 2,513 corpus files.
- **K1** (`canonical_type_key`): every type spelling becomes one key at the door where it enters. This retired the
  "two spellings compared" class, five instances found by the floor, all the recurring class CLAUDE.md names.
- **Five keyword-only type recognizers** routed through the type door (one corpus-live: an `extend-type` method whose
  return type was a symbol had `->` and the type read as its **body**). One silent misclassification in `from-holon`
  fixed. 32 position-only goldens recaptured (each listed). Rete's scan keyed by spelling fixed. Ledger 149 → 148.
- **X-G:** 37 goldens are `.wat.golden`; one restored; six dead fixtures deleted; the EDN-bridge control repointed at a
  real program (`tests/collection/probe_arc257_native_map_set.wat`).

## Process notes

- The first agent kept going past a red floor (STOP-3) instead of stopping. Its fixes were right, and I judged each on
  its own evidence. The later agents stopped as briefed each time.
- The golden census (255.58) missed a `read_to_string` consumer; the X-G agent found it by the floor, and its follow-up
  search covered every non-`include_str!` reader.

## Residue (measured by the orchestrator; code only, excluding `wat-scripts/fixes/`, comments and strings)

| class | count | goes to |
|---|---|---|
| bare constructor **calls** with no `:-` (`PersistentVector` 1,334, `Tuple` 230, `PersistentMap` 122, `List` 109, `Vector` 6) | ~1,800 | the open constructor question (255.65 §3): what an untyped constructor call becomes |
| calls of functions named like types (`keyword` 137, `u8` 57, `bool` 32, `Record` 26, `char` 17, `i64` 7) | ~275 | the same question: conversions/casts are function heads |
| `:nature :wat::core::Struct` | 79 | **a type position the rules missed**; a follow-up codemod rule |
| other positions, and `(Head :- …)` left | 256 + 21 | to classify |
