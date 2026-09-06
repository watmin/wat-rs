# SCORE — `filter:test-pass` names one quantity

The reuse arm no longer bumps `filter:test-pass`. The waste gate refuses this axis instead of reporting `0.0 < 50.0`. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ one key, one quantity | **HOLD.** Reuse arm bumps only `filter:test-reuse`. `filter:test-pass` is evaluated-and-passed. One-line docs at every increment site. |
| 2 ★ `test-pass ⊆ test-evals` | **HOLD.** Both eval arms bump `filter:test-evals` immediately before a possible `filter:test-pass`. The reuse arm does not bump pass. Guaranteed by construction. |
| 3 ★ no identity gate | **HOLD.** `evals <= 2*passes`, `evals <= 4m`, and `worst_waste < 50.0` are gone from this axis. Waste% prints `n/a`. |
| 4 ★ refusal driven | **HOLD.** `assert_waste_applicable` requires `evals > 0`. `node_share_waste_gate_is_refused_when_evals_are_zero` catch_unwind-drives it and requires `INAPPLICABLE SHAPE`. |
| 5 numbers moved | **HOLD.** Table below. |
| 6 no engine change | **HOLD.** Same `record_token` sites; only a census bump was removed. Floor green includes the branch-pair differential. |
| 7 floor | **HOLD.** `Summary [ 453.783s] 5440 tests run: 5440 passed (3 slow), 21 skipped`. `.floor/2026-09-05T23-48-48Z/`. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. Row 3 is the point of the strike.

## Numbers that moved

`node_share_filter_eval_census`, all three ladder points `[10\|25\|50] × 200`:

| key | before | after |
|---|---|---|
| `filter:test-evals` | 0 | 0 |
| `filter:test-reuse` | 200 | 200 |
| `filter:test-pass` | **200** | **0** |
| wasted / waste% | 0 / 0.0% | not a number (`n/a`) |

`filter:test-reuse` already named the tree-proven push. No second key: a duplicate of reuse would have been the union under a new spelling.

## Why not a new key for the reuse push

`filter:test-reuse` is the event. The extra `filter:test-pass` on that arm was the mis-named union. Consumers that need "a token reached a TestNode push" sum `reuse + pass` if they want it; this axis's waste arithmetic does not.

## Other reader of `filter:test-pass`

`where_tree_branch_differential.rs` reads the key into a diagnostic field. It does **not** assert the union (skip arithmetic is `ref.evals − tree.evals − tree.reuse`). Its field doc was updated. STOP-2 did not fire: no second assertion depended on the union meaning.

## Still open (named, cut)

Census sections B–M. A4, D2p, F2.
