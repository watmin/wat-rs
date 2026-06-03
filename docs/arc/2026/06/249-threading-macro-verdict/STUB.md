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

## The `->` "collision" — RESOLVED by Clojure precedent (positional)

This is **Clojure's own situation, not a novel wat hazard** — an earlier draft over-framed it. In the Clojure ecosystem `->` already does double duty:

- **`clojure.core/->`** = the thread-first macro (call-head: `(-> x f g)`).
- **core.typed's function-type syntax** = `[Params -> Return]` — `->` is the function/return arrow *inside a type annotation*.

They coexist by **context** (call-head vs type-annotation position); no actual clash. wat's `->` is the core.typed-style return arrow (`[args] -> :Ret`; established earlier this session: `<-` ≈ `:-`, `->` = the fn arrow). So wat **inherits Clojure's resolution**: `->` as a *form-head* = thread-first macro; `->` *infix in a signature* = type arrow. Positionally unambiguous — the type arrow is never a form-head. Per *Clojure-faithfulness-includes-its-warts*, wat follows suit.

So the **HOW is settled** (the Clojure precedent), not an open verdict:
- `->>` (thread-last) — glyph free; mint it.
- `->` (thread-first) — double-duty by position, exactly as Clojure; no new mechanism, just parse a `(-> …)` form-head as threading.

The **only remaining open question is WHETHER to build threading at all** — LOW value (the builder rarely threads). A small four-questions verdict: faithfully mint both, or skip as not-worth-it. The glyph is no longer a reason to skip.

## Why it's in the pre-232 gate

Builder wants it resolved before rejoining 232 (2026-06-04). **"Resolved" explicitly includes a four-questions decision to NOT build threading.** Either way it's small: one design stone (the verdict, likely an `intueri` cast on the glyph question) + at most one macro stone for `->>` (and `->` per the verdict).

## Refs

- `docs/arc/2026/06/247-clojure-hof-order/{SCORE,DESIGN,BRIEF}.md` — the banked sibling + the false premise.
- `->` type-arrow sites: `src/check.rs` (list above), `src/thread_io.rs`.
- The gate: `docs/arc/2026/05/232-defprotocol-extend-type/RESUME-CONTEXT.md`.
