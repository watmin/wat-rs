# BRIEF — make the reconstruction read the phase live, compare the native arm, and assert it

One benchmark reconstructs the `filter` phase and checks itself against a constant frozen a month
ago. The constant is ~49x stale, the arm it reconstructs from is the interpreter rather than the
native path, and the check is a `println!` with nothing behind it. Make all three true.

## Read in order

1. `src/rete/kernel/tests/node_share_cost.rs:255-290` — the constant
   `FILTER_MS_MEASURED_IN_FIRE = 6.83`, the comment that declares the check, and the `println!` that
   prints `RECONSTRUCTION B+C … (% accounted)`.
2. `src/rete/kernel/tests/node_share_cost.rs:60-79` — the test's own doc: why the arms are scaled to
   "ONE ROUND'S WORTH", and `evals_per_round = N * tokens.len()` at `:136` (50 × 200 = 10,000).
   This is why the comparison is legitimately on the same scale.
3. `src/rete/kernel/tests/node_share_cost.rs:178-245` — arms A, B, C, D, E, F. **F is the native
   path** (`lower-once + exec_where`); B is the whole of `eval_test_core`.
4. `src/rete/kernel/tests/node_share_cost.rs:414-450` — `node_share_fire_phase_census`, and its
   `node_share_phase_census(50, 200)` call at `:446`. This is where the live `filter` row comes from
   and the helper you will reuse.
5. `src/rete/kernel/fire/pass/filter.rs:22-60` and `src/rete/kernel/fire/mod.rs:1996` — the native
   filter path: `compiled_wheres` → `dispatch_where_tests` → `exec_where`. Confirms F is the arm
   that matches the fire.

## Driven at HEAD, so you know the ground

```
F  compiled exec_where     ( 10000 x)     2.597 ms    259.7 ns/eval
B  env build + walk        ( 10000 x)     9.743 ms
C  token clone             (    50 x)     0.135 ms
RECONSTRUCTION  B+C =  9.878 ms  vs a measured `filter` of 6.83 ms  ( 145% accounted)
```
and from `node_share_fire_phase_census`, same `[50 200]`: `filter  0.14 ms raw  0.14 net`.

## The three pieces

1. **Read the phase live.** Call `node_share_phase_census(50, 200)` inside this test and take the
   `filter` row, the way `:446` does. Delete `FILTER_MS_MEASURED_IN_FIRE`.
2. **Reconstruct from the native arm.** The reconstruction line becomes `F + C` against that live
   reading. Leave A, B, D, E and every derived row (`B−A`, `D−A`, `B−E`, `B/F`) exactly as they are:
   they are the interpreter headroom study and remain honest.
3. **Assert it.** The comment's declared check becomes a real assertion with a band you pick from
   measurement, and the failure message must interpolate the whole table so a red arrives carrying
   its own evidence.

## Blast radius

`src/rete/kernel/tests/node_share_cost.rs` only. Nothing under `src/rete/kernel/fire/`.

## STOP triggers — halt and report rather than adjusting

1. **⛔ If the assertion cannot pass, DO NOT WIDEN THE BAND TO FIT.** `F + C` is ~19x the live
   `filter` at HEAD. If that holds, the native arm does not reconstruct the phase it claims to
   decompose — report the numbers and stop. That is the finding, and it is exactly what the original
   comment predicted. A band chosen so the test goes green is the defect this strike exists to
   remove, re-created one level up.
2. **If the live `filter` read comes back at a very different scale than 0.14 ms**, stop and report
   — the two tests must be at the same `[50 200]` for the comparison to mean anything.
3. **If you find yourself editing `src/rete/kernel/fire/`**, stop. The 49x drop is a real engine win;
   the instrument is what failed to see it.
4. **If removing the constant changes any row other than the RECONSTRUCTION line**, stop and report
   which.

## Mutation proofs — run both, report both

1. **Point the live read at a phase that is not `filter`** (e.g. `production`) → the assertion must
   go RED. Proves it reads the row it names.
2. **Scale one arm** (multiply `F` by 10 before the reconstruction) → the assertion must go RED.
   Proves the band is narrow enough to catch a real change rather than admitting anything.

## Report

- The table before and after, verbatim.
- The live `filter` value you read, and the band you chose **with the samples you chose it from**.
- Whether STOP-1 fired. If it did, that is a successful strike, not a failed one — report the
  numbers and stop.
- Both mutation results.
- Your scoped nextest Summary lines including `binary_id(wat::lint)`.
- Anywhere this brief was thin, wrong, or pointed you at the wrong line. Be blunt; the last three
  riders each found a real defect in the brief and those were the most valuable lines they returned.
