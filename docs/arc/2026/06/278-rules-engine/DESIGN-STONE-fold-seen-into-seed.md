# DESIGN-STONE — fill `seen` on the seed walk

> **Origin (2026-08-20).** 20: in-fire `setup:seen` **3.99** is
> insert. Isolated `identity()` walk **1.73** on real facts.
> Alpha seed already chases those same Arcs for class extract.
> Two walks. This stone is one. HashSet insert still happens.
> Session stays a PersistentVector.

## The measurement we have

`setup:seen:insert` walks `input_facts` and `seen_insert`s.
`alpha:seed` walks `input_facts` again and activates.
20 named the duplicate chase. Folding does **not** skip
inputs and does **not** skip the HashSet insert.

## The algorithm

Alloc the two HashSets at SETUP (0.01). Seed round:

```
for fact in input_facts {
    seen_insert(fact)
    alpha_activate_fact(fact)
}
```

Delete the standalone insert loop. Delta rounds unchanged
(derived facts already enter `seen` at production).
`setup:seen:insert` mark retired (would be 40k pairs or a
lie wrapping activate). Outer `setup:seen` is alloc.

Every input fact is in `seen` before production. Token
stays two spans. Frozen Session stays a PersistentVector.

## ★ THE ONE CONTRACT DECISION

**`seen` still contains every input fact before any derived
fact is considered.** Production is after seed alpha.
Folding the insert into the seed walk does not skip a fact.

## The gate

1. `accum_leftover_split` `[200 200]`: `setup:seen` **< 0.5
   ms**. Seed and FIRE printed. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): `setup:seen` 3.99 →
**~0.01**. Seed absorbs HashSet insert and drops the
duplicate chase: FIRE 19.78 → **~18**. Isolated S still
2.95 if measured alone.

## Blast radius

`kernel.rs` `fire_fixpoint_delta` seed loop. Census
REQUIRED drops `setup:seen:insert`. No `.wat`. No
Session field.

## Out of scope = REJECTED

- Session-`Vec`. Skip seen inputs. Second hasher. Intern
  `names`. 2e / 2o. 297. Insertion. Per-fact timers.
- Fold seen into delta. Scratch repr.

## Sequencing

1. Fold. Weigh seen / seed / FIRE. Stop.

## Weigh (2026-08-20) — LANDED

`accum_leftover_split` / `accum_alpha_leftover_split`,
mean of 3. Gate: rete lib 97, clippy `-D warnings` silent.

| lump | before | after |
|---|---:|---:|
| setup:seen | 3.99 | **0.01** |
| seed | 11.68 | **16.01** |
| FIRE | 19.78 | **19.57** |
| honest_FIRE | 20.19 | **19.15** |

Predicted FIRE ~18. **Missed.** HashSet insert moved into
seed (+4.3). Duplicate chase did not cut FIRE by 1.7 —
wash. One walk. `seen` still holds every input before
production. Keep: FIRE did not rise; the extra walk is
gone. Isolated A 10.88 vs seed 16.01 is the insert now
living in seed.

Next leftover: isolated M−T intern/ops **4.92** (scratch
1.67 STOP). Do not intern names. Do not start 297.
