# DESIGN-STONE — leftover accum is remainder vs tax, not alpha raw

> **Origin (2026-08-18).** 2v named the next cell: accum
> `[200 200]` FIRE **61.72**. Top-row alpha **41.65**. That is
> the candidates trap until a leftover split names remainder vs
> tax (same class as 2p). Do not intern alpha off the raw. Do
> not persist gather. Do not start 297.

## The measurement we do not have

`alpha:*` marks fire per fact. At G=200 W=200 that is tens of
thousands of pairs. `render_phase_table` subtracts cal from
each *child* and leaves those clock reads in the parent.
2p proved production's naive unmarked was tax + bookkeeping.
Alpha 41.65 has not been split that way on today's engine
(fold gone, pools landed).

`setup:seen` leftover after 2g is HashSet insert. Drop after
2f is Value Drop. Fold after #1 is ~2 ms. Production is the
2l pile. Rank those against honest alpha before drawing.

## The algorithm

Reuse `accum_phase_census(200, 200)`. Mean of 3. Same
calibration as `render_phase_table`. Print:

```
FIRE
alpha                 raw / pairs
  candidates/match/element/push   raw / net / pairs
remainder_alpha       alpha_raw − Σ children_raw
tax_in_alpha          Σ children_pairs × cal
honest_alpha          Σ children_net (floor 0)

setup:seen            raw / net
drop-memories         raw / net
accumulate / fold / index / snapshot
production / compiled-rhs / dedup
filter
hash-join
OUT

honest_FIRE           FIRE − remainder_alpha − tax_in_alpha
                      − (prod remainder+tax if those pairs fire)
```

Rank engine rows. Drawable only if ≥ 1 ms and not 2o-dead /
names / stamp / Session rewrite / persist-gather.

No fire-path change. No new 40k marks.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the engine.**
The next strike is drawn from the largest *drawable* engine
row, not from alpha raw.

## The gate

1. `accum_leftover_split` prints the table. FIRE > 0. Alpha
   pairs > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings`.

## Predicted win

A ranking. Independent guess (written first): **remainder +
tax own most of alpha 41.65.** honest_alpha is small (match
net ~few ms). Leftover engine is `setup:seen` and/or drop
and/or production. Fold stays small. Do not persist gather.

## Blast radius

`src/rete/kernel/tests/` only. No `.wat`. No engine change.

## Out of scope = REJECTED

- Intern alpha off the raw. Persist gather. 297.
- Nested extra marks. Intern `names`. Skip stamp. 2o.

## Sequencing

1. Test. Print. Rank. Stop.

## Weigh (2026-08-18) — LANDED

`accum_leftover_split` `[200 200]`, mean of 3.
Instrument **79.0 ns**/pair. FIRE **63.83**.

| lump | ms |
|---|---:|
| alpha raw | 43.51 |
| remainder_alpha | **20.32** |
| tax_in_alpha | **22.26** |
| **honest_alpha** | **5.00** (match net 4.37; candidates/element below instrument) |
| **setup:seen** | **7.43** |
| drop-memories | 3.65 |
| accumulate | 7.06 (index **5.23**, fold 1.82) |
| honest_prod | 0.19 |
| filter | 0.24 |
| **honest_FIRE** | **20.97** |

Prediction held: remainder+tax *are* alpha 41. Do not intern
alpha off the raw. Fold stays small. Persist gather is still
~0 on a cold fire (index is first-round hash).

Largest honest engine row: **`setup:seen` 7.43** (2g: HashSet
insert; do not add a second hasher). Second: **accum:index
5.23**. Third: match net 4.37. Fourth: drop 3.65.
