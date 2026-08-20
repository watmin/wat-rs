# EXPECTATIONS — alpha-class-lookup (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | split | `cargo nextest run --release -E 'test(accum_alpha_class_lookup_split)' --no-capture` | S > 0; n_types printed |
| 2 | rete lib | `cargo nextest run --release --lib rete::` | 0 failed |
| 3 | clippy | `cargo clippy --offline --lib -- -D warnings` | silent |

## Independent prediction

**Runtime: 15–25 minutes.**

**Predicted:** L wins. Cut ~2–3 ms. Intern linear roots.
G−E 3.26 → ~0.3–1. FIRE not wall-gated.

## Trap doors

1. Do not under-approx (miss a class).
2. Do not pointer-hash `Arc<str>`.
3. Do not intern `children` / `range_children`.

## Will not accept

- 297. Fact insertion. Retry 2e / 2o.
