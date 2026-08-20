# EXPECTATIONS — dirty-join-parents (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | A0 extra | `cargo nextest run --release -E 'test(a0_depth_cost_split_at_equal_work)' --no-capture` | hash-join extra printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** hash-join extra 1.43 → ~0.2. ROUND extra
2.06 → ~0.8. Cascade FIRE −1. First-keying still hits.

## Trap doors

1. First-keying fires the round the second side arrives
   (that side's delta is non-empty).
2. Join-after-filter (Test→HashJoin) stays its own pass.
3. Middle join (tri): `d_beta[J1]` dirties J1 as parent of J2.

## Will not accept

- Root-join dirty set. Scratch. 297. Insertion. Intern `names`.
