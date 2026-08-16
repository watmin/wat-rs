# NOTE (arc 300) — C5's mixed-numeric ordering loses precision above 2⁵³ and gives a WRONG answer

**Filed 2026-08-16. RULED: FIX IT** (builder, this session: *"we fix the bug"*). Surfaced while
adjudicating a 296-recapture test that turned out to assert C5's *superseded predecessor*; the
supersession was correct, and this is the separate, real defect the detour uncovered.

## The flaw, in one line

**`(< 9007199254740992.0 9007199254740993)` returns `false`. The true answer is `true`.**

## Grounded (2026-08-16, live via the MCP eval session)

```clojure
(:wat::core::< 9007199254740992.0 9007199254740993)   ⇒ false      ; TRUE is correct
(:wat::core::< 9007199254740993 9007199254740992.0)   ⇒ false      ; correct, but BY ACCIDENT
```

2⁵³ = 9007199254740992 is the last integer f64 represents exactly. 2⁵³+1 = 9007199254740993 is **not**
representable and rounds to 9007199254740992.0. So coercing the `i64` operand to `f64` makes the two
compare **equal**, and `<` yields `false` in both directions.

The reverse direction is right only because "coerced-equal" and "genuinely greater" both produce
`false` — **one accident, not two correct answers.** Any test that only checked that direction would
have passed.

## What it contradicts — C5's own pinned contract

`DESIGN-STONE-rational-C5-mixed-compare.md` pins:

> *"**Ordering `< > <= >=`** on mixed numerics → **the numeric-value comparison** (`(< 1 2.0)` → `true`)."*

A numeric-**value** comparison of 2⁵³ and 2⁵³+1 is `true`. The implementation is doing a
coerce-to-`f64` comparison, which is a different operation that agrees with the value comparison only
below 2⁵³.

**C5 itself is not in question.** Accepting mixed-numeric comparison at the checker was the right
reversal — it matched eval and clj, and it correctly superseded arc 237.8a's *"cross-numeric path
DELETED"*. The contract is fine; the implementation under-delivers it at the top of the i64 range.

## ⛔ What must be settled BEFORE the fix — do not skip this

**clj coerces here too.** `(< 9007199254740992.0 9007199254740993)` in Clojure also returns `false`,
because Clojure's `<` on a double and a long promotes to double. So C5's two justifications —
*"matching eval and clj"* and *"the numeric-value comparison"* — **diverge at exactly this boundary**,
and the stone does not say which one wins.

Two coherent designs; pick deliberately, do not drift into one:

| | rule | consequence |
|---|---|---|
| **EXACT** | compare mathematical values (promote the f64 to rational, or compare exactly) | correct at every magnitude; **diverges from clj** above 2⁵³ |
| **CLJ-FAITHFUL** | keep coerce-to-f64 | matches clj exactly; **wrong answers** above 2⁵³, and C5's "numeric-value" wording must be corrected to say so |

The builder ruled *"we fix the bug"*, which reads as **EXACT** — but the wording of C5 should be
amended either way, because right now it promises one thing and does the other.

## The fix's shape (unmeasured — establish before building)

`values_compare` is the arm C1–C4 added (per C5's own text). The exact form is a promote-to-rational
comparison, which the substrate already has — 300's rational work is the reason a rational type exists
here at all. **Census the mixed-numeric compare arms first**; do not assume there is one.

## THE GATE — the negative control is keepable, so KEEP IT

Per `docs/DUNGEON-CRAWL.md` Phase 3: this control is expressible as ordinary test code, so it is
**banked as a test**, not performed and discarded. It must cover **both directions**, because one of
them passes under the buggy implementation:

```
(< 2⁵³.0  2⁵³+1)   must be TRUE     ← fails today; the load-bearing row
(< 2⁵³+1  2⁵³.0)   must be FALSE    ← passes today BY ACCIDENT; pins the accident
(<= 2⁵³.0 2⁵³+1)   must be TRUE
(=  2⁵³+1 2⁵³.0)   must be FALSE    ← `=` stays category-aware per C4; unchanged by this fix
```

Also sweep `>` and `>=` — the family is six ops and the census above covers five.

## Kin

- `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C5-mixed-compare.md` — the contract
  this violates, and the correct supersession of 237.8a.
- `tests/wat_lang/wat_not_eq.rs::not_eq_f64_cross_numeric_coerce` — the 296-cohort test that asserts
  **237.8a's retired design**. It is **SUPERSEDED, not a finding**: retire or rewrite it against C5.
  Do not "recapture" it — its expectation is a design we deliberately replaced.
