# DESIGN-STONE — packed seed d_alpha is a range, not 0..n

> **Origin (2026-08-23).** `from_one` LANDED. Wrap of 40k
> maps is physics. Occupancy already packed n facts of a
> class. Seed then writes `Vec<usize> = (0..n).collect()`
> and clones it per alpha. Root-join walks occupants —
> that walk is WHAT. The list is occupancy theater.

## The enemy

```
els = Arc<Vec<Element>> from packed fact ids
slots = (0..ids.len()).collect()
for aid in leaves:
    alpha[aid] = els.clone()
    d_alpha[aid] = slots.clone()   // "all of them are new"
```

`els.len()` is n. Occupancy answered which facts sit
in the leaf. Seed then reconstructs 0,1,2,…,n−1 to
say the same thing, once per alpha that shares the
class.

Unpacked activate still pushes live slots. Delta
rounds still push. Only packed seed is the range.

## The algorithm

```
packed_full: HashSet<aid>     // fire-scoped, cleared each round
seed packed class:
    alpha[aid] = els.clone()
    packed_full.insert(aid)   // no d_alpha 0..n
consumers:
    if packed_full.contains(aid): walk 0..alpha[aid].len()
    else: walk d_alpha[aid]
```

Same occupants, same order (visit order of the
column). Dual-impl WHAT unchanged. Do not skip
the walk. Do not Session-Vec.

## ★ THE ONE CONTRACT DECISION

**Packed seed dirty is a range over occupancy
already written.** `d_alpha` stays indices for
activate and later rounds. Empty `d_alpha` is
still “no news” unless the aid is in
`packed_full`.

## The gate

1. `accum_alpha_leftover_split` / `fanout_three_leftover_split`
   Instant seed or harvest does not regress vs the
   print before this intern. Any drop counts.
2. 7strat 3/3 including three-stratum.
3. rete lib. clippy `--lib -D warnings`.

## Predicted win

Independent guess (written first): accum seed
drops the 40k-`usize` write and the clone per
Reading alpha. Fanout Left/Right packed the same
way. Wrap W stays physics.

## Blast radius

`fire/delta.rs` packed seed fill + root-join /
hash-join `d_alpha` readers.
`fire/mod.rs` `append_d_alpha`.
`arm.rs` `seed_dirty_join_parents`.
No `.wat`. No Session field. No `AlphaDelta` type
change.

## Out of scope = REJECTED

- Session-Vec. Skip freeze. intern `names`. Array1. 297.
- `Arc<[Element]>` occupancy (delta `make_mut` push).
- Skip packing Readings. Skip the root-join walk.

## Sequencing

1. Range instead of 0..n. Weigh. Stop.
2. Revert if 7strat red or leftover Instant regresses
   without a named drop.

## Weigh (2026-08-23) — LANDED

7strat 3/3 including three-stratum. `class_scan_harvest_includes_input` 2 T. Clippy `--lib -D warnings` silent.

Same-session leftover (mean of 3), before intern then after:

| | before | after |
|---|---:|---:|
| accum seed | 10.39 | **10.33** |
| accum FIRE | 13.67 | 14.13 |
| fanout without FIRE | 25.04 | **24.88** |
| fanout with FIRE | 31.12 | **30.29** |
| harvest:query | 6.01 | 6.16 |

Seed Instant is pack physics; the 0..n list was a thin slice. Fanout FIRE **−0.83**. Occupancy no longer reconstructs indices it already wrote. Dual-impl WHAT unchanged. Do not Session-Vec. Do not skip the walk.
