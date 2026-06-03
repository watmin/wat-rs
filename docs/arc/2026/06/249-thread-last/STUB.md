# Arc 249 — thread-last `->>` (and the `->` glyph reckoning) — STUB

**Status:** ⏸ **STUBBED 2026-06-04.** Banked from arc 247's SCORE as "the `->>` sibling." In the **pre-232 gate** (builder's call — resolve before rejoining arc 232; see `docs/arc/2026/05/232-defprotocol-extend-type/RESUME-CONTEXT.md`). LOW intrinsic priority (the builder rarely threads) — but **declared on disk now to kill a compaction risk** (a banked-in-prose-only arc is lossy across a gap) **and to correct a buried false premise** (below).

## The banked intent (247)

Arc 247 flipped the seq-HOFs to Clojure fn-first (`(map f xs)`, `(filter pred xs)`, …). Clojure threads fn-first seq-HOFs with **thread-last `->>`** — the value flows into the **last** argument of each step: `(->> xs (map f) (filter pred))`. 247's SCORE banked `->>` as the natural sibling so chains like `(map f (filter pred xs))` stop nesting.

## ⚠ CORRECTION — the false premise propagated into 247

247's SCORE (`:26`) and BRIEF (`:38`) assert/assume **"only `->` (thread-first) exists."** **That is FALSE** (verified on disk 2026-06-04):

- `->` in wat is the **return-type annotation arrow** — `(:wat::core::defn :f [a <- :T] -> :Ret body)`, used ~12× in `src/check.rs` (5684, 7085, 7246, 8665, 8773, 8889, 8974, 9103, 9217, 9328, 9473, 9591, 9706) + `src/thread_io.rs` (`(readln -> :T)`).
- There is **NO threading macro** in the substrate — neither thread-first `->` nor thread-last `->>`. `grep` for thread-first/thread-last/pipe macros returns nothing.

The "`->` thread-first exists" claim originated as an **unverified session assertion** and propagated into 247's docs. 247's DESIGN (`:33`) correctly *hedged* ("confirm whether the substrate has `->>`"); the SCORE then stated it as fact without verifying. 247's SCORE is immutable (`feedback_inscription_immutable`) — **this STUB forward-corrects it.** (Same failure class as the 235↔232 mis-coupling caught the same day: assert-without-grounding.)

## The real design question — the glyph collision

Clojure threading uses **`->`** (thread-first) + **`->>`** (thread-last). But wat's **`->` is already the type-arrow.** Minting Clojure-style threading collides with it. So 249 is a DESIGN decision, not a mint:

- **The positions differ** — the type-arrow is *infix* in argspec position (`[…] -> :Ret`); Clojure `->`/`->>` are *call-heads* (`(-> val step…)`). Positional disambiguation MIGHT be parseable. But same-glyph-two-meanings is a cold-read readability hazard (against the LLM-first / `intueri` ethos).
- **Options at design (246.0-style four-questions):**
  1. **Positional disambiguation** — `(-> …)`/`(->> …)` in call-head = threading; `->` infix in argspec = type-arrow. Parseable, but glyph-overloaded.
  2. **Different glyphs/names for threading** — avoid the `->` collision entirely (a non-colliding name).
  3. **Don't add threading** — given LOW value (builder rarely threads) + the collision cost, a four-questions verdict to *skip it* is a legitimate "resolved." This is a real candidate, not a cop-out.

## Why it's in the pre-232 gate

Builder wants it resolved before rejoining 232 (2026-06-04). **"Resolved" explicitly includes a four-questions decision to NOT build threading** (the collision + low value may not clear the bar). Either way it's small: one design stone (the verdict) + at most one macro stone.

## Refs

- `docs/arc/2026/06/247-clojure-hof-order/{SCORE,DESIGN,BRIEF}.md` — the banked sibling + the false premise.
- `->` type-arrow sites: `src/check.rs` (see list above), `src/thread_io.rs`.
- The gate: `docs/arc/2026/05/232-defprotocol-extend-type/RESUME-CONTEXT.md`.
