# DESIGN-STONE — split harvest wrap (clones vs Arc vs intern-id)

> **Origin (2026-08-23).** Array slice intern LANDED.
> Isolated wrap **8.78**. In-fire harvest:query **6.06**.
> This stone unfuses wrap. Do not intern until a part
> is ≥ 1 ms. Do not add a third PMap arm.

## The enemy

One-entry `from_pairs` is still, per map:

```
scan.var.clone()
f.clone()
Arc::from([(k, v)])
next_intern()
```

40k of those is 6 ms in-fire. The Arc and the intern
id and the clones are fused. This stone prints the
parts.

Query-memory stays name → vector of binding maps.
`PMap` stays Array | Trie. Session stays a
PersistentVector. `rust_identity` stays clone-stable
and ignored by Eq/Hash.

## The algorithm

Tight loop. 40k `fan::Pair` facts. Mean of 3.
Unscaled. Scan paid outside. No 40k fire marks.

```
C  clone (var, fact) × 40k into Vec<(Value, Value)>
R  Arc::from([pair]) × 40k from pre-cloned pairs
I  AtomicU64::fetch_add × 40k           // next_intern
W  from_pairs([(var, fact)]) × 40k      // authority
```

Treat **C** as the clones. Treat **R** as the Array
allocation. Treat **I** as the intern stamp.
**W − C** is wrap minus clones.

1. Largest part **< 1 ms**: stop. Remaining wrap is
   physics (40k maps is WHAT).
2. **I ≥ 1 ms**: Array `rust_identity` is the Arc
   pointer; drop `next_intern` on the one-entry path.
   Overlay HIT keys the *network* map, not query rows.
3. **R ≥ 1 ms**: Array of one pair is stored inline
   (`Array` still the arm — not `PMap::Array1`).
4. **C ≥ 1 ms**: intern the clones (var is already
   an Arc; fact is already an Aggregate Arc).
5. Array1 as a sibling of Array and Trie: REJECTED.

## ★ THE ONE CONTRACT DECISION

**This stone prints the parts.** Dual-impl WHAT is
unchanged. The only intern it may take is cheaper
construction of the **existing** Array arm. Eq/Hash
compare entries. Overlay HIT still uses
`rust_identity` of the network map.

## The gate

1. `harvest_wrap_parts` prints C / R / I / W. W > 0.
2. If the stone implements: `fanout_three_leftover_split`
   still 40k maps. harvest:query drops ≥ 1 ms vs 6.06.
3. rete lib. One-entry `from_pairs` equals `assoc`.
   A map used as a key is found across arms.
4. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): **R owns ~6 ms.**
C is Arc bumps, < 2 ms. I is Relaxed fetch_add, < 1 ms.
If R holds: intern inline one-pair Array. Predicted
harvest:query **6.06 → ~2**.

## Blast radius

`kernel/tests.rs` only unless a part is ≥ 1 ms.
Then `pmap.rs` Array construction. No `.wat`.
No Session field. No QueryMemory type change.

## Out of scope = REJECTED

- Native `Vec` in the frozen Session. Skip freeze.
- `PMap::Array1` as a sibling of Array | Trie. 297.
- Intern `names`. Fuse harvest into freeze to move
  the mark.
- Shared intern-id `0` on every one-entry map in
  the runtime (overlay identity is per instance).

## Sequencing

1. Print. Rank.
2. Largest part < 1 ms → stop.
3. Else the one intern named above. Weigh harvest:query. Stop.

## Weigh (2026-08-23) — LANDED print; intern reverted

`harvest_wrap_parts` (40k, mean of 3):

| lump | ms |
|---|---:|
| C clone (var, fact) into Vec | 3.66 |
| R `Arc::from([pair])` | 3.03 |
| **I fetch_add** | **0.20** |
| W from_pairs | 6.08 |
| W−C | 2.42 |

I is not the intern. R is malloc of 40k map bodies. `Value` contains `PMap` contains `Value` — a one-pair Array **cannot sit inline** (infinite size). `ArrayBody::One(Box<(k,v)>)` compiled and passed PMap tests; in-fire harvest:query **6.07 vs 6.06**. Box is not ≥ 1 ms cheaper than Arc. Reverted.

Remaining wrap is physics: 40k heap maps is WHAT. Do not Session-Vec. Do not skip freeze. Do not `PMap::Array1`. Clippy `--lib -D warnings` silent.

## Weigh (2026-08-23) — live print after identity-filter; from_one LANDED

Identity-filter intern LANDED (`fae5b3e5`). In-fire harvest:query **6.37**. Isolated wrap-parts, mean of 3:

| lump | ms |
|---|---:|
| **C clone (var, fact)** | **3.92** |
| **R `Arc::from([pair])`** | **3.40** |
| I fetch_add | 0.20 |
| W from_pairs | 10.30 |
| W−C | 6.38 |

I is not the intern. C is Arc bumps of `String`/`Aggregate` — already the intern the stone named. R intern (inline one-pair) is infinite-size; Box already washed.

Intern: `PMap::from_one` on the existing Array arm. Harvest calls it. `from_pairs` of one pair delegates. Skip the iterator dance. Not Array1. Not Box. Eq/Hash still compare entries.

In-fire harvest:query **6.37 → 5.97 (−0.40)**. Under the ~0.5 ms gate. Builder took it: chase everything that Instant-proves, including sub-gate construction on the existing arm. Isolated W 10.30 → 10.03.

Remaining wrap is still physics of 40k heap maps. Do not Session-Vec. Do not skip freeze. Do not `PMap::Array1`. Do not drop `next_intern` on one-entry maps.
