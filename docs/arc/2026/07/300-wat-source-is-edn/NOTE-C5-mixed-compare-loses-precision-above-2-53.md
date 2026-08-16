# NOTE (arc 300) — C5's mixed-numeric ordering loses precision above 2⁵³ and gives a WRONG answer

> **DISPOSITION 2026-08-15 — SETTLED. See `DESIGN-STONE-C5b-exact-mixed-numeric-order.md`.**
> The fork this NOTE refused to pick is ruled **EXACT**. This NOTE's grounding was correct and its demand
> for a census paid off — but **three of its statements were wrong**, and the corrections are below, at the
> bottom. Read the stone for the settled shape; read this for how it was found.

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

## ⛔ CORRECTIONS 2026-08-15 — three things above are wrong

Written when the census this NOTE demanded had not yet been run. The census ran; it disagreed with the NOTE
on three counts. Recorded here rather than edited away, because **the pattern of the errors is the lesson**:
every one of them is a scope estimate made by reading rather than measuring.

**1. "the family is six ops and the census above covers five" — it is FOUR ops.**
`=` and `not=` route through `values_equal`, which is **category-aware** per C4: an i64 and an f64 are
different numeric categories, so it returns `Some(false)` *without ever coercing*. No coercion, no rounding,
no defect. `=`/`not=` were never in this family. The affected ops are **`< > <= >=`**.

**2. "The fix's shape … `values_compare` is the arm" — there are THREE tables, not one.**
The NOTE's own instruction (*"census the mixed-numeric compare arms first; do not assume there is one"*)
was right and was worth more than it knew:

| site | tower | mixed i64↔f64 | not-comparable |
|---|---|---|---|
| `src/runtime.rs:9793` `values_compare` | i64·u8·f64·BigInt·Rational | lossy | `None` → `TypeMismatch` |
| `src/runtime.rs:13020` `walk_match_clause` | i64·u8·f64 | lossy | **silent `false`** |
| `src/rete/matcher.rs:954` `compare_values` | i64·u8·f64 | lossy | `None` |

Table 3's doc comment names its own duplication and points at `runtime.rs ~:10615` — a line that has since
moved to `:13020`. **A comment that tracks a clone by line number is a clone that will drift, and it did.**

**3. "the census above covers five [ops]" understated the LOSSY PAIRS too — there are six, not two.**
`values_compare` coerces down to f64 for `i64↔f64`, `BigInt↔f64`, **and** `Rational↔f64`, both directions.
`(< 1N 2.0)` and `(> 3.0 1/2)` are the identical defect one type over. The NOTE grounded only the i64 pair
and generalised from it.

### What the NOTE got right, and it was the load-bearing part

- The bug, grounded, **in both directions** — including that the reverse direction is right *by accident*.
  That instinct is what made the gate table demand both directions, and row 3 of it now pins the accident.
- The refusal to settle EXACT-vs-clj-faithful by reading. It was genuinely undecidable from the stone's
  text, and it was the builder's to rule. He ruled: *"we fix the bug."*
- The demand for a census. It was the only reason tables 2 and 3 were ever found.

### One claim still unverified

*"clj coerces here too"* was reasoned, not run — no JVM in this loop by standing direction. It is carried as
**unverified** in the C5 stone's amendment. If it turns out Clojure agrees with wat, the documented
divergence disappears.

## Kin

- `DESIGN-STONE-C5b-exact-mixed-numeric-order.md` — **the settled disposition.** One exact door, three
  callers, each keeping its own policy for the non-`Ordering` outcomes.
- `docs/arc/2026/07/300-wat-source-is-edn/DESIGN-STONE-rational-C5-mixed-compare.md` — the contract
  this violates, and the correct supersession of 237.8a.
- `tests/wat_lang/wat_not_eq.rs::not_eq_f64_cross_numeric_coerce` — the 296-cohort test that asserts
  **237.8a's retired design**. It is **SUPERSEDED, not a finding**: retire or rewrite it against C5.
  Do not "recapture" it — its expectation is a design we deliberately replaced.
