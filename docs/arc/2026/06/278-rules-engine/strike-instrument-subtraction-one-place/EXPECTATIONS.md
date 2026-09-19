# EXPECTATIONS — one place for the instrument subtraction

| what | command | expected |
|---|---|---|
| the helper exists and is a free `fn` | `grep -c 'fn net_ns' src/rete/kernel/tests/mod.rs` | 1 |
| both private closures are gone | `grep -c 'let net_of = \|let net = ' src/rete/kernel/tests/{mod,accum_cost}.rs` | 0 for the subtraction spellings |
| every site converted | `grep -rnE '\-.*as f64 \* cal' src/rete/kernel/tests/*.rs` | **0** remaining hand-rolled |
| the count was re-derived | SCORE names 13 or reports the delta | stated either way |
| the two folds keep their `min` at the call site | read `fanout_cost.rs`, `strat_cost.rs` | `.min(net_ns(…))`, not `net_ns` returning a min |
| ⛔ **numbers unchanged** | capture one cost table's output before and after (`cargo test --release <a cost test> -- --exact --nocapture`), `diff` them | **byte-identical**, modulo timing columns. This is the strike's whole safety property |
| the helper cites the argument | `grep -c 'two copies is how one of them silently stops\|mod.rs:47' <helper doc>` | ≥1 — the doc points back at the comment that demanded it |
| cost tests green | `cargo nextest run --release -E 'test(cost)'` | green |
| floor | `./scripts/floor.sh` | 5485, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

35–50 min. Thirteen mechanical edits plus one floor.

## Trap doors

- **Absorbing `min` into the helper.** The pinned decision. `mod.rs:303-308` is the record of what
  happens when the estimator is decided in two places; this strike must not decide it in one wrong
  one.
- **A cast that changes a value.** Sites differ: some pass `f64` raws, some `u64` (`fanout_cost.rs:558`
  is `rhs_raw as f64`, `:828` is `ns as f64`). Match each site's existing arithmetic exactly; if a
  cast would move, STOP-2.
- **Trusting 13.** Two prior counts of this same population were wrong — the ward's 17 and my 20.
  Re-derive and report.
- **Treating this as cosmetic.** The file already carries one defect from this class
  (`mod.rs:303-308`). The comment predicted it and was right.
