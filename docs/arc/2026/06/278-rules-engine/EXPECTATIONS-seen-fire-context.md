# EXPECTATIONS — seen-fire-context (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_seen_fire_context_split)' --no-capture` | insert > 0; A / X / S printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 10–20 minutes.**

**Predicted:** insert owns ~3.5 of 4.0. Alloc tiny.
Isolated S ~1.7. In-fire − S is fire context (~2 ms).
No intern this stone.

## Trap doors

1. Isolated facts are the seeded Session, not synthetic.
2. Two extra pairs, not per fact.
3. Do not intern Session-`Vec`.

## Will not accept

- 297. Insertion. Retry 2e / 2o. Skip seen inputs.
