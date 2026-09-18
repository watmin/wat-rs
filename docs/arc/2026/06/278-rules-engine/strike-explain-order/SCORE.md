# SCORE — F1 explain attribution order

Cure + both gates. Floor GREEN. Zero `src/`. Gate A is the proof; Gate B was only probabilistically red at HEAD.

## Scorecard

| # | result |
|---|---|
| 1 ★ oracle DETERMINISTIC | **HOLD.** Gate B green; harvest-support walks `topological-node-ids`. |
| 2 ★ AGREES with native | **HOLD.** `native_and_oracle_attribute_the_same_rule` passes. |
| 3 ★ control still discriminates | **HOLD.** `the_control_is_a_single_producer_and_agrees` passes. |
| 4 ★ Gate A mutation-proved | **HOLD.** Re-introduced `(:wat::core::PersistentMap/keys network)` as `:wat::rete::MUTATION-PROBE-DELETE-ME` in `explain.wat`. Lint RED: `wat/rete/oracle/explain.wat:122 in \`:wat::rete::MUTATION-PROBE-DELETE-ME\`: raw \`PersistentMap/keys network\` (not :wat::rete::topological-node-ids, no rune:lint(oracle-keys-order-insensitive))`. Restored → green. Detector unit tests keep both directions. |
| 5 ★ five sites classified | **HOLD.** See below. |
| 6 one definition | **HOLD.** `topological-node-ids` in `pass.wat` (loads before fire/explain). `fire.wat` fire-once calls it; the inline sort is gone. |
| 7 floor | **HOLD.** `Summary [ 450.866s] 5435 tests run: 5435 passed (1 slow), 21 skipped`. `.floor/2026-09-05T12-15-09Z/`. |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 blast radius | **HOLD.** `wat/rete/oracle/{pass,fire,explain}.wat` + `tests/lint/no_raw_network_keys_in_oracle.rs` + `tests/rete/probe_arc278_explain_order.{rs,wat}`. Zero lines in `src/`. |

## The five sites

| site | verdict |
|---|---|
| `explain.wat` harvest-support | **calls the verb.** The driven defect. First-producer-wins over topological ids. |
| `fire.wat` fire-once | **calls the verb.** The law that was already written; the copy is gone. |
| `pass.wat` alpha-feeding | **calls the verb.** Unique reverse-lookup: at most one AlphaNode lists `hj-id` in children. Sorted walk is identity under uniqueness. |
| `pass.wat` alpha-id-for-cond | **calls the verb.** Unique reverse-lookup: at most one AlphaNode whose `tests[0]` write-forms equals `cond`. |
| `pass.wat` node-parents | **runed.** Fold builds the SET of parent node-ids as a PersistentVector (`contains?` + `tokens-from-parents` concatenates EVERY parent's tokens). The SET does not depend on HAMT order. First-producer-wins is harvest-support's ProductionNode walk, not this parent list. |

Also converted, not in the original five: `fire.wat` collect-query-memory (assoc by unique qname) and `network-has-production?` (boolean OR over `net`). Both now call the verb. `accum-pass.wat` keys of accumulator maps left alone (DESIGN cut).

Sorting the four converted sites did not change any rete test (484/484 before the floor). STOP-1 did not fire.

## Gate A vs Gate B

Gate A is the proof: a raw `PersistentMap/keys network` outside the verb is unwritable (or runed). Gate B is behavioural, green after the cure, and was only probabilistically red at HEAD (`experiri` 2/8, orchestrator 0/8). Do not treat Gate B's historical red as the demonstration.

## Floor note

First floor at this strike (`.floor/2026-09-05T12-01-37Z`) was RED on `no_inlined_edn` at `no_raw_network_keys_in_oracle.rs:46` — `strip_prefix("(:wat::core::defn ")` looked EDN-esque. Captured, not re-run. Restructured to `strip_prefix('(')` + `strip_prefix(":wat::core::defn ")`. Final floor is the HOLD above.
