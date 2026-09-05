# SCORE — F1 explain attribution order (after REVIEW)

The first SCORE's rune on `node-parents` was narrower than its use. Sorted that walk too; rune gone; Gate B now compares the derivation the token yields.

## Scorecard

| # | result |
|---|---|
| 1★ 2★ deterministic + agrees | **HOLD.** Unchanged: harvest-support walks `topological-node-ids`. |
| 3★ control | **HOLD.** Single-producer row still agrees. |
| 4★ Gate A | **HOLD, tighter.** Exemption list is **empty**. Exactly one `PersistentMap/keys network` in oracle code, inside the verb. A rune does not save a raw walk (`a_runed_walk_is_still_a_hit`). |
| 5★ five sites | **HOLD, revised.** `node-parents` now **calls the verb** (was runed). The other four still call it. |
| 6 one definition | **HOLD.** |
| 7 floor | **HOLD.** `Summary [ 433.929s] 5437 tests run: 5437 passed (1 slow), 21 skipped`. `.floor/2026-09-05T12-49-06Z/`. (+2 Gate B `:or` tests vs the previous floor.) |
| 8 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 9 blast | **HOLD.** Zero `src/`. |

## What changed in this pass

`pass.wat` `node-parents` iterates `topological-node-ids`. Parent-id VECTOR order is now topological, matching native `index_network_edges` (walks `sorted_node_ids`). `tokens-from-parents` is deterministic → `Support/token` is deterministic → the derivation tree is deterministic.

The rune is deleted. Gate A no longer has a carve-out.

Gate B extended: `:or` rule whose two arms (`:orx::Left` / `:orx::Right`) derive the same `:orx::Out`. Compares `via[0]` supporting-fact type, native == oracle. Left-only control. `or_two_arms_native_and_oracle_attribute_the_same_token` / `or_control_a_single_arm_agrees`.

Sorting `node-parents` changed no existing rete test (486/486, including the two new). STOP-1 did not fire.

## Not this

F2. `src/`. The four sites already on the verb.
