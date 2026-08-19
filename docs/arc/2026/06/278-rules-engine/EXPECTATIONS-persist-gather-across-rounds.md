# EXPECTATIONS — persist-gather (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | round census | `cargo nextest run --release -E 'test(accum_alpha_memory_shape)' --no-capture` | rounds + `alpha_elements` printed |
| 2 | index builds | `cargo nextest run --release -E 'test(gather_index_is_built_once_per_alpha_and_keyset)' --no-capture` | builds ≤ 2, elements ≤ 80,000 |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 5 | in-fire | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | `[200 200]` fold < 25; snapshot < 1; FIRE printed |

## Independent prediction

**Runtime: 30–45 minutes.** Census, then the lifetime move.

**Predicted:** accum `[200 200]` FIRE **~49** (wash) unless
the census shows Reading alpha growing after round 0. Index
builds stay ≤ 2. Deep-cascade / strat is the named cell if
accum is one-round.

## Trap doors

1. Do not persist to dodge the fold.
2. Do not store the cache on the Session.
3. Do not key on `alpha_id` alone.
4. Do not wall-gate FIRE.

## Will not accept

- Inventing a cascade harvest to justify a zero win on accum.
- 297. Fact insertion. Retry 2e / 2o.
