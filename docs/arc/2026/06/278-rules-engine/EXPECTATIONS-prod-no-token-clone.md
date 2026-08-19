# EXPECTATIONS — prod-no-token-clone (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | no clone-collect | read the diff | no `new_tokens.extend(...cloned())` in production |
| 2 | fanout census | `cargo nextest run --release -E 'test(fanout_fire_phase_census)' --no-capture` | production printed; hash-join > 0 |
| 3 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Loop reshape.

**Predicted:** production 36.84 → ~12–20 ms. FIRE 72 → ~50–60.
If production stays ~37, leftover is RHS construct — say so.

## Trap doors

1. `:or` still visits every parent.
2. `support` still clones the token when armed.

## Will not accept

- Class-string intern. Persist. `right_idx` rewrite.
