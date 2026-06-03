# Arc 249 — threading macros: `->` (thread-first) + `->>` (thread-last) — STUB

**Status:** ⏸ **STUBBED 2026-06-04.** Originally banked from arc 247's SCORE as "the `->>` sibling" — **SCOPE PROMOTED 2026-06-04 to BOTH threading forms** once grounding showed *neither* exists (the dir was renamed from `249-thread-last`). In the **pre-232 gate** (builder's call — resolve before rejoining arc 232; see `docs/arc/2026/05/232-defprotocol-extend-type/RESUME-CONTEXT.md`). LOW intrinsic priority (the builder rarely threads) — but **declared on disk to kill a compaction risk** (a banked-in-prose-only arc is lossy across a gap) **and to correct a buried false premise**.

## Why it covers BOTH forms now (the promotion)

The banked intent was thread-last `->>` *only* — because 247 believed thread-first `->` already existed. **It doesn't.** Grounding (2026-06-04) showed there is **no threading macro in wat at all**, so the arc that adds threading must add *both* Clojure forms or neither:

- **`->`** (thread-first) — value flows into the **first** argument of each step. Clojure pairs it with **coll-first** ops `(-> coll (assoc :k v) (get :k))`.
- **`->>`** (thread-last) — value flows into the **last** argument of each step. Clojure pairs it with **fn-first seq-HOFs** `(->> xs (map f) (filter pred))` — the ergonomics 247's fn-first flip created the need for.

You need both because wat now has both shapes (247 made the seq-HOFs fn-first; the container ops stayed coll-first). One threader fits each.

## ⚠ CORRECTION — the false premise propagated into 247

247's SCORE (`:26`) and BRIEF (`:38`) assert/assume **"only `->` (thread-first) exists."** **FALSE** (verified on disk 2026-06-04):

- `->` in wat is the **return-type annotation arrow** — `(:wat::core::defn :f [a <- :T] -> :Ret body)`, ~12× in `src/check.rs` (5684, 7085, 7246, 8665, 8773, 8889, 8974, 9103, 9217, 9328, 9473, 9591, 9706) + `src/thread_io.rs` (`(readln -> :T)`).
- **NO threading macro** exists — neither `->` nor `->>`. `grep` for thread-first/thread-last/pipe macros returns nothing.

The "`->` thread-first exists" claim originated as an **unverified session assertion** and propagated into 247's docs. 247's DESIGN (`:33`) correctly *hedged*; the SCORE stated it as fact without verifying. 247's SCORE is immutable (`feedback_inscription_immutable`) — **this STUB forward-corrects it.** (Same failure class as the 235↔232 mis-coupling caught the same day: assert-without-grounding.)

## The real design question — an ASYMMETRIC glyph collision

The two forms are NOT symmetric in difficulty:

- **`->>` (thread-last) — glyph is FREE.** `->>` is unused; mintable cleanly as a call-head macro. No collision.
- **`->` (thread-first) — COLLIDES.** `->` is already the type-arrow. A thread-first `->` contends directly for a taken glyph.

So the design hinges on thread-first. Positions differ (type-arrow is *infix* in argspec `[…] -> :Ret`; threading `->` is a *call-head* `(-> val …)`), so positional disambiguation MIGHT parse — but same-glyph-two-meanings is a cold-read hazard (anti `intueri`/LLM-first). Options (246.0-style four-questions, ideally an `intueri` cast):

1. **Mint `->>` (clean) + positional-disambiguate `->`** — both Clojure-faithful; `->` overloaded by position.
2. **Mint `->>` (clean) + a non-colliding name for thread-first** — keep threading, drop the `->` glyph clash (less Clojure-faithful on the glyph, honest on the collision).
3. **Don't add threading at all** — LOW value (builder rarely threads) + the collision cost; a four-questions verdict to *skip it* is a legitimate "resolved," not a cop-out.

## Why it's in the pre-232 gate

Builder wants it resolved before rejoining 232 (2026-06-04). **"Resolved" explicitly includes a four-questions decision to NOT build threading.** Either way it's small: one design stone (the verdict, likely an `intueri` cast on the glyph question) + at most one macro stone for `->>` (and `->` per the verdict).

## Refs

- `docs/arc/2026/06/247-clojure-hof-order/{SCORE,DESIGN,BRIEF}.md` — the banked sibling + the false premise.
- `->` type-arrow sites: `src/check.rs` (list above), `src/thread_io.rs`.
- The gate: `docs/arc/2026/05/232-defprotocol-extend-type/RESUME-CONTEXT.md`.
