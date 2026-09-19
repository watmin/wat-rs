# SCORE — three timing diagnostics runed; `excusare` vocabulary minted and gated

The four timing-ordering assertions are gone. The three sites carry `rune:excusare(…)` in their
`#[ignore]` reason. The vocabulary is in `WARD_VOCABULARIES` and in `docs/CONVENTIONS.md`. Floor
moved exactly −1 run / +1 skipped. No tolerance. No `benches/` move.

## Scorecard

| # | result |
|---|---|
| 1 ★ ordering assertions gone | **HOLD.** `grep -nE '^\s+[a-z_]*(trie\|arr) [<>]' binding_repr_bench.rs` → no hits. |
| 2 ★ three runed reasons | **HOLD.** `:146` `below-resolution` (1.0–1.9× vs 3.5×–4.4× band). `:264` `no-falsifier` (five ops × two reps × four cards; no single ordering). Dominance `below-resolution` (captured red 5860.1 vs 3995.9 at card 64, ≈5.3× on a curve that reads 871.5 at 32). |
| 3 ★ the rune is GATED | **HOLD.** `excusare` row in `WARD_VOCABULARIES`. Invented `invented-for-mutation` at the dominance ignore → RED, quoted below, restored. |
| 4 ★ vocabulary table | **HOLD.** `docs/CONVENTIONS.md`, purgare's shape. `perennial` is the ward's own. `below-resolution` / `no-falsifier` marked proposed-upstream with `~/work/NOTE-excusare-lacks-a-term-for-a-gate-that-cannot-be-built.md`. |
| 5 ★ dated verdict | **HOLD.** Six isolated samples, median + range, both columns, in the dominance fn's doc comment. Trie won card-64 EXTEND in 6/6 — not STOP-1. |
| 6 faithfulness survives | **HOLD.** The twins-equal-binding-set loop is still above the timings. |
| 7 floor | **HOLD.** `.floor/2026-09-07T04-32-44Z/`: `Summary [ 461.850s] 5470 tests run: 5470 passed (3 slow), 22 skipped`. Predicted 5471→5470 / 21→22. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 3 is what stops this being a marker with no checker.** Row 5 is what replaces a live gate with an honest measurement.

## Row 5 — the six samples (isolated, release, 2026-09-07)

Largest cardinality (64), both columns. Median of six is the mean of the two central values.

| sample | EXTEND trie | EXTEND array | GET trie | GET array |
|---|---|---|---|---|
| 1 | 697.2 | 2119.4 | 33.1 | 340.9 |
| 2 | 688.0 | 2012.6 | 33.1 | 344.8 |
| 3 | 642.3 | 2569.8 | 69.9 | 662.5 |
| 4 | 664.6 | 1989.4 | 28.4 | 349.8 |
| 5 | 716.7 | 2019.7 | 39.7 | 343.0 |
| 6 | 652.1 | 2037.8 | 37.0 | 347.5 |
| **median** | **676.3** | **2028.8** | **35.1** | **346.2** |
| **range** | 642.3–716.7 | 1989.4–2569.8 | 28.4–69.9 | 340.9–662.5 |

DOMINANCE: NO in all six. Trie won EXTEND at card 64 in 6/6. The captured floor red (trie 5860.1 vs array 3995.9) did not reproduce in isolation; that is the contention-band claim, not a representation flip. R60's cut stands.

## Row 3 — mutation

At `binding_repr_bench.rs:597`, the dominance ignore's category was `invented-for-mutation` for one drive:

```
ward rune category outside its closed set (1 site(s)). …
  /home/john/work/holon/wat-rs/src/rete/kernel/tests/binding_repr_bench.rs:597 — rune:excusare(invented-for-mutation) is not one of ["perennial", "below-resolution", "no-falsifier"]
```

Restored to `below-resolution`. Gate green after restore.

The five existing `rune:excusare(perennial)` sites in `src/comms/` became visible to the gate the moment the row landed — they were always in the tree, previously unregistered. `process.rs` already carried all three wards, so the positive control did not need moving. Non-vacuity floor for `excusare` is 5 (the perennial population); the zip is length-checked against `WARD_VOCABULARIES`.

## What stayed, stated

The dead-clock assert (`extend_array_wins + get_array_wins > 0`) and the `rows.len() == cards.len()` structural check remain. They are not timing-orderings between the two columns. The printed table and the `DOMINANCE:` verdict line remain — a human reads them on demand via `--run-ignored=only`.

## What this did not do

Did not put a tolerance on the assertion. Did not take min-of-k. Did not move the three into `benches/`. Did not touch `binding_cardinality_distribution`. Did not treat the captured red as a flake.

## Final floor

`.floor/2026-09-07T04-32-44Z/`: `Summary [ 461.850s] 5470 tests run: 5470 passed (3 slow), 22 skipped`.
