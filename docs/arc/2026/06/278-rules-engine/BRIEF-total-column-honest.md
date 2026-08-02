# BRIEF — get `intrinsic_meta`'s `total` column honest (board #52 = stone S1+S2)

**Spec:** `DESIGN-STONE-where-admits-only-rete-ops.md`, strike-order row **S1+S2 / #52**.

One act on one table, in two directions. S1 and S2 were drawn as separate stones; they edit the **same
`matches!` expression** in `src/rete/purity.rs` (`intrinsic_meta`) — the f64 ops at `:224-230`, the four
holon verbs at `:372-373` — so they are one strike, not two.

## The definition you are auditing against — builder-ruled, stricter than IEEE

> **Total = PRODUCES AN ORDINARY VALUE on every input. NOT "never raises."**
> NaN and ±Inf are **UNDEFINED**. IEEE reaches totality by sentinel, and a sentinel is the substrate's
> own name for undefined — NaN is *worse* than a raise, because **every comparison against it is false**,
> so the rule silently does not fire.

T1 set the `total` column by **default-deny** and never applied that stricter definition to the entries
it set TRUE. That is the gap.

## Read in order

1. `src/rete/purity.rs:159-270` — `intrinsic_meta`, the table you are correcting. Note it already
   distinguishes carefully (the `string::` arm sets `total` per-verb with a stated reason per verb —
   that is the standard of evidence to match).
2. `src/rete/purity.rs:358-380` — the `pure_det` block containing the four holon verbs.
3. `src/rete/purity.rs:945` — why the holon four were ruled pure on 2026-08-01: leaving them
   unclassified had *"welded shut R4's designed VSA seam."*

## Direction 1 — remove falses marked TRUE (was S1)

**BOUNDED to entries currently `total: true`.** Not the 110-verb list. One question per verb, answered
by **reading its implementation**, never by reasoning about its name:

> *Can this produce NaN or ±Inf?*

The stone's candidates — **verify each; do not trust this list, it is a starting point:**

```
f64::+  f64::-  f64::*            <- overflow to ±Inf        => likely NOT total
bigint::to-f64  rational::to-f64  <- unbounded source → +Inf => GROUND IT
i64::to-f64                       <- i64 max ≪ f64 max        => likely keep
bigint::+ - *  rational::+ - *    <- arbitrary precision      => likely keep
rational/numerator /denominator   <- accessors                => likely keep
```

**Every changed entry needs its reason in a comment**, in the register of the `string::` arm's — which
verb, which input, which implementation line proves it.

**The closure property this buys, and why it matters:** if every *producer* of NaN/±Inf requires
`:undefined`, NaN cannot arise inside a `where` at all, so comparisons downstream are safe **by
construction** rather than by audit.

## Direction 2 — add trues marked FALSE: the four holon verbs (was S2)

All four are `total: false` today, by default-deny, not by judgement. **Arming the fence without
classifying them re-welds R4's VSA seam shut one day after it was opened.**

**Grounded already — do NOT re-derive, cite and move on:**

- **`dot` cannot overflow.** `Vector.data` is `Vec<i8>`; `dot_raw` sums `(x as i64) * (y as i64)`. You
  would need d ≈ 5.7×10¹⁴ elements to overflow the i64 accumulator, and the subsequent `as f64` cannot
  overflow. Its real partiality is **dimension mismatch**, not arithmetic.
- **`cosine` cannot NaN** — `holon-rs/src/kernel/similarity.rs` guards `norm < 1e-10 → 0.0`.
- **But that `0.0` is a live mask on a REACHABLE input** — proven by run 2026-08-02,
  `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`: `(vector-blend v v 1.0 -1.0)` cancels
  every i8 cell to zero in two lines of ordinary wat. Control: genuine unrelatedness reads `-0.0086`,
  never exactly `0.0`.

**What you are recording for each of the four is its classification and the reason** — under the
measurement/predicate law already ruled in the stone:

| verb | kind | consequence for the `total` column |
|---|---|---|
| `cosine`, `dot` | **measurements** | their undefined points are real and must eventually be *faced*, not absorbed |
| `coincident?`, `presence?` | **predicates** | total by contract — an undefined comparison yields `false`, which is the honest answer to the question asked |

## ⛔ SCOPE — classification ONLY

**This strike changes classification booleans and the comments that justify them. Nothing else.**

- **STOP-1 — do NOT convert any verb to an outcome enum.** Turning `cosine`/`dot` into outcome-returning
  verbs is a separate strike over 56 call sites, and it is **blocked on an intueri cast** for the
  degenerate variant's name that has not happened. If the work seems to require it, you have left scope.
- **STOP-2 — do NOT arm the fence.** `total?` is deliberately UNARMED at `compile-condition`
  (`wat/rete.wat:599-600`). Arming is #57, after the corpus migrates. Nothing here touches `rete.wat`.
- **STOP-3 — do NOT widen the audit.** Entries currently `total: false` stay false unless they are one
  of the four holon verbs. Default-deny is correct for everything unmeasured; "probably fine" is not a
  measurement.
- **STOP-4 — a name is not evidence.** `bigint::+` sounding arbitrary-precision is not the same as
  reading that it is. Every verdict cites the implementation.

## Expect the floor to move, and re-point rather than weaken

Changing a classification can redden a test that asserts the old one. **That is the classification
changing, not a regression** — a test asserting `f64::+` is total would now be asserting a lie, and the
honest fix is to re-point it at what is now true, with a comment saying why it moved. Do not weaken an
assertion to keep it green. If a test's *subject* dies entirely, say so and leave it; the orchestrator
dispositions it.

Also check `UNREVIEWED_BASELINE` in `purity.rs` — it is a count, and a count that no longer matches its
subject is a gate that has stopped measuring.

## Gates — run these, in this order, and report each result line

```
cargo build --release
cargo test --release --lib -- purity          # the axis's own unit tests
cargo test --release --test lint              # repo lints
cargo test --release --test rete              # the fence's consumers
```

Do **not** run the full `cargo nextest run` — the orchestrator weighs the floor centrally, once.

## Do not

Do not commit. Do not push. Do not stash. Do not revert anything you did not write.
