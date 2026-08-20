# DESIGN-STONE — split honest alpha without per-fact timers

> **Origin (2026-08-19).** 6 retired the candidates trap.
> Accum `[200 200]` FIRE **26.53**. Outer `alpha` **18.16**
> (68%). `setup:seen` 3.92. `accum:index` 2.03. Child-net
> ranking had *zeroed* the real tree/`exec_compiled`/push
> work into remainder. We do not know the shares. Guessing
> is how this arc interned the wrong row. This stone prints
> the split. It does not intern.

## The measurement we do not have

18 ms is one mark around both rounds. Seed is 40,200 facts.
Delta is ~1,000 derived. Inside one fact: class extract,
`alpha_tree.candidates`, `exec_compiled` (ops + intern),
Copy-`Element` push. 6 forbade 281,800 child timers. A
rank that puts them back is the trap returning.

## The algorithm

In-fire, **two** extra pairs (not per fact):

```
alpha:seed   — first worklist (facts PV)
alpha:delta  — later owned_delta
```

Outer `alpha` stays. Tax: 2 × cal ≈ 0.2 µs.

Isolated, after compile+seed once (un-timed). Mean of 3.
40,200 real accum facts. Reset bind pools each run (cold
intern, same as first fire):

```
Wp  PersistentVector iter
W   Vec iter
C   class extract
T   candidates
M   exec_compiled per candidate (no push)
A   alpha_activate_fact (control)
```

Deltas: `C−W`, `T−C`, `M−T`, `A−M`. `S+D` vs in-fire
alpha. `A` vs `S`.

Drawable only if a lump is ≥ 1 ms **and** not 2o-dead /
names / stamp / Session-`Vec` / persist-gather.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the
engine** (marks are `phase_start`, empty in release).
Do not restore per-fact alpha timers. Do not intern
off this rank until a named leftover is ≥ 1 ms.

## The gate

1. `accum_alpha_leftover_split` prints S / D / Wp / W /
   C / T / M / A. Seed > 0. A > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **seed owns ~17 ms,
delta ~1 ms.** Isolated `M−T` (`exec_compiled` + intern)
is the largest piece. Tree `T−C` small. Push `A−M` tiny
(Copy). If `A` sits well below in-fire seed, leftover is
fire context (2z's seen lesson) — say so; do not intern.

## Blast radius

`kernel.rs` two coarse marks + one test. No `.wat`. No
crate. Token stays two spans.

## Out of scope = REJECTED

- Per-fact `alpha:*` timers. `census_count` as a timer.
- Intern `names`. Facts in `bind_pool`. 2e / 2o. 297.
- Fact insertion. Session-`Vec`. Fold `setup:seen`.

## Sequencing

1. Two marks. Isolated stacked loops. Print. Rank. Stop.
2. Do not intern this stone.

## Weigh (2026-08-19) — LANDED, no intern

`accum_alpha_leftover_split` `[200 200]`, mean of 3.

In-fire:

| lump | ms | pairs |
|---|---:|---:|
| FIRE | 26.10 | |
| alpha | 18.00 | 2 |
| **seed** | **17.97** | 1 |
| delta | 0.04 | 1 |

Isolated, 40,200 facts, cold intern:

| lump | ms |
|---|---:|
| Wp PV iter | 0.44 |
| W Vec iter | 0.01 |
| C−W extract | 0.96 |
| **T−C tree** | **4.46** |
| **M−T exec_compiled+intern** | **7.65** |
| **A−M push** | **3.45** |
| A control | 16.54 |
| A vs seed | 16.54 vs 17.97 |

Seed owns alpha. Delta exhausted. A tracks seed (fire
context ~1.4 ms, not 2z's gap). Prediction held on seed
and on `M−T` as the largest piece. Tree and push were
*not* small.

Drawable ≥ 1 ms, not 2o / names / stamp / Session-`Vec`:

1. `exec_compiled` + intern **7.65**
2. alpha-tree walk **4.46**
3. Element push / `d_alpha` **3.45**
4. `setup:seen` 3.92 still sits beside this split

Do not intern this stone. Next intern is `M−T` if named.
