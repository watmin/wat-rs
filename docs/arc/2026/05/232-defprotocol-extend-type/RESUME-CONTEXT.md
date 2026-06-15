# Arc 232 — RESUME CONTEXT (the rejoin pointer)

**Status: PARKED, pending arc 237 — which is now CLOSED (2026-06-04, `e6c9ea28`).** This is the arc that waited on 237. Do not resume blind: read this first.

## Where 232 stands

- **232.0 + 232.0a SHIPPED** — `:wat::core::apply` (call-by-keyword) + the typed-entities reflection layer (`extract-classifier` + `Bind/left`/`Bind/right`). The substrate primitives defprotocol needs.
- **232.1 FM-2-bis probe SHIPPED 3/3 PASS** (`f38e120`) — the dispatch composition proven end-to-end.
- **232.1 (defprotocol + extend-type macros, BUNDLED) — DESIGNED + BRIEFED, UNSTRUCK.** Sub-DESIGN + BRIEF + EXPECTATIONS exist; no SCORE. This is the parked stone. It ships two defmacros (the Clojure four-corner middle: defrecord ✓ + **defprotocol + extend-type** + satisfies?).

## Why it waited on 237 — and what changed underneath it

The original 232.1 sub-DESIGN (2026-05-23) built defprotocol as a macro over hand-rolled **classifier-cond dispatch** (`extract-classifier` + `apply`). The 237 DESIGN deferred defprotocol to *after* 237 with a reshaped target (237 DESIGN `:186`, `:514`):

> *"AFTER arc 237 closes, defprotocol becomes a macro layer over **defclause + typeunion + extend-type** for open extension. Reduced scope (~2-3 stones)."*

237 consolidated the substrate's dispatch into **defclause** (monomorphic ops) + **intrinsics** (type-level computation) — see `docs/OP-PLACEMENT.md`. That is the foundation defprotocol should build on now. Striking the pre-237 design would build defprotocol on a classifier-cond mechanism that 237 superseded.

## THE GATE — 232 HOLDS these closures (rejoin only when ALL FOUR close)

Builder's decision 2026-06-04: **four arcs resolve before 232.1 re-opens.** 232 is the *holder*; it does not rejoin until the gate is empty.

| Arc | What | State | Why it precedes 232 |
|---|---|---|---|
| **246** | `src/collection/` warded home | OPEN (246.0 DESIGN done) | forward-arc from 237's death; current |
| **245** | wat-corpus warding | STUB (needs 245.0 instrument design) | forward-arc from 237's death |
| **249** | threading macros: `->` + `->>` (BOTH forms) | STUB on disk (promoted from thread-last-only — neither form exists) | builder: do before 232 |
| **235** | records-with-rich-VSA | PROPOSED/notes | builder's **choice — NOT a dependency** (235 is independent of 232: it extends arc 234 + uses 237's `:guard`); deliberately sequenced here |

Suggested order: **246 → 245 → 249 → 235 → rejoin 232.**

**Spawn-block hope (and rule):** we *expect* these four to spawn no new arcs. If any does, the new arc **joins this gate** — 232 stays parked until the gate is empty (`feedback_spawn_block_winding`).

## ON REJOIN to 232 — three jobs

1. **Revisit 232.1 for the defclause foundation** (see "Why it waited" above). Does defprotocol now dispatch via **defclause** (open-extension on a defclause name — flagged in the 237 DESIGN as "232.1 territory") rather than hand-rolled classifier-cond? Confirm the reduced scope (~2-3 stones); the probe (`f38e120`) may need re-aiming at the defclause path.
2. **Strike the chain:** re-settle DESIGN-STONE-232.1 → re-BRIEF → strike → SCORE → 232.3 (built-in-type extension) → 232.5 INSCRIPTION (arc 232 closes).
3. **Identify who 232 blocks — THE MAIN QUEST.** 232 (defprotocol) is the floor of the substrate side-quest stack; below it is the consumer/application work it was built *for*. 232's own DESIGN names the trigger: *"likely surfaced by Truth Engine, MTG enterprise, or trading-lab v2"* (`DESIGN.md` § "Open trigger"). Confirm the real main quest when we arrive — that is what all these side-quests served.

*Marked 2026-06-04 at 237's close; restructured to the holder-gate per builder direction. So the rejoin doesn't cost a crawl.*
