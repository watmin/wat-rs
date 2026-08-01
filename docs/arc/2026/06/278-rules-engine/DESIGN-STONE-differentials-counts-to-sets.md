# DESIGN-STONE — the internal differentials compare COUNTS; promote them to SETS

> **Origin (2026-07-31):** asked whether we need "session-shape differentials," the answer was no —
> a whole-Session comparison would forbid the shape divergence R22 explicitly licenses (the kernel
> may differ in SHAPE while matching in RESULT), and would have gone red on this morning's alpha
> stone while being *correct to*. But the question was circling a real asymmetry next to it.

## The finding — our external check is stricter than our internal one

```
the grid's :accuracy :match   compares the canonically-encoded SORTED DERIVED SET   (21/21 points)
our 8 rete differentials      compare a single i64 COUNT                            (per case)
```

Every rete oracle-vs-native probe exposes `-> :wat::core::i64` entries and asserts
`native == oracle` on the number (`probe_arc278_7exists_native_differential.rs:28`, and the same
shape in all eight). **A count cannot catch "right number of facts, wrong facts."**

That is not a hypothetical class. A binding threaded to the wrong variable, a join keyed on the wrong
tuple, an accumulator folding the right cardinality of the wrong group — each derives the correct
*quantity* with different *contents*, and every one of those passes a count differential.

We already know how to do better, in this repo, today: `gen-accum.sh` encodes each derived fact
injectively into an i64 (`kind*1e15 + g*1e9 + val`), sorts ascending, and compares the vectors
byte-for-byte. **The technique is proven and in production on the axis that matters most.** The
internal differentials simply never adopted it.

## ★ THE ONE CONTRACT DECISION

**The encoding must be INJECTIVE, and its injectivity must be argued in the probe's own header.**

A canonical encoding that can collide turns a set comparison back into something *weaker* than it
looks — two different fact sets mapping to the same sorted vector, passing, while presenting as a
strictly stronger check than the count it replaced. That is the worst outcome available here: a gate
that reads as an upgrade and is a downgrade.

Concretely, for each probe: state the field ranges, show that the multipliers exceed them (as
`gen-accum.sh` does — `g*1e9` is safe because `g < 1e6` at every size the axis runs), and pick
multipliers from the axis's own bounds rather than copying accum's constants.

**If a probe's facts cannot be injectively encoded into an i64** (too many fields, unbounded strings),
say so and compare the fact vectors structurally instead — do NOT widen a lossy encoding until it
"fits."

## The non-vacuity requirement — do not lose the anchor in the migration

Today each case asserts twice:

```rust
assert_eq!(native, oracle, "native==oracle");   // the differential
assert_eq!(native, 1, "≥1 reading → 1");        // the ANCHOR — a literal expected value
```

The second line is what stops the pair from passing vacuously when both engines derive nothing.
**Both assertions must survive**: set-equality between the engines, AND an expected set (or at minimum
an expected length) against a literal. A migration that keeps only the first is a regression wearing
an upgrade's clothes — the exact shape this arc keeps finding.

## Scope — 8 probes, tests only

`tests/rete/probe_arc278_*_differential.{wat,rs}`:
`6b_ii_b_where` · `7b_negation` · `7exists` · `7strat` · `8b_accumulate` · `8custom` ·
`native_insert` · `insert_all`.

**No `src/`, no `wat/`.** Nothing in the engine moves — this strengthens the net, it does not touch
what the net is around.

⚠ **Sequencing:** `insert_all` is being written by another rider right now. This stone lands AFTER
that one and includes it; do not start while that tree is dirty.

## The gate

1. All 8 probes green with set comparison in place.
2. Each probe's header states its encoding and argues injectivity from the axis's own field bounds.
3. **Each case retains a literal expected value** (set or length) alongside the engine-vs-engine
   assertion.
4. **A deliberate break proves the upgrade is real:** in a scratch copy, perturb one derived fact's
   *content* while preserving the count, and confirm the set differential goes RED where the count
   differential would have stayed green. Report the before/after. Without this, the claim "sets are
   stronger than counts" is asserted rather than demonstrated (R59 `NISI FRANGAS, NIHIL PROBAS`).

## Out of scope = REJECTED (affirmative cuts)

- **Whole-Session / shape differentials.** They would forbid the divergence R22 licenses and would
  have blocked the alpha stone. The kernel is *required* to match on RESULT, not on shape.
- **A checked result/scratch contract.** Worth doing — today's ruling that `facts` +
  `production-memory` are the RESULT and alpha/beta are fire-scoped scratch currently lives only in a
  doc comment. But it is a *different* stone with a different gate; folding it in here would blur
  what this diff proves.
- **The sqlite store differential.** Same technique, different subject (mem vs sqlite, not oracle vs
  native). Separate.
- **retract / streaming.** Absent axes, not weak ones. No amount of set-comparison creates a
  differential for a code path nothing exercises.
