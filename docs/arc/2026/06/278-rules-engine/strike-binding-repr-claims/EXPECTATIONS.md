# EXPECTATIONS — C5

> ⛔ **INVARIANTS, NOT MILLISECONDS.** Absolute times here are not reproducible better than ~16%.

## ⛔ NO PINNED TEST COUNT

Floor ≥ its current value, zero FAIL rows.

## The scorecard

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ the floor assertion can fail | `… < usize::MAX`, message `"unreachable"` | the check its comment declares (`> 0`), message carries the table |
| 2 | both tests name the live repr | neither mentions `BindSpan` | both do, citing `session.rs:64` |
| 3 | the comparison's purpose is stated | implied only | named as evidence for `fire/delta.rs:725-726` |
| 4 | the `163 ns` anchor is gone | 4 prose mentions + printed in the table | absent; the reason stated |
| 5 | the source figure's collapse is recorded | `alpha_match_cost_per_binding` → **−22 ns/fact** | recorded where the anchor used to be |
| 6 | arms survive | trie + array arms, both floor tests | unchanged; nothing deleted |
| 7 | the retired matcher path untouched | reached only from tests | untouched — C13, not this strike |
| 8 | radius | — | `binding_repr_bench.rs` only |
| 9 | lints | 210/210 | green |
| 10 | clippy | rc=0 | silent |

## The mutation proofs

1. **Zero both win-counters** → the new assertion REDs. Proves it is the non-vacuity check its
   predecessor pretended to be.
2. **Break one asserted ordering** → REDs. Proves the orderings are load-bearing.
3. Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

30–45 minutes. The assertion is a line; the honest headers are the work.

## What would make this strike a failure even if every test passes

**Replacing one unfalsifiable assertion with another.** `> 0` on a counter that is always positive is
the same defect. Mutation 1 is what separates them, and it must actually drive the counters to zero
rather than asserting on a value the code cannot produce.

**And re-anchoring to a fresh hard-coded nanosecond figure.** The source measures −22 ns/fact; a new
constant would be C6's defect re-created in the file that sits next to it.
