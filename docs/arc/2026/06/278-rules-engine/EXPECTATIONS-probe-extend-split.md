# EXPECTATIONS — probe-extend-split (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(probe_extend_cost_split)' --no-capture` | B / M / E / K / H printed; E > 0 |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.** Test only.

**Predicted rank of the 12.30:** E (extend_token) leads; B and
M split the copies; K+H < 1 ms at 2,000 lefts. Absolute ns
will not equal 307 — allocator state differs from a fire.
The ratio names 2r.

## Trap doors

1. Do not add 40k nested marks.
2. Scale K and H to **2,000**, B/M/E to **40,000**. Mixing
   those counts is the lie.
3. Do not retry `Token.extra` even if B leads (2o).

## Will not accept

- Engine rewrite in this diff.
- FIRE wall gate.
- Intern `names`. 2o retry.
