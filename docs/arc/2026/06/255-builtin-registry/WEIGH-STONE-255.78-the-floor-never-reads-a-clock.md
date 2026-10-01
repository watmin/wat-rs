# WEIGH — STONE 255.78: the floor never reads a clock to decide (T1) — ACCEPTED

**Executor: a Sonnet subagent, commit `f12c10995`.** Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator at `f12c10995`

| row | result |
|---|---|
| release floor | `.floor/2026-10-01T18-41-07Z`: **6349 passed / 24 skipped**, exit 0; `harvest_wrap_split` and all 11 `floor_never_reads_a_clock` tests pass |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| engine cost of the new counters | `census_count`/`census_count_n` are `#[cfg(not(test))]` no-ops (`src/rete/kernel/census.rs:697-703`); the snapshot witness is `#[cfg(test)]`-gated |

## What landed

| gate | now |
|---|---|
| `harvest_cost.rs` `harvest_wrap_split` | counters inside the three closures; scan matches and wrap maps must agree (mutation-proved) |
| `gather_probe_cost.rs` `drop_memories_cost_split` | allocation delta of the clear at N and N/10 through the existing `CountingAllocator` (mutation-proved) |
| `node_share_cost.rs` set-probe majority | operation counts (`tokens × tids` against the other rungs); a `b > a && b > e` cross-timing clause in the same fn struck |
| `accum_cost.rs` fold < 25 ms | `accum:rematch` count = 0 (the rematch `DESIGN-STONE-accum-fold-the-wall` names) |
| `accum_cost.rs` snapshot < 1 ms | `accum:snapshot-alloc-bytes` = 0 |
| `fanout_cost.rs` production > 2× hash-join | **the claim had no count witness:** measured, both phases handle 40,000 items 1:1; the 9× wall gap is per-item cost. The clock assertion was dropped; a structural `hj_emitted == prod_derivations` invariant stays; the timing table still prints |
| `binding_repr_bench.rs` array < trie | deleted from the floor; `benches/binding_repr.rs` carries its twin |

**The wall:** `tests/lint/floor_never_reads_a_clock.rs`, a syntactic per-function clock-taint check over `src/**/tests/**`
and `tests/**`, mutation-proved. It reaches zero with **16 code-site `rune:lint(clock-verdict)` exemptions**, each a
deterministic count the taint over-reached into (shared short names, `(ns, count)` tuples), with its reason. That is a
noisy instrument: the rune count is the thing to watch, and a new rune is a review point.

## For the builder

- **STOP-3, a hang watchdog:** `tests/process/doomed_child_boot_ack_does_not_hang.rs:87`, `assert!(elapsed < 2s)`. The
  claim is "does not hang"; the executor says a hang has no clock-free witness. `mora`'s reading differs: the parent's
  ack write end being closed is a wire fact (EOF on the pipe) that decides it without a duration. Rune-exempted pending
  your ruling.
- The fanout dominance claim is no longer checked on the floor (above).

## Verdict

Accepted and pushed, together with 255.77.
