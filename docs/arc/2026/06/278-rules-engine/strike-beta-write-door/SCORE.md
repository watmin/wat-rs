# SCORE — D3 beta-write door

The four bypasses are gone. The bypass is a compile error. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ four bypasses gone | **HOLD.** `beta_written` in `src/rete/` lives only in `census.rs` (decl) and `session.rs` `BetaStore::record_token` / `record_tokens`. |
| 2 ★ compile error | **HOLD.** `wm.beta.map.len()` in `hash_join.rs` → `error[E0616]: field \`map\` of struct \`session::BetaStore\` is private`. Reverted after capture. |
| 3 ★ doc claim now TRUE | **HOLD.** `record_token`/`record_tokens` take `&mut BetaStore`; the inner `map` is private. A future site cannot `entry().push`. |
| 4 ★ census numbers | **HOLD.** `fanout_cost` / `cascade_cost` / `node_share_cost` / `pass_semantics` all green (27/27). `left_activate_join` now uses the door's `reserve`+`extend_from_slice`; no cost gate moved. |
| 5 round reset | **HOLD.** `BetaStore::clear` is the named door. `delta.rs` start-of-fire and drop-memories call it. No `&mut BetaMemory` escapes. |
| 6 floor | **HOLD.** `Summary [ 453.094s] 5437 tests run: 5437 passed (1 slow), 21 skipped`. `.floor/2026-09-05T20-36-05Z/`. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 8 blast | **HOLD.** `src/rete/kernel/` only (5 files). |

## Doors

| door | why it is safe |
|---|---|
| `BetaStore::record_token` / `record_tokens` | census + durable write, one act |
| `BetaStore::clear` | empties the map; not a token write; fire start, drop-memories, test scratch |
| `BetaStore::push_ref` (`#[cfg(test)]`) | full-recompute twins `root_join_pass` / `hash_join_pass` have no `d_beta`; they are not the shipping path |

The field on `FireSession` stays public so callers split-borrow `wm.beta` from `wm.alpha` / `wm.bind_pool`. The **map** is private. Same shape as `JoinRightIndex`.

## Still open (named, cut)

The existing census gate still cannot reach these four sites: no `:where` in any census world. The bypass cannot compile, so the gate does not need to see them. Adding a `:where` to a census world is still CUT.
