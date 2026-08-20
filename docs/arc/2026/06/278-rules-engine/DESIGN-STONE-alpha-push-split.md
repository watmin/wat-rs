# DESIGN-STONE — split Element push / `d_alpha`

> **Origin (2026-08-19).** After 11, leftovers ≥ 1 ms:
> `setup:seen` ~3.9 (2z: fire context; Session-`Vec`
> refused; fold-into-seed is one PV walk ≈ 0.34 isolated),
> Element push **3.45**, `intern_val` 2.77. This stone is
> the internable 3.45. Do not intern `seen`.

## The measurement we do not have

On success (80,200 times):

```
make_element            // Copy
wm.alpha.entry(aid).or_default().push(el)
d_alpha.entry(aid).or_default().push(slot)
```

`wm.alpha` is `HashMap<i64, Vec<Element>>` (SipHash).
A−M **3.45** is unsplit. Guessing HashMap vs Vec growth
is the 7 lesson.

## The algorithm

Same fixture as 7–11. Mean of 3. Cold intern tables.
Stacked after `exec_compiled` success:

```
M   candidates + exec_compiled
H   M + alpha.entry.or_default()     // no push
V   H + Vec<Element>::push
D   V + d_alpha.entry.push
A   alpha_activate_fact              // control
```

Deltas: `H−M` HashMap entry, `V−H` Vec push, `D−V`
`d_alpha`, `A−D` leftover.

1. **STOP intern** if no lump ≥ 1 ms.
2. Else intern **only the largest** lump: FxHashMap for
   `i64` maps if entry wins; `reserve` if Vec wins. Do
   not change `beta` / `production`. Do not intern `seen`.
   Token stays two spans.

## ★ THE ONE CONTRACT DECISION

**Memories stay maps of node-id → Vec.** We may change
the hasher. We do not put facts in `bind_pool`. We do
not intern `setup:seen`.

## The gate

1. `accum_alpha_push_split` prints M/H/V/D/A and deltas.
   D > 0. Do not wall-gate FIRE.
2. If intern: rete lib. Isolated A−M printed.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`H−M` (SipHash
`entry`) owns ≥ 1 ms.** Vec push smaller (Copy Element,
amortized growth). If HashMap ≥ 1, intern FxHashMap for
`wm.alpha` + `d_alpha`. If not, say so; do not intern.

## Blast radius

`kernel.rs` test; `WorkingMemory.alpha` / `d_alpha` type
only if intern. No `.wat`. No `seen`.

## Out of scope = REJECTED

- Session-`Vec` for facts. Fold `seen` into seed. Intern
  `names`. Facts in `bind_pool`. 2e / 2o. 297. Insertion.
- Per-fact timers. `beta`/`production` hasher. Alpha-tree
  `range_children`.

## Sequencing

1. Print M/H/V/D/A. Rank.
2. No lump ≥ 1 → stop.
3. Else intern the largest. Weigh. Stop.

## Weigh (2026-08-19) — LANDED, FxHashMap intern

Before intern (mean of 3):

| lump | ms |
|---|---:|
| H−M HashMap entry | **1.38** |
| V−H Vec push | 0.12 |
| D−V d_alpha | 0.61 |
| A−D leftover | 1.70 |
| A−M push lump | 3.81 |

Prediction held. Interned `wm.alpha` + `d_alpha` to
`FxHashMap`. `beta` / `production` untouched. `seen`
untouched.

After intern:

| lump | ms |
|---|---:|
| H−M entry | **−0.68** (gone) |
| A−M | **2.30** (was 3.81) |

Did not intern Vec push. Next leftovers: `setup:seen`
~3.9, `intern_val` 2.77, ops 1.90.
