# EXPECTATIONS — where-dim-reuse (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | eval census | `cargo nextest run --release -E 'test(node_share_filter_eval_census)' --no-capture` | `[50 200]` passes ≈ 200; reuse or evals > 0; not 10 000 evals |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |
| 4 | FIRE | `cargo nextest run --release -E 'test(node_share_fire_phase_census)' --no-capture` | `[50 200]` FIRE printed; not wall-gated |

## Independent prediction

**Runtime: 20–35 minutes.**

**Predicted:** `[50 200]` evals → **0** (all reuse), passes
**200**. filter 0.50 → **~0**. FIRE 1.71 → **~1.2–1.5**.
Accum unchanged in kind.

## Trap doors

1. Do not skip maybe / wildcard.
2. Do not skip a `where` that is not only `(= dim lit)`.
3. Dim raise still `exec_where`.
4. Do not wall-gate FIRE.

## Will not accept

- Range edges. 297. Fact insertion. Retry 2e / 2o.
