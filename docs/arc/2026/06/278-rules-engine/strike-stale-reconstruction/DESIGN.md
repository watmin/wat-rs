# DESIGN — a reconstruction checked against a constant nobody re-measured

## Why

Work-list **C6**. `node_share_where_cost_decomposition` reconstructs the `filter` phase from timed
arms and prints how much of it they account for. The target is frozen in the source:

```rust
// The measured `filter` phase this reconstructs (2026-08-01, node_share_fire_phase_census,
// [50 200]). Printed so the reconstruction can be CHECKED, not assumed: if B + C does not
// land near it, the harness is measuring something the fire does not do.
const FILTER_MS_MEASURED_IN_FIRE: f64 = 6.83;
```

**The comment declares the exact check that would catch this, and nothing performs it.** Driven at
HEAD, both tests at the SAME `[50 200]` the comment names:

| | |
|---|---|
| `FILTER_MS_MEASURED_IN_FIRE`, frozen 2026-08-01 | **6.83 ms** |
| live `filter`, `node_share_fire_phase_census` | **0.38 ms** ⚠ corrected — see below |
| the table's own verdict | **`RECONSTRUCTION B+C = 9.878 ms … ( 145% accounted)`** |

The constant is **~18x stale**. The printed check has been reading 145% — its own stated failure
condition — for a month, in a `println!` with no assertion behind it.

⚠ **The staleness hides a WIN, not a regression.** `filter` fell from 6.83 to 0.14 because the
compiled-where work landed. An instrument frozen at the old number cannot show the improvement it
was built to track.

## The second defect: the reconstruction uses the arm the fire does not run

`B` is `env build + eval_inner walk` — the whole of **`eval_test_core`**. The native filter path does
not call it. Verified on the current tree:

- `fire/pass/filter.rs` builds from `arm.compiled_wheres` and calls `dispatch_where_tests`; the only
  `exec_where` call sites are `fire/mod.rs:304,493,1996`.
- `eval_test_core` has **one** live caller in non-test `src/`: `eval_test.rs:150`, inside
  `eval_test`, which is reachable only as the wat builtin `:wat::rete::eval-test`
  (`runtime.rs:5751`). The other eight references are prose.
- Arm **F** *is* the native path — `lower-once + exec_where`, 2.597 ms / 259.7 ns per eval.

So the table reconstructs a native phase out of an interpreter arm. `B/F = 3.75x` is printed in the
same table, so the two are known to differ by nearly 4x — and the reconstruction still uses `B`.

## The contract decision, pinned

**The reconstruction reads the filter phase LIVE, compares the NATIVE arm, and ASSERTS the result.**

- Replace `FILTER_MS_MEASURED_IN_FIRE` with a live `node_share_phase_census(50, 200)` read of the
  `filter` row, in the same test. A constant that must be hand-maintained to stay true is the
  failure class, not the value.
- The reconstruction line becomes `F + C` against that live reading. `A`, `B`, `D`, `E` stay exactly
  as they are — they are the interpreter *headroom* study (`B−E` is what a perfect compile could
  remove) and that remains honest and useful.
- The declared check becomes an assertion with a band chosen from measurement.

## ⛔ THE STOP THAT MATTERS: DO NOT CHOOSE THE BAND TO MAKE IT PASS

`F + C ≈ 2.7 ms` against a live `filter` of `~0.39 ms` is **~7x over**, so the assertion may not
pass even after both fixes. **That is a finding, not a band to widen.** If the native arm cannot
reconstruct the phase it claims to decompose, the harness measures something the fire does not do —
which is precisely what the original comment said, and what nobody ever ran.

## ⛔⛔ CORRECTED AFTER THE STRIKE RAN — TWO OF THIS FILE'S CLAIMS WERE WRONG

1. **The live `filter` figure above was `0.14 ms` and is `0.38`.** The census prints THREE size
   blocks (10/200, 25/200, 50/200) and I took the first `filter` grep hit — the 10/200 row. Every
   ratio derived from it was wrong: ~18x stale, not 49x; ~7x over, not 19x. **Take a row with its
   block header, never by a bare grep.**
2. **⛔ THE PREMISE OF THIS DESIGN WAS FALSE. The fire calls `exec_where` ZERO times on this axis.**
   `dispatch_where_tests` (`fire/mod.rs:2012`) finds every candidate
   `proven && is_pure_cmp` and takes the reuse branch at `:2038`, skipping the eval. Measured, every
   size: `evals 0, reuse 200, envs 0, keyallocs 0`. So arm `F` is scaled to the PRE-where-tree count
   of 10,000, "ONE ROUND'S WORTH" is itself stale, and no rescaling rescues the reconstruction —
   at the true scale `F` contributes nothing. Swapping `B → F` is cosmetic beside that. See
   `SCORE.md` § A and § C.

## Files

- `src/rete/kernel/tests/node_share_cost.rs` only.
- Nothing under `src/rete/kernel/fire/`. **The engine is not the defect; the ratio it produces is a
  49x improvement the instrument failed to notice.**

## Out of scope = REJECTED

- C5 (`binding_repr_bench.rs`'s tautology assert), C9, C10, C11. Separate rows.
- Re-deriving why `filter` fell 49x. Interesting, and not this strike — this strike makes the
  instrument able to see it.
