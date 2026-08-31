# DESIGN — STONE total-T4a: 27 verified totality rulings move to the registration site

> Builder: *"we continue to make the registry the sole source of truth for these properties."*

## The blocker this removes

`intrinsic_meta` (`src/rete/purity.rs`) cannot derive its `total` axis from the registry while
**431 of 432 verbs answer `@Totality Unreviewed`**. Derivation today would tell arc 278's `where`
fence "unknown" for 38 verbs it currently calls total, and the 9-file / 98-row `where`-corpus
would go red.

```
431  @Totality Unreviewed        1  @Totality Partial  (i64::/, transcribed in T2b)
 38  verbs the fence calls TOTAL
 27  of those are REGISTERED — can carry the label today   ← THIS STONE
 11  still unhomed — the named residue
```

## ★ THIS IS TRANSCRIPTION, NOT ADJUDICATION

The rete `total` sub-list is not a guess list. Its own header:

> *"This sub-list is exactly the verbs the 9-file / 98-row `where`-corpus uses inside a `where`
> (directly or via a transitively-checked user fn), **each verified total by READING its own
> implementation in `runtime.rs` (never inferred from the name)**."*

And the per-op reasoning is already written, and it is good:

> *`i64::to-f64` — a total, lossy-but-never-raising conversion (`i64::MAX` ≈ 9.2e18 is nowhere near
> f64's overflow boundary ≈1.8e308, so the result is always finite, never ±Inf).*
>
> *`f64::>` — a comparison, not an arithmetic op: `eval_f64_compare` returns a `bool` for any two
> f64 inputs including NaN/±Inf (IEEE says `NaN > x` is `false`, never a raise), so the OUTPUT
> itself can never be the undefined thing this axis polices.*

**That reasoning currently lives in a hand-list in `rete/purity.rs`, describing verbs declared in
eight other files.** Moving it to the site that declares the verb is exactly what
`NOTE-the-registry-asserts-properties-nothing-verifies.md` asked for. No verb is being judged here;
27 verdicts already made and cited are being relocated to where they belong.

## The 27, by home

```
src/intrinsic/i64.rs        < <= = > >= not= to-f64 to-string          8
src/intrinsic/f64.rs        < <= = > >= not= to-string                 7
src/intrinsic/vector.rs     length contains? get                       3
src/intrinsic/collection.rs last reverse range                         3
src/intrinsic/holon/atom.rs cosine dot coincident? presence?           4   ← the builder-ruled four
src/intrinsic/special/…     if · let                                   2
```

## ⛔ The one contract decision, pinned: ZERO BEHAVIOUR CHANGE

**Nothing in production reads `IntrinsicEntry.totality` yet** — it carries `#[allow(dead_code)]`
naming T4 as when readers arrive. So changing 27 declarations from `Unreviewed` to `Total` changes
no runtime behaviour, and **the floor must come back identical**. That is this stone's strongest
acceptance criterion and the reason it is drawn separately from the derivation.

★ **`intrinsic_meta`'s total sub-list is NOT touched.** This stone only adds to the registry. The
duplicate that results is deliberate, temporary, and collapsed by T4b — which is drawn immediately
after, not "later". A duplicate held open indefinitely is the shape 255.1c retired
(`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`); a duplicate held open for exactly one
stone, with the collapse already drawn, is a migration.

## What it leaves — and why that is the point

After T4b derives, `intrinsic_meta`'s total sub-list holds **exactly the 11 unhomed verbs**:

```
map · mapv · filter · foldl · reduce      W7 HOF family (parked on effectful_by_prefix)
= · not= · and · or · not · bool::to-string   remaining P6-c dispatch population
```

It stops being a hand-list and becomes a **named homing backlog**: every row names a verb whose
ruling cannot move home until the verb has a home. `38 → 11`, and the 11 have owners.

## Out of scope = REJECTED

- **The derivation itself.** T4b, drawn next.
- **Any verb not in the 27.** The other ~404 keep `@Totality Unreviewed` honestly.
- **Re-verifying the 27's totality from scratch.** The rulings were made by reading the
  implementations; this stone moves them. A rider who *disagrees* with one reports it as a finding
  rather than silently transcribing — see the brief's STOP-2.

## Calibration

Predicted 35–55 min. Eight files, 27 directives, 27 reasoning paragraphs to place and attribute.
