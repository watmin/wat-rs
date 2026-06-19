# EXPECTATIONS — Stone P2: Rust `fire-once` + the differential harness

Independent scorecard, fixed BEFORE the strike. Weigh the differential (native == wat derived facts) and the
faithful-port + no-oracle-change discipline hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | differential: native == wat | `cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored` | **4/4 GREEN** (Oslo: both 1 + right fact; Bergen: both 0; 2×2: both 2) |
| 2 | eval_insert refactor safe | `…4a_production_fire / …5a_defrule_query -- --include-ignored` | 4/4 · 4/4 |
| 3 | oracle + north star intact | `…northstar_cold_and_windy / …3b_hash_join / …4c_retraction -- --include-ignored` | 1/1 · 4/4 · 4/4 |
| 4 | rete unit tests | `cargo test --release -p wat --lib rete 2>&1 \| grep "test result"` | green (kernel round-trip + any new) |
| 5 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 935/36 (+ any new kernel tests; the 36 pre-existing UNCHANGED) |
| 6 | deftest / load order | `…--test test / test_stdlib_load_order` | 264/1 · 1/0 |
| 7 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings (P1's `#[allow(dead_code)]` REMOVED — the seam is now used) |

## Trap-doors named — weigh hardest

- **The differential is the whole point.** Native `fire-once'` and wat `fire-once` must derive the SAME facts.
  Compare on the OBSERVABLE (`query` count + content), NOT raw memories. Row 1's three cases (match / no-match
  / 2×2) each assert `native == wat` AND the absolute. If native diverges from wat on ANY, a pass is mis-ported.
- **The 2×2 is the join canary.** Exactly 2 (same-loc joins), not 4 (cross-loc leakage) and not 0 (broken
  compat). The native `token-element-compatible?` must be shared-var agreement (a var on only one side never
  conflicts), and `extend-token` must merge bindings + conj the support tuple — same as the wat helper.
- **The oracle is NEVER touched.** The wat `fire-once`/`fire-rules`/helpers must be byte-unchanged — it's the
  reference. `git diff wat/rete.wat` must be EMPTY. If the native impl needed an oracle change to match, the
  native impl is wrong.
- **`eval_insert` behavior preserved.** The `build_insert_fact` extraction must leave `eval_insert`'s dispatch
  behavior identical — rows 2 (4a/5a) are the canary. Read the diff: the entry evals its two args then calls
  the inner; the inner is the old body.
- **`#[allow(dead_code)]` comes OFF.** P1's seam is now used by fire-once' → the allows are removed (not left
  rotting). If they're still there, the seam isn't actually wired. Build clean with NO new warnings either way.
- **Internal mutation sealed.** The `WorkingMemory` never returns to wat; only a frozen `Session` (via
  `to_persistent`). No new wat-callable mutation/transient op. `grep` the diff for unexpected `:wat::` additions
  beyond the single `:wat::rete::fire-once'` dispatch arm + TypeScheme.
- **No scope creep.** No keyed joins, no fixpoint/delta/retraction, no public `fire`, no bench. Single pass only
  (matches wat `fire-once`, NOT `fire-rules`).

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-7 myself; 2-7 EXACTLY baseline (+ the new kernel tests); only row 1 flips RED→GREEN.
2. `git diff wat/rete.wat` → EMPTY (oracle untouched). `git diff --stat` → only kernel.rs, matcher.rs,
   runtime.rs, check.rs, the probe.
3. Read the four passes line by line against the wat helpers: alpha (alpha_match_inner per AlphaNode test),
   root-join (seed-token shape), hash-join (alpha-feeding + compat + extend), production (node-parent +
   build_insert_fact). Confirm Element/Token built with the exact class_fqdn + struct_form shape.
4. Confirm `build_insert_fact` is a clean extraction (eval_insert entry = eval args → call inner).
5. Commit SCOPED on green; push. (Then P3: key the joins by join-bindings — the O(N²)→O(match) bend, differential still green.)
