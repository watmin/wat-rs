# DESIGN-STONE — split `exec_compiled` (ops vs intern)

> **Origin (2026-08-19).** 7 ranked honest alpha: seed
> **17.97**, isolated `M−T` (`exec_compiled`+intern)
> **7.65**, tree 4.46, push 3.45. `M−T` is unsplit. Bind
> clones a field into scratch every call; intern runs on
> success only. Guessing which owns 7.65 is how this arc
> interned the wrong row. This stone prints the split. It
> does not intern.

## The measurement we do not have

`exec_compiled` is `scratch clear/resize` + `exec_ops` +
`materialize_into` (`intern_key` linear, `intern_val`
HashMap, optional `fact.clone()`). 7 timed them as one
lump. Cold intern (reset pools each run) was the 7
protocol — first-fire, not warm.

No per-fact timers. Isolated stacked loops on the same
40,200 accum facts.

## The algorithm

Same fixture as 7 (compile+seed once, un-timed). Mean of 3.
Cold intern each `M` run. `exec_ops` is `pub(crate)` so
the test can call it; fire-path unchanged.

```
T   candidates                         // 7's T
O   T + scratch reset + exec_ops       // no intern
Mc  T + exec_compiled, intern reset    // 7's M (cold)
Mw  intern tables kept; bind_pool
    cleared each run                   // intern hits, engine-sized append
```

Deltas: `O−T` ops. `Mc−O` intern/materialize (cold).
`Mc−Mw` intern-cold tax. Print fact_bind count and
ops-true vs candidates (once, un-timed).

Drawable only if a lump is ≥ 1 ms **and** not 2o / names
/ stamp / Session-`Vec`.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the
engine** besides `exec_ops: pub(crate)`. Do not restore
per-fact alpha timers. Do not intern off this rank.

## The gate

1. `accum_compiled_match_split` prints T / O / Mc / Mw
   and the deltas. O > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`O−T` (ops) owns
most of 7.65** — Bind clones + scratch, 80k times.
`intern_val` of ~400 unique i64s is < 1 ms (2x's I was
0.37 for 40k u64s). `Mc−Mw` small. If `fact_bind` is
set, intern of `fact.clone()` can steal the row — say
so. Do not intern.

## Blast radius

`compiled_cond.rs` (`exec_ops` vis). One kernel test.
No `.wat`. Token stays two spans.

## Out of scope = REJECTED

- Per-fact timers. Intern this stone. Fold `setup:seen`.
- Intern `names`. Facts in `bind_pool`. 2e / 2o. 297.
- Fact insertion. Session-`Vec`. Tree intern. Push intern.

## Sequencing

1. `exec_ops` pub(crate). Isolated T/O/Mc/Mw. Print.
2. Rank. Stop. Do not intern.

## Weigh (2026-08-19) — LANDED, no intern

`accum_compiled_match_split`, 40,200 facts, mean of 3.
`fact_bind` **0**. candidates **80,200**. ops-true **80,200**
(tree exact on this cell).

| lump | ms |
|---|---:|
| T candidates | 5.29 |
| O + exec_ops | 7.18 |
| Mc cold intern | 13.36 |
| Mw warm intern | 13.15 |
| **O−T ops** | **1.90** |
| **Mc−O intern/materialize** | **6.18** |
| Mc−Mw intern-cold tax | **0.22** |

Prediction failed: ops does **not** own 7.65. Intern
cold-insert of ~400 i64s is 0.22 — not the row. The
6.18 is **every success**: `intern_val` HashMap get of
`Value` + scratch clone + `pool.push`. ~77 ns × 80,200.

Drawable ≥ 1 ms: intern/materialize **6.18**, then ops
1.90. Tree 4.46 and push 3.45 still sit beside from 7.
Next intern if named: `materialize_into` on the success
path (not a second hasher; not skip stamp).
