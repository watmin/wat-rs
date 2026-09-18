# DESIGN-STONE — honest FIRE rank after the reserve

> **Origin (2026-08-18).** 2s reserved catch-up pools. FIRE
> **39.48**. Production **19.09** — 2p named ~12 ms of that as
> `#[cfg(test)]` instrument. Probe leftover is copies (B 5.38,
> 2o-dead). compiled-rhs_net **4.81** is still 2l's pile.
> Weigh before drawing. Do not intern `names`. Do not retry 2o.

## The measurement we do not have

2p's leftover-production table is pre-slot, pre-reserve.
compiled-rhs_net moved 6.68 → 4.14 → **4.81** (wash).
Production **21.60 → 19.09**. FIRE **45.20 → 39.48**.
The parent still *looks* like the wall. 2p's law still holds:
subtracting child *nets* leaves the 80k clock reads in the
parent. Release `phase_start` is empty. A rank that treats
19.09 as engine work will draw the wrong stone.

## The algorithm

Reuse `fanout_phase_census(100, 20)`. Mean of 3. Same
calibration as `render_phase_table`. Print:

```
FIRE                  IN + SETUP + ROUND + OUT
production            raw / net
compiled-rhs          raw / net
dedup-store           raw / net
probe                 raw / net
remainder_raw         prod − rhs − dedup
tax_in_parent         (rhs + dedup) pairs × cal
honest_prod           rhs_net + dedup_net
honest_FIRE           FIRE − remainder_raw − tax_in_parent
```

Rank engine rows: probe, honest_prod's children, alpha, out.
A row is drawable only if it is ≥ 1 ms **and** not 2o-dead,
not intern-`names`, not skip-stamp.

No fire-path change. No new 40k marks.

## ★ THE ONE CONTRACT DECISION

**This stone prints the rank. It does not change the engine.**
The next strike is drawn from the largest *drawable* engine
row, not from production raw. Do not retry 2o.

## The gate

1. `fanout_honest_fire_rank` prints the table.
   compiled-rhs pairs = 40,000. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking. Independent guess (written first): **honest_FIRE
≈ 27 ms.** Probe **7.08** (copies, B 2o-dead) is the largest
engine leftover. compiled-rhs_net ~5 is still a pile. Nothing
drawable ≥ 1 ms that is not 2o / names / stamp. Say so.

## Blast radius

`src/rete/kernel/tests/` only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Nested 40k marks. `Token.extra`. Intern `names`. Skip stamp.
- Persist. 297. Rewrite `seen`.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`fanout_honest_fire_rank` `[100 20]`, mean of 3.
Instrument **102.0 ns**/pair. FIRE this run **41.80** (wash
vs 39.48).

| lump | ms |
|---|---:|
| FIRE | 41.80 |
| production raw | 19.97 |
| compiled-rhs net | **4.66** |
| dedup-store net | 1.93 |
| probe | **8.11** |
| hash-join | 9.23 |
| alpha | 6.56 |
| OUT | **3.26** |
| remainder_raw | 5.22 |
| tax_in_parent | 8.16 |
| honest_prod | 6.59 |
| **honest_FIRE** | **28.42** |

Prediction held on honest_FIRE (≈27). Probe **8.11** is the
largest engine leftover and is still the copies (B 5.38,
2o-dead). compiled-rhs_net **4.66** is still 2l's pile
(stamp / wrap / Arc / Vec).

Drawable and not 2o / names / stamp: **OUT 3.26** (almost all
`out:production` — freeze of 40k Pairs). Alpha raw is the
candidates trap; do not intern it.

Do not retry 2o. Do not intern `names`. Do not skip the stamp.
