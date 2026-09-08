# BRIEF — STONE N RELAND 5: re-sweep with the repaired tool

> ⛔ **THIS IS AN ORCHESTRATOR ERROR BEING CORRECTED, NOT A NEW DEFECT.**

## WHAT WENT WRONG — mine

The codemod was **repaired in RELAND 2** (`e77311bf9`: `rewrite-each` recurses with the stdlib base
instead of threading the per-file fmap, plus the decls retry chain). **It has not been re-applied at
scale since.** RELAND 3 and RELAND 4 closed *specific named failures* by hand, because that is how I
wrote those briefs — "close these" rather than "re-apply the tool, then handle what screams."

So the last two relands hand-edited instances of a class the repaired tool sweeps in one pass. That
is exactly what R21 exists to prevent.

**Proof it is the tool's reach and not a hard case** — a currently-failing file:

```
tests/types/enums_mixed_unit_tagged.wat
  :2   (:wat::core::defenum :my::Event :wat::enum::Pure  :Open [size <- :wat::core::f64]  :Hold)
  :11  (:my::Event::Open …                     <- still positional
```

A TOP-LEVEL declaration with an unambiguous field name. Nothing about it is hard. It was simply
never swept after the fix.

## THE WORK

1. **Re-run the repaired codemod over the FULL worklist** — every `.wat` under `wat/`, `wat-tests/`,
   `wat-scripts/`, `tests/`. Not the failing subset. The tool is different from when the corpus was
   last swept.
2. **Run it a second time** and confirm 0 further changes.
3. **Read every `[positional-ctor] UNRESOLVED` scream.** THAT is the hand-edit worklist, and only
   that. For each, say why the tool declined.
4. Re-measure and report the delta.

⛔ **DO NOT skip files that "look done."** The last sweep predates the fix; a file's current state
tells you nothing about whether the repaired tool agrees with it.

⚠ **PRESERVE THE CENSUS BAIT.** `wat-scripts/fixes/*.wat` and `tests/cli/grep_smoke_target.wat` hold
retired tokens BY DESIGN so other tools can match them. RELAND 4 already caught one over-migration
there and restored it. The tool must skip them; if it cannot, list what it changed in those files so
the restore is explicit.

## THE DONE-WHEN

```
0 failed on the floor (ORCHESTRATOR runs it) · clippy 0 · the five probe rows green
the second sweep changes 0 bytes
every remaining hand-edit traced to a SCREAM, with its reason
```

## STOP TRIGGERS

- **STOP-1 — a failing file is hand-edited before the full re-sweep.** The sweep first; the screams
  are the only hand-edit worklist.
- **STOP-2 — the sweep is scoped to the failing subset.** Full corpus. The tool changed.
- **STOP-3 — census bait is migrated.** Skip it, or report what moved so it can be restored.
- **STOP-4 — idempotence is offered as done-ness.** It is necessary, not sufficient — a tool that
  declines everything is idempotent. The bar is the floor.
- **STOP-5 — the wall is weakened.** Every remaining site is real.
