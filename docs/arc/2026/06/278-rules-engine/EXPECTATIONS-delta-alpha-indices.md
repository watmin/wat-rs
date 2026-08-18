# EXPECTATIONS — delta-alpha-indices (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | no clone at push | read the diff | `wm.alpha[...].push(el)` — not `el.clone()` |
| 2 | census | `cargo nextest run --release -E 'test(accum_fire_phase_census)' --no-capture` | snapshot < 1 ms; fold < 25 ms. FIRE is printed, not gated (wall-clock lied: 83.99 quiet / 97.55 loaded) |
| 3 | rete lib | `cargo nextest run --release -E 'test(rete::)'` | 0 failed |
| 4 | rete integration | `cargo nextest run --release -E 'binary_id(wat::rete)'` | 0 failed |
| 5 | clippy | `cargo clippy --offline --all-targets -- -D warnings` | silent |

## Independent prediction

**Runtime: 20–30 minutes.** Type + three sites.

**Predicted:** a few ms (unmarked `d_alpha` drop). If FIRE stays
~92 ms, say so — the copy was not the row.

## Trap doors

1. Root-join / hash-join borrow of `wm.alpha` vs `wm.beta`.
2. `right_idx` still clones — leave it.
3. Index after `push` is `len() - 1`. Do not push to `d_alpha`
   before `wm.alpha`.

## Will not accept

- Persist bundled in.
- `right_idx` rewritten here.
- A silent FIRE miss without the table.
