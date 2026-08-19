# DESIGN-STONE — leftover production is remainder vs tax-in-parent

> **Origin (2026-08-18).** Match-pool landed. 2o extra-span tried and
> reverted. Fanout `[100 20]` FIRE **44.23**. Production **19.34**
> (46%). Probe leftover twice missed. Do not intern `names`. Do not
> guess a third probe intern.

## The measurement we do not have

`prod:compiled-rhs` last printed **6.36 net** (2m). `prod:dedup-store`
is at the instrument floor. Production is **19.34**. Naive unmarked
is `production_net − rhs_net − dedup_net` ≈ 12 ms.

That subtraction is a lie. `render_phase_table` says: *net = raw
MINUS this row's own pairs. PARENT rows still contain their
children's share.* A child that fires 40,000 times has ~4 ms of
clock reads subtracted from **its** net and left inside the
parent. Two children put ~8 ms of phantom "unmarked" in
production. Same class as the alpha:candidates trap.

`rhs_construct_cost_split` already ranked the construct pile
(bind-get 30%, stamp 28%, wrap 18%, `Arc<Vec>` 13%, Vec 11%).
It does not say whether 19.34 is that pile or leftover work
outside `exec_compiled_rhs`.

## The algorithm

Reuse `fanout_phase_census(100, 20)`. Mean of 3. Calibrate the
mark pair the same way `render_phase_table` does. Print:

```
production          raw / net / pairs
prod:compiled-rhs   raw / net / pairs
prod:dedup-store    raw / net / pairs

remainder_raw   = prod_raw − rhs_raw − dedup_raw
tax_in_parent   = (rhs_pairs + dedup_pairs) × cal
naive_unmarked  = prod_net − rhs_net − dedup_net
                = remainder_raw + tax_in_parent
```

Treat **remainder_raw** as the leftover work. Treat
**compiled-rhs_net** as the construct pile (2l). Rank those two.
Do not nest a new 40k mark. No fire-path change.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine.**
The next strike is drawn from the larger of remainder_raw and
compiled-rhs_net, not from naive unmarked. Do not intern
`names`. Do not skip the stamp. Do not rewrite `seen`.

## The gate

1. `fanout_production_leftover_split` prints the four numbers.
   `prod:compiled-rhs` pairs = 40,000. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking. Independent guess (written first): **tax_in_parent
owns most of the naive 12 ms.** remainder_raw is small.
compiled-rhs_net is the real wall and is still 2l's pile.
If remainder_raw leads, say so — that is a different stone.

## Blast radius

`src/rete/kernel.rs` tests only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Nested 40k phase marks.
- Intern `names`. Skip stamp. Rewrite `seen`. Persist. 297.
- Retry 2o / probe intern.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`fanout_production_leftover_split` `[100 20]`, mean of 3.
Instrument **86.9 ns**/pair. Production this run **21.60**
(wash vs 19.34).

| lump | ms | what |
|---|---:|---|
| production raw | 21.60 | 2 outer marks |
| compiled-rhs raw / net | 10.16 / **6.68** | 40 000 pairs |
| dedup-store raw / net | 5.96 / **2.49** | 40 000 pairs |
| **remainder_raw** | **5.48** | prod − rhs − dedup |
| **tax_in_parent** | **6.95** | 80 000 × 86.9 ns |
| naive unmarked | 12.43 | remainder + tax |

Prediction half-right: tax_in_parent owns **56%** of the naive
12 ms. remainder_raw is **not** small — 5.48 / 80 000 = **68 ns**,
the `Instant::now` + `phase_end` HashMap that sit *after* the
child's elapsed is captured. Together they *are* the unmarked.

compiled-rhs_net **6.68** is the largest engine row and is still
2l's pile. dedup_net 2.49 is 62 ns/op — at the floor. Do not
intern `names`. Do not rewrite `seen`. The inner marks are
`#[cfg(test)]` only; a release fire never pays the 12 ms.

Next is not a production intern.
