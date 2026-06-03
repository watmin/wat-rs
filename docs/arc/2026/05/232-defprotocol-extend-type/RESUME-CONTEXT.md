# Arc 232 — RESUME CONTEXT (the rejoin pointer)

**Status: PARKED, pending arc 237 — which is now CLOSED (2026-06-04, `e6c9ea28`).** This is the arc that waited on 237. Do not resume blind: read this first.

## Where 232 stands

- **232.0 + 232.0a SHIPPED** — `:wat::core::apply` (call-by-keyword) + the typed-entities reflection layer (`extract-classifier` + `Bind/left`/`Bind/right`). The substrate primitives defprotocol needs.
- **232.1 FM-2-bis probe SHIPPED 3/3 PASS** (`f38e120`) — the dispatch composition proven end-to-end.
- **232.1 (defprotocol + extend-type macros, BUNDLED) — DESIGNED + BRIEFED, UNSTRUCK.** Sub-DESIGN + BRIEF + EXPECTATIONS exist; no SCORE. This is the parked stone. It ships two defmacros (the Clojure four-corner middle: defrecord ✓ + **defprotocol + extend-type** + satisfies?).

## Why it waited on 237 — and what changed underneath it

The original 232.1 sub-DESIGN (2026-05-23) built defprotocol as a macro over hand-rolled **classifier-cond dispatch** (`extract-classifier` + `apply`). The 237 DESIGN deferred defprotocol to *after* 237 with a reshaped target (237 DESIGN `:186`, `:514`):

> *"AFTER arc 237 closes, defprotocol becomes a macro layer over **defclause + typeunion + extend-type** for open extension. Reduced scope (~2-3 stones)."*

237 consolidated the substrate's dispatch into **defclause** (monomorphic ops) + **intrinsics** (type-level computation) — see `docs/DISPATCH.md`. That is the foundation defprotocol should build on now. Striking the pre-237 design would build defprotocol on a classifier-cond mechanism that 237 superseded.

## ON REJOIN — do this, in order

1. **Revisit the 232.1 design in light of 237.** Does defprotocol now dispatch via **defclause** (open-extension: adding clauses to a defclause name after declaration — flagged in the 237 DESIGN's out-of-scope as "deferred to arc 232.1 territory") rather than hand-rolled classifier-cond? Confirm the reduced scope (~2-3 stones). The probe (`f38e120`) may need re-aiming at the defclause path.
2. Re-settle DESIGN-STONE-232.1 → re-BRIEF → strike → SCORE → 232.3 (built-in-type extension) → 232.5 INSCRIPTION (arc 232 closes).

## Queue — 232 is the rejoin target, but NOT next

237's death (2026-06-04) opened two forward-arcs being resolved **first** (builder's call):
- **arc 246** — `src/collection/` warded home (OPEN; 246.0 DESIGN done).
- **arc 245** — wat-corpus warding (unblocked, not yet opened).

**Wind: 246 → 245 → THEN rejoin here (232.1).** Also 237-gated and downstream of 232: **arc 235** (records-with-rich-VSA-encodings — waits on 237's `:guard` for per-field validation; a future *opening*, after 232).

*Marked 2026-06-04 at 237's close, so the rejoin doesn't cost a crawl.*
