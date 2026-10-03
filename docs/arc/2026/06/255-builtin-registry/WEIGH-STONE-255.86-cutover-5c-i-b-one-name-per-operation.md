# WEIGH — STONE 255.86: cutover 5c-i (b) — one name per operation (G1), `/` means a type member (R-a) — ACCEPTED

**Executor: grok via pulsare, through four amendments.** Cutover `4f8f551ea`, cure `54e2d948d`, tests `9162b9f40`/
`d3dc12bbc`, data pins and one-process gate `c93b83701`/`b819c0379`, SCORE `fd5321bd4`. Weighed by the orchestrator on
2026-10-03.

## Re-run by the orchestrator at `fd5321bd4`

| row | result |
|---|---|
| release floor | `.floor/2026-10-03T05-24-19Z`: **6397 passed / 24 skipped**, exit 0 |
| the new tests | `one_name_replacements_return_the_pinned_values` 2.40 s; `formattable_and_reject_queried_in_the_type_registry` 1.58 s; `stone_rows_are_refused_naming_their_replacement` **0.85 s** (was 16 s as 43 serial processes) |
| clippy | exit 0 |
| no timeout override | `.config/nextest.toml`'s 255.86 override removed (17 lines) |
| pins are data | the fixture holds 62 `:wat::test::assert-eq` on values, 0 `str` renderings |

## What landed

- **G1:** the 67-pair table `wat-scripts/fixes/one-name-per-operation.edn` and its recorded codemod moved the per-type
  collection calls to the polymorphic core verbs; scalars to their type namespaces; new homes `wat.bytes/` and
  `wat.record/`; the `of` functions into the typed constructors; 43 retirement rows refuse the old names naming the
  replacement. Two lessons from the mapping, each ruled: `vector/concat` meant **`into`** (a designed PV←Vector clause),
  and a missing container arm on a core verb is **added** (PersistentMap on `dissoc`/`keys`/`values`;
  `(PV, PV)` on `into`), not a reason to keep the per-type name.
- **R-a:** the non-type parents respelled `/` → `::`, including `:myapp::Formattable` and `:wat-tests::holon::Reject` once
  the registry said neither is a type; the member-join wall now decides "type" with `TypeEnv::contains`.
- Census: no rc flips. Embedded dry run: 0 changed, 0 refused.

## Corrections the weigh made (on the record)

- My STOP-1 did not separate a coverage gap from a semantic conflict, and cost a round (memory: STOP triggers).
- The first tests pinned **rendered strings**; now values, as data.
- A retirement gate was made green by **raising a timeout to 120 s**; removed, and the gate restructured to one process.

## Noted, not blocking

- `infer_assoc`'s value parameter now uses `assignable` (permissive, matching what the per-type verbs accepted; no flips).
- `NAMING_RULE_EXCEPTIONS` 19 → 25 (six rete rows keep `rete_name`, pointing `core_name` at the core verb).
- The reader's lexer panics on `∅` and `≠` (a char-boundary slice); the driver catches it. A real lexer defect, open.

## Verdict

Accepted and pushed. 5c-i is complete. Next: 5c-ii, the stdlib conversion.
