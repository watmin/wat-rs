# DESIGN-STONE — leftover `setup:seen` is the facts-vector walk

> **Origin (2026-08-18).** 2x: `setup:seen` 7.43 → **4.31**.
> Isolated u64 insert is **1.20**. Gap ≈ **3 ms**. The mark is
> `input_facts.iter()` (a PersistentVector of 40,200) plus
> `seen_insert`. Session still holds a PersistentVector. Weigh
> the walk vs a transient `Vec` before drawing.

## The measurement we do not have

2x's `I` iterated a `Vec<Value>`. Fire iterates
`rpds::VectorSync`. Two walks pay: `setup:seen` and round-1
alpha. A transient `Vec` decoded once at `to_transient` would
make both Vec iters. Decode cost is unranked.

## The algorithm

Tight loop. 40,200 stamped Records in both a `Vec` and a
`VectorSync`. Mean of 3.

```
W  PersistentVector iter only
I  FxHashSet<u64> from a Vec<u64>      // 2x's I
V  Vec<Value> iter + seen_insert
P  PersistentVector iter + seen_insert // engine
D  PV collect into Vec
```

`P − V` is the walk. `D + V` vs `P` is a one-shot decode.
Round-1 alpha is a second walk; do not fold it into this mark.

1. **STOP** if `P − V` < **1 ms**. Leftover is the u64 insert.
   Do not change facts representation.
2. Else first worklist is a `Vec<Value>` filled once from the
   PV at `to_transient` (or the start of SETUP). Frozen Session
   still a PersistentVector. Token stays two BindSpans. Do not
   skip filling `seen`.

## ★ THE ONE CONTRACT DECISION

**`seen` still contains every input fact before any derived
fact is considered.** The transient worklist may be a `Vec`.
The frozen Session field is unchanged.

## The gate

1. `seen_pv_walk_split` prints W / I / V / P / D. P > 0.
2. If the stone implements: `setup:seen` printed at
   `accum_leftover_split` `[200 200]`. Do not wall-gate FIRE.
   Token still two `BindSpan`s.
3. rete lib.
4. clippy `-D warnings`.

## Predicted win

Independent guess (written first): **P − V ≈ 2–3 ms** (the
tree walk). D + V ≈ P, so decode-for-seen-alone does not win.
A `Vec` shared with round-1 alpha might, but that is two
marks — say so; do not intern seen-only if D + V ≥ P.

## Blast radius

`kernel.rs` tests; `WorkingMemory` / `to_transient` only if
step 2. No `.wat`. No Session field change.

## Out of scope = REJECTED

- Native `Vec` in the frozen Session. Persist gather. 297.
- Second hasher. Skip `seen` inputs. Intern `names`. 2o.

## Sequencing

1. Print. Rank.
2. P − V < 1 ms → stop.
3. Else only if D + V < P by ≥ 1 ms: transient Vec. Weigh
   `setup:seen`. Stop.

## Weigh (2026-08-18) — LANDED, no intern

`seen_pv_walk_split` (40,200 stamped Records, mean of 3):

| lump | ms |
|---|---:|
| W PV iter only | 0.34 |
| I u64-set from `Vec<u64>` | 0.37 |
| V Vec iter + insert | 1.29 |
| P PV iter + insert (engine) | 1.67 |
| D PV collect into Vec | 3.12 |
| **P−V walk** | **0.38** |
| D+V | 4.41 |
| (D+V)−P | **+2.74** (worse) |

P − V < 1. Decode-then-Vec loses. Isolated P is **1.67** vs
in-fire **4.30** — the leftover is fire context, not the tree
walk. Frozen Session stays a PersistentVector. Do not intern.

Next named engine row: **drop-memories 3.63**.
