# DESIGN-STONE — split in-fire `setup:seen` (alloc vs insert)

> **Origin (2026-08-20).** 19: accum FIRE **19.78**. Largest
> named leftover: `setup:seen` **4.01**. 2z: isolated P **1.67**,
> in-fire **4.30**. Session-`Vec` refused (P−V 0.38; D+V worse).
> We do not intern fire context without naming it. This stone
> prints the split. It does not intern off an unranked lump.

## The measurement we do not have

The mark is one pair around HashSet `with_capacity` + PV
iter + `seen_insert`. Isolated 2z used *synthetic* Records,
not the seeded Session after `to_transient`. Alloc vs insert
unsplit. In-fire vs isolated-on-real-facts unsplit.

## The algorithm

In-fire, **two** extra pairs (not per fact):

```
setup:seen:alloc   — the two HashSets with_capacity
setup:seen:insert  — input_facts.iter + seen_insert
```

Outer `setup:seen` stays. Tax: 2 × cal.

Isolated, after compile+seed `[200 200]` (un-timed). Mean of 3.
Same facts the engine walks:

```
A  HashSet with_capacity(n) only
X  identity() walk, no insert
S  seen_insert loop            // engine
```

Deltas: `A` alloc, `X` extract, `S−A` insert. Compare `S`
to in-fire insert. Compare in-fire seen to `S`.

1. **STOP intern** if no lump is ≥ 1 ms **and** drawable
   (not Session-`Vec` / skip-seen-inputs / second hasher /
   fold-into-seed).
2. If alloc ≥ 1: say so; do not intern a second intern table.
3. If in-fire insert − S ≥ 1: leftover is fire context.
   Name it. Do not intern a Session `Vec`.
4. Do not intern `names`. Token stays two spans.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine**
beyond empty-in-release marks. `seen` still contains every
input fact before any derived fact is considered.

## The gate

1. `accum_seen_fire_context_split` prints in-fire alloc /
   insert and isolated A / X / S. Insert > 0. Do not
   wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **insert owns ~3.5 of
4.0**. Alloc tiny. Isolated S tracks 2z's 1.67. In-fire
insert − S is the fire-context gap (~2 ms). Not internable
without folding seen into another walk (refused). Say so.

## Blast radius

`kernel.rs` two coarse marks + one test. No `.wat`. No
Session field.

## Out of scope = REJECTED

- Session-`Vec`. Skip seen inputs. Second hasher. Fold
  seen into seed. Intern `names`. 2e / 2o. 297. Insertion.
- Per-fact timers.

## Sequencing

1. Marks. Test. Print. Rank.
2. No drawable ≥ 1 → stop.
3. Else name the intern. Do not intern this stone.

## Weigh (2026-08-20) — LANDED, no intern

`accum_seen_fire_context_split` `[200 200]`, 40,200 facts,
mean of 3. Gate: rete lib 97, clippy `-D warnings` silent.

| lump | ms |
|---|---:|
| in-fire setup:seen | **3.99** |
| alloc | 0.01 |
| insert | **3.98** |
| isolated A alloc | 0.01 |
| isolated X `identity()` walk | **1.73** |
| isolated S seen_insert | **2.95** |
| S−A | 2.94 |
| in-fire insert − S | **1.03** |

Prediction held: insert owns the mark; alloc is dead.
Isolated S on *real* seeded facts is 2.95, not 2z's 1.67
(synthetic). Of that, `identity()` walk is **1.73** (Arc
chase to a u64 field). HashSet insert ≈ 1.2. Fire context
**1.03**.

No drawable intern: Session-`Vec` still refused; skip-seen
refused; second hasher refused; fold-into-seed would save
the *duplicate* chase, not the HashSet insert. Named, not
taken. `seen` still fills every input fact.

Next leftover: accum M−T intern/ops ~4.8 (scratch 1.67
STOP) / fold-into-seed if named. Do not start 297.
