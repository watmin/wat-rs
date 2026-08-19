# DESIGN-STONE — `seen` of stamped facts is the fingerprint

> **Origin (2026-08-18).** 2w: accum `[200 200]` honest_FIRE
> **20.97**. Largest engine row: `setup:seen` **7.43**. 2g
> stamped shallow Aggregates and Hash writes the u64. Leftover
> is HashSet insert of 40k `Value`s, not the walk. 2g: do not
> add a second hasher. Weigh an identity-set before drawing.

## The measurement we do not have

`seen` is `FxHashSet<Value>`. Hash of a stamped Aggregate is
already the fingerprint. Eq still walks `nature+class+fields`.
Insert clones the `Value` (Arc bump) into a fat slot. 2g
refused a second hasher. It did not measure `FxHashSet<u64>`
of the stamp the constructor already paid.

## The algorithm

1. Tight loop, 40,200 stamped Pair-shaped Records (the accum
   input count). Mean of 3.

```
C  clone 40,200 Values                 // Arc bump
S  FxHashSet<Value> insert (engine)
I  FxHashSet<u64> insert (identity)
```

`(S − I) × 1` is the predicted cut at `[200 200]`.

2. **STOP** if that cut is **< 1 ms**. Leftover is hashbrown
   of 40k slots either way. Do not touch `seen`.

3. Else `seen` of stamped Aggregates is `FxHashSet<u64>`.
   `identity == 0` stays a `FxHashSet<Value>` side table
   (Session / nested). No second hasher. Token stays two
   BindSpans. Do not skip input facts.

Fingerprint collision (unequal data, same u64) treats the
second as seen. Hash already buckets by that u64; Eq was
the only split. 2^-64. Differentials stay the net.

## ★ THE ONE CONTRACT DECISION

**A stamped fact's membership in `seen` is its construction
fingerprint.** Eq of `Value` is unchanged. We do not hash
`Arc` pointers. We do not add a hasher.

## The gate

1. `seen_identity_set_split` prints C / S / I. S > 0.
2. If the stone implements: `setup:seen` printed at
   `accum_fire_phase_census` `[200 200]`. Do not wall-gate
   FIRE. Token still two `BindSpan`s.
3. rete lib.
4. clippy `-D warnings`.

## Predicted win

Independent guess (written first): S sits near 7 ms. I ~1 ms.
Cut **~5–6 ms**. FIRE 63.83 → **~58**. If S−I < 1, leftover
is the insert itself — say so.

## Blast radius

`value.rs` (`identity()` getter). `kernel.rs` `seen` + split
test. Production / setup insert sites. No `.wat`.

## Out of scope = REJECTED

- Second hasher. Pointer-hash. Skip `seen` inputs.
- Persist gather. 297. Intern `names`. 2o.

## Sequencing

1. Print the split. Rank.
2. Cut < 1 ms → stop.
3. Else identity-set. Weigh `setup:seen`. Stop.

## Weigh (2026-08-18) — LANDED

`seen_identity_set_split` (40,200 stamped Records, mean of 3):

| lump | ms |
|---|---:|
| C clone | 1.94 |
| S `FxHashSet<Value>` | 3.48 |
| I `FxHashSet<u64>` | 1.20 |
| **S−I** | **2.28** |

Cut ≥ 1. Intern licensed. `seen` of stamped Aggregates is
`FxHashSet<u64>`. `identity == 0` stays a `Value` side table.
No second hasher.

`accum_leftover_split` `[200 200]`:

| mark | before | after |
|---|---:|---:|
| `setup:seen` | 7.43 | **4.31** |
| FIRE | 63.83 | **61.00** |
| honest_FIRE | 20.97 | **14.50** |

`setup:seen` **7.43 → 4.31** (−3.12). FIRE **63.83 → 61.00**.
Token stayed two BindSpans. Leftover `setup:seen` 4.31 is the
PersistentVector walk + insert of 40k u64s. Next named engine
row: **accum:index 5.16**. Do not persist gather (cold fire).
