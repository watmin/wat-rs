# 300 · DESIGN STONE C5c — NaN is UNORDERED, and `<=` stops lying about it

> **STATUS: DRAWN 2026-08-16. STRIKE-READY.** Builder: *"the comparator... we need to fix that... no warts."*
> Closes the defect Stone **C5b** deliberately flagged-and-preserved. C5b built the mechanism this needs;
> this stone spends it.

## THE DEFECT

```clojure
(:wat::core::<  1 (:wat::core::f64::/ 0.0 0.0))   ⇒ false      ; correct
(:wat::core::<= 1 (:wat::core::f64::/ 0.0 0.0))   ⇒ TRUE       ; WRONG — IEEE 754 says false
(:wat::core::>= 1 (:wat::core::f64::/ 0.0 0.0))   ⇒ TRUE       ; WRONG
```

Measured live, this session. IEEE 754: **every** comparison involving NaN is false, except `!=`.

**Why it happens.** `values_compare` maps NaN to `Ordering::Equal` (`unwrap_or`, `runtime.rs:9798`), and
`eval_compare` spells `<=` as `|o| o != Ordering::Greater`. `Equal` passes that predicate. The bug is not
the predicate and not the arm — it is that **`Ordering` has three states and IEEE needs a fourth:
UNORDERED.** `Equal` is the only lie available, so the substrate tells it.

## WHY THIS IS SMALL NOW — C5b already built the fourth state

`src/value/numeric_order.rs` (landed today, stone C5b):

```rust
pub(crate) enum NumOrd { Ord(Ordering), Incomparable, NotNumeric }
```

**`Incomparable` MEANS NaN.** The door already computes exactly the fact `eval_compare` needs; nothing
consumes it yet on this path because C5b deliberately preserved the wart rather than fold two stones
together.

## THE MEASURED BLAST RADIUS — and the worry that proved false

The obvious objection: *"sorting needs a total order, so NaN→Equal has to stay."* **Measured, and it is
not true here.** `values_compare`'s consumers:

| site | what it is |
|---|---|
| `runtime.rs:9867` `:9882` | its OWN Vec/Tuple recursion |
| `runtime.rs:9895` `:9900` `:9901` | its OWN Option/Result recursion |
| **`runtime.rs:9933`** | **`eval_compare` — the only external consumer** |

Sorting goes through `collection::transform::eval_vec_sort_by` (arc 251's comparator-sort primitive), not
this path. There is **no sort/min/max consumer to protect.**

And `eval_compare` is the single door for the whole family — `:wat::core::< > <= >=` plus the per-type
`i64::`/`f64::` spellings (`runtime.rs:5308-5332`), each passing an `Ordering` predicate.

## THE SHAPE

`eval_compare` consults the door FIRST, and falls back only for non-numerics:

```rust
match numeric_order(&a, &b) {
    NumOrd::Ord(o)       => pred(o),
    NumOrd::Incomparable => false,          // ← IEEE: NaN is unordered; ALL FOUR ops false
    NumOrd::NotNumeric   => /* existing values_compare path: String, bool, keyword,
                               Instant, Duration, Vector, Vec, Tuple, Option, Result */
}
```

One function. No new types. No change to `values_compare` itself.

**`=` and `not=` are untouched** — they route through `values_equal`, which is category-aware and never
consults an ordering. `(not= 1 ##NaN)` stays `true`, matching IEEE's one exception.

## ⚠ THE EDGE THIS STONE DOES NOT CLOSE — state it, do not discover it

A **`:wat::core::Vector<:wat::core::f64>`** containing NaN still recurses through `values_compare`, which
keeps NaN→`Equal`. So after this stone:

- **scalars** get IEEE semantics — `(<= 1.0 NaN)` ⇒ `false`
- **NaN nested inside a collection** keeps total-order semantics — element-wise lex treats it as `Equal`

That asymmetry is **defensible**: a collection ordering wants totality, and lexicographic comparison with
a non-total element comparator is not well-defined. But it is a real seam, and it is written here so the
next reader meets a documented decision instead of a fresh surprise. If it is ever wrong, it is its own
stone.

*(Naming note, per the builder's correction this session: `:wat::core::Vector<T>` is the sequence, backed
by `Value::Vec`; `:wat::holon::Vector` is the VSA hypervector, backed by `Value::Vector`. `:wat::core::vec`
is RETIRED — `retirement.rs:114` still carries its remedy. Use the wat names.)*

## THE GATE

| # | assertion | at HEAD |
|---|---|---|
| 1 | `(< 1 NaN)` ⇒ false | green — must stay |
| 2 | **`(<= 1 NaN)` ⇒ false** | **RED — the defect** |
| 3 | **`(>= 1 NaN)` ⇒ false** | **RED — the defect** |
| 4 | `(> 1 NaN)` ⇒ false | green — must stay |
| 5 | same four with NaN on the LEFT — all false | 2 RED |
| 6 | `(< NaN NaN)` · `(<= NaN NaN)` ⇒ both false | `<=` RED |
| 7 | `(not= 1 NaN)` ⇒ **true**, `(= 1 NaN)` ⇒ **false** — `values_equal` untouched | green — must stay |
| 8 | ±∞ unchanged: `(< 1 ##Inf)` ⇒ true, `(<= 1 ##Inf)` ⇒ true | green — must stay |
| 9 | C5b's exactness intact: `(< 9007199254740992.0 9007199254740993)` ⇒ true | green — must stay |
| 10 | non-numeric ordering unchanged: String, bool, keyword, Instant, Duration, Vector, Option, Result | green — must stay |
| 11 | i64/f64 per-type spellings agree with the polymorphic ones on every NaN row | mixed |
| 12 | floor green, clippy 0 | — |

**NaN is produced by division, not a literal** — `##NaN` is not wat syntax:
`(:wat::core::f64::/ 0.0 0.0)` ⇒ `#wat-edn.float/nan`, `(:wat::core::f64::/ 1.0 0.0)` ⇒ `#wat-edn.float/inf`,
`(:wat::core::f64::/ -1.0 0.0)` ⇒ `#wat-edn.float/neg-inf`. All three verified live this session.

**Row 7 is the one that catches an over-eager fix.** Making NaN unordered must NOT leak into equality;
`(not= 1 NaN)` is `true` in IEEE and must remain so.

## STOP TRIGGERS

- **STOP-1 — `values_equal` changes.** `=`/`not=` are category-aware and out of scope entirely.
- **STOP-2 — a non-numeric ordering changes.** String/bool/keyword/Instant/Duration/Vector/Vec/Tuple/
  Option/Result must be byte-identical; they reach `eval_compare` through `NotNumeric`.
- **STOP-3 — `values_compare`'s own NaN→`Equal` is changed.** This stone does not touch it; that is the
  collection-totality seam above and it stays.
- **STOP-4 — C5b's exactness regresses.** Row 9 is the whole of yesterday's stone.
- **STOP-5 — more than ~6 reds you cannot attribute.** Report and stop.

## Kin

- `DESIGN-STONE-C5b-exact-mixed-numeric-order.md` — built the `NumOrd` door and **flagged this wart
  rather than folding it**; its gate row 12 pins the wrong answer deliberately so this stone can find it.
- `DESIGN-STONE-rational-C5-mixed-compare.md` — the contract, amended.
- `docs/arc/2026/06/296-diagnostics-fully-edn/SCORE-296-WaveB2-wat_lang.md` — the flag-don't-fold posture.
