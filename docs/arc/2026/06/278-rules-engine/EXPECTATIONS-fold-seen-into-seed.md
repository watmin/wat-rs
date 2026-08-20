# EXPECTATIONS — fold-seen-into-seed (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | leftover | `cargo nextest run --release -E 'test(accum_leftover_split)' --no-capture` | setup:seen < 0.5 ms; FIRE > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** setup:seen 3.99 → **~0.01**. FIRE 19.78 →
**~18**. Seed may rise (HashSet insert moves into it).

## Trap doors

1. Every input fact still enters `seen` before production.
2. Do not mark seen_insert per fact.
3. Delta rounds do not re-insert inputs.

## Will not accept

- Session-`Vec`. Skip seen inputs. 297. Insertion.
