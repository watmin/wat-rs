# NOTE (arc 109 vocabulary) — integer division is the ONLY verb whose answer depends on which integer type you happen to hold

**Filed 2026-08-28 by the builder's direction, during arc 255 Stone O. A POINTER, not a decision.**
Surfaced while probing `:wat::core::apply`'s doors: the builder asked whether `(+)`→0 · `(+ 1)`→1 ·
`(+ 1 1)`→2 · `(+ 1 1 1)`→3, which is exactly right and exactly what wat does. Running the same
question across the rest of the arithmetic surface found **one** divergence, and it is not the one
this note was expected to be about.

## ⛔ FIRST — THE SIBLING NOTE IS STALE, AND ITS STALENESS IS THE REASON THIS ONE READS SO DIFFERENTLY

`NOTE-rational-number-support.md` (filed 2026-07-03) says, as its pivotal grounded facts:

> *"wat has **no rational value type** yet, so wat's readers refuse it."*
> *"The runtime's numeric values are `i64`/`f64`/`u8` (note: **no `BigInt` in the runtime** — only in
> the EDN data layer)."*
> *"Layer 2 — the language … the bigger half — needed for wat PROGRAMS to compute with rationals."*

**All three were overtaken by arc 300 stones C1/C2 and have been false for weeks.** Measured today
against `./target/release/wat` at `fe602d707`:

```
1/2                        =>  1/2      ← reads as a SOURCE literal, prints in normal form
(:wat::core::/ 1/2 1/3)    =>  3/2      ← rational arithmetic, in the language
(:wat::core::+ 1/2 1/2)    =>  1N       ← and it COLLAPSES to bigint on exact results
(:wat::core::/ 1N 2N)      =>  1/2      ← bigint division collapses to rational — Clojure exactly
(:wat::core::/ 4N 2N)      =>  2N       ← and back to bigint when it divides evenly
(:wat::core::/ 1 1/2)      =>  2N       ← i64 ⊕ rational contagion works
```

13 rational/bigint verbs are registered (`:wat::rational::{+ - * / numerator denominator to-f64}`,
`:wat::bigint::{+ - * / to-f64 to-rational}`), `:wat::i64::to-rational` and `:wat::i64::to-bigint`
exist, and `wat/core.wat`'s `:wat::core::/` defclause carries **23 clauses** including every
contagion pair. **Layer 2 landed.** `[[feedback_a_blocker_note_is_a_claim_with_a_date_on_it]]`

> **Do not read the sibling note's "current state" or "direction" sections as live.** Its *reader*
> half (Layer 1, `crates/wat-edn`) may still be owed — this note did not measure it — but its
> premise that the LANGUAGE lacks rationals is refuted above. Whoever next touches either note
> should re-measure the `wat-edn` side and correct the sibling in place.

## The actual gap — and it is about wat's own consistency, not about Clojure

Every division path in the language is exact and Clojure-shaped **except one**:

| expression | today | Clojure | |
|---|---|---|---|
| `(/ 1N 2N)` bigint ÷ bigint | `1/2` | `1/2` | ✓ |
| `(/ 4N 2N)` bigint, exact | `2N` | `2N` | ✓ |
| `(/ 1/2 1/3)` rational | `3/2` | `3/2` | ✓ |
| `(/ 1 1/2)` i64 ⊕ rational | `2N` | `2N` | ✓ |
| `(/ 4.0)` f64 reciprocal | `0.25` | `0.25` | ✓ |
| **`(/ 1 2)` i64 ÷ i64** | **`0`** | **`1/2`** | ⛔ |
| **`(/ 4)` i64 reciprocal** | **`0`** | **`1/4`** | ⛔ |

★ **State it as a wat property, not a parity gap, because that is the sharper form:**
**`/` is the only verb in the language whose answer depends on which integer type the caller happened
to be holding.** `(/ 1 2)` is `0`; `(/ 1N 2N)` is `1/2`. Same mathematical question, two answers,
no diagnostic, and the i64 one silently discards the remainder. Every other verb —
`+ - *`, and `/` on every other type pair — gives one answer per question.

The truncation is not an oversight in a corner: `wat/core.wat:384` declares it deliberately,
`([x <- i64 y <- i64] -> :wat::core::i64 (:wat::i64::/ x y))`, and the arm sitting 25 lines below it
has a comment explaining, carefully and correctly, why bigint division **collapses to rational**
*"(clj: `(/ 1N 2N) => 1/2`)"*. The two arms were written with different intentions in the same form.

