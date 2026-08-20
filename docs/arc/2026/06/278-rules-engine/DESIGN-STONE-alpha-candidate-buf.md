# DESIGN-STONE — reuse the alpha candidate buffer

> **Origin (2026-08-20).** 18: accum leads honest FIRE **20.73**.
> Alpha **12.74**. 10 ranked tree walk: T−I alloc **0.82 < 1**,
> no reused buffer. `candidates_into` exists; fire still calls
> `candidates()` (new Vec per fact). At this tip T−I is
> **2.60**. Stone 10's intern is now drawable.

## The measurement we have

`accum_alpha_tree_walk_split` today:

| lump | ms |
|---|---:|
| T−I Vec alloc | **2.60** |
| I−G walk | 0.76 |
| G−E class | 0.68 |

`alpha_activate_fact` still does `candidates()` → `Vec::new`
× 40,200. Isolated A 11.39 vs seed 12.83.

## The algorithm

`alpha_activate_fact` takes `&mut Vec<i64>`. Fills it with
`candidates_into`. One buffer per fire (same scope as
`match_scratch`). `alpha_pass` the same. Isolated T/I/M
arms unchanged so T−I still names the alloc.

Over-approx contract unchanged. Token stays two spans.
Do not populate `range_children`.

## ★ THE ONE CONTRACT DECISION

**The tree still over-approximates.** Reusing the buffer
does not change the candidate set. Fire no longer allocates
a `Vec<i64>` per fact.

## The gate

1. `accum_alpha_tree_walk_split` still prints T−I.
   `accum_alpha_leftover_split` prints A and seed.
   Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): isolated A falls by
**~2.6 ms**. Seed 12.83 → **~10.2**. FIRE 21.08 → **~18.5**.
`setup:seen` untouched.

## Blast radius

`kernel.rs` `alpha_activate_fact` + `alpha_pass` + fire
loop. Isolated A call sites. No `.wat`. No tree shape.

## Out of scope = REJECTED

- Range edges. Intern `names`. `setup:seen`. 2e / 2o.
- 297. Insertion. Per-fact timers. Scratch repr.

## Sequencing

1. Buffer. Weigh A / seed / FIRE. Stop.

## Weigh (2026-08-20) — LANDED

`accum_alpha_leftover_split` / `accum_leftover_split`,
mean of 3. Gate: rete lib 96, clippy `-D warnings` silent.

| lump | before | after |
|---|---:|---:|
| seed | 12.83 | **11.68** |
| isolated A | 11.39 | **10.15** |
| A−M (push lump) | 1.77 | **0.67** |
| honest_alpha | 12.71 | **11.82** |
| FIRE | 21.15 | **19.78** |
| setup:seen | 4.04 | 4.01 |

Predicted −2.6 in-fire; measured **−1.15** on seed
(isolated A −1.24). The alloc was sitting in A−M, not
in T−C. T−I still prints (~1.94) because isolated T
calls `candidates()`. `candidates` is `#[cfg(test)]`.
Over-approx unchanged. `setup:seen` untouched.

Next leftover on this cell: **setup:seen ~4.0** (2z fire
context; needs a new split). Do not intern names. Do not
start 297.