## What the deciding strike must weigh

**1. It is a RETURN TYPE change, which is why this is a ruling and not a fix.** The arm declares
`-> :wat::core::i64`. Making it exact means `-> :wat::core::rational`, and every caller annotated to
receive an `i64` from `(/ a b)` stops type-checking. That is the whole cost, and it is the reason
this note exists instead of a patch.

**2. The escape hatch already exists and is the same one Clojure ships.** `:wat::core::quot`
(`(quot 7 2)` → `3`), `rem` (`1`), and `mod` (`1`) are all present and correct. In Clojure, `/` is
exact and `quot` truncates; a caller that wants the old behaviour writes `quot`, which is what they
should have been writing to say what they meant.

**3. The conversion the fix needs already exists.** `:wat::i64::to-rational` is registered, and the
i64⊕rational contagion arms already route through it. The arm's body would be
`(:wat::rational::/ (:wat::i64::to-rational x) (:wat::i64::to-rational y))` — the same shape the
bigint arm uses — and **collapse comes for free**: `(/ 4 2)` would still be `2` (as `2N`), because
`:wat::rational::/` already collapses exact results, proven above by `(+ 1/2 1/2)` → `1N`.

**4. The corpus ripple is small, and measured.** Only **10** textual uses of the polymorphic
`:wat::core::/` exist across `wat/ wat-scripts/ tests/ crates/ examples/`, in 9 files:
`wat/holon/{Reject,ReciprocalLog,Circular}.wat`, `wat/rete/acc.wat`, and 5 test files (3 of which are
the arc-300 rational probes and are *about* this). The type-locked `:wat::i64::/` has 96 uses and is
**unaffected** — it is the leaf, it stays truncating, and it is the honest thing to call when you
mean machine division. ⚠ This census is TEXTUAL and counts neither the `-> :wat::core::i64` return
annotations that would have to move nor any `(/ …)` reached through a macro; **the deciding strike
must impose the change and read the compiler's screams rather than trust this 10.**
`[[feedback_impose_the_check_and_read_the_screams]]`

**5. `(/ 4)` — the 1-ary reciprocal — is the same defect and must move with the 2-ary arm.**
`wat/core.wat:381` is `(:wat::i64::/ 1 x)`, so `(/ 4)` → `0`. Fixing 2-ary and leaving 1-ary would
leave the inconsistency intact at a different arity.

**6. There is a 3+-ary i64 fold arm** (`wat/core.wat:389`) that folds through `:wat::i64::/`. If the
2-ary arm returns a rational, that fold's accumulator type changes; the bigint arm's own comment
already worked through exactly this problem and concluded *"`:wat::rational::/` accepts a bigint
accumulator (self-promoted), so this fold CAN carry a collapsed intermediate."* The rational N-ary fold at
`wat/core.wat:437-445` is the worked precedent to copy.

## Where this belongs on the road

Builder's road, 2026-08-27: `1 home everything → 2 crates → 3 kill :: → 4 symbol heads →
5 EDN/Clojure-compliant syntax → 6 totality`. This is **step 5**, not arc 255, and it is filed here
rather than struck now for that reason. It is not blocked on anything: the value type, the reader
literal, the conversion verb, the collapse behaviour and the `quot` escape hatch are all on disk
today. It waits on the builder's word about a return type, and on its turn.

## Refs

- `wat/core.wat:378-489` — the `:wat::core::/` defclause; the runtime reports 23 clauses (the NoMatchingClause dump for `(/)` enumerates 0..22). Line 384 is the arm.
- `wat/core.wat:407` — the bigint arm's comment: the Clojure semantics, already reasoned out.
- `wat/core.wat:437-445` — the rational N-ary fold: the precedent for point 6.
- `src/intrinsic/rational.rs`, `src/intrinsic/bigint.rs` — the registered arms (arc 300 C1/C2).
- `NOTE-rational-number-support.md` — the sibling. **Stale on its premise; see the banner above.**
- `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-O-one-declaration-feeds-both-doors.md`
  — the design whose correction banner surfaced this.
- `wat-scripts/scratch-pad/255-stone-o-apply-has-three-broken-doors.wat` — the probe that ran the
  builder's `(+)`/`(+ 1)`/`(+ 1 1)`/`(+ 1 1 1)` question and found this next door.
