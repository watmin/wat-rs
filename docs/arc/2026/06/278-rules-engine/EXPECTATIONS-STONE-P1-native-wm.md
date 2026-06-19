# EXPECTATIONS — Stone P1: native working-memory + transient/freeze boundary

Independent scorecard, fixed BEFORE the strike. Weigh the round-trip losslessness + the internal-only
discipline hardest. (P1's "probe" is an in-crate round-trip unit test, not a wat probe — the converters are
sealed Rust.)

| # | what | command | expected |
|---|---|---|---|
| 1 | round-trip identity | `cargo test --release -p wat --lib rete::kernel 2>&1 \| grep "test result"` | **GREEN** (round-trip a fired session + an empty-memory session → identity) |
| 2 | oracle intact (north star) | `cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 \| grep "test result"` | 1/1 |
| 3 | rete chain intact | `…5b_collect_rules / …4c_retraction / …3b_hash_join -- --include-ignored` | 4/4 · 4/4 · 4/4 |
| 4 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931+N / 36 (N new kernel tests; the 36 pre-existing, UNCHANGED) |
| 5 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 (UNCHANGED) |
| 6 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 7 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings (the known 25 stand) |

## Trap-doors named — weigh hardest

- **Round-trip is the whole point — and `==` is the judge.** `to_persistent(to_transient(s))` must equal `s`
  by `Value` `PartialEq`. Two failure modes: (a) a memory's per-node `Vec` order flips (PV→Vec→PV must
  preserve order — `pv.iter().cloned().collect()` then `VectorSync` push in the same order); (b) a passthrough
  field (network/rules/facts/next-id) gets reshaped instead of carried verbatim. Read the converters: the 4
  non-memory fields must be byte-for-byte the same `Value`.
- **Declaration order on rebuild.** `to_persistent` must place struct_form as
  `[network, rules, alpha, beta, production, facts, next-id]` — the exact Session record order
  (`wat/rete.wat:124-131`). A swap passes round-trip-of-empty but corrupts a populated session — the fired-
  session case (memories non-empty) is the canary.
- **Internal only — no wat leak.** Confirm NO new `:wat::rete::*` dispatch arm, NO TypeScheme, NO wat-callable
  transient/mutation. `grep` the diff for `dispatch`/`register(`/`":wat::` additions — there should be none.
  P1 is pure `src/rete/kernel.rs` + the `mod` line.
- **No firing snuck in.** The diff must contain no propagation/match/join logic — just the rep + 2 converters
  + the test. If `eval_alpha_match`/cross-join/production logic appears, that's P2 scope creep.
- **Empty-memory case.** A freshly-`compile`d session has empty memory maps; round-trip must yield an empty
  `PersistentMap`, not drop the field or produce `nil`. Probe both fired (populated) and compiled (empty).
- **TypeMismatch, not panic.** A non-Session value, or a malformed memory key/value, → `RuntimeError`, never
  an `unwrap`/panic in the converters.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-7 myself; 2-7 EXACTLY baseline + N new kernel tests green.
2. Read `to_transient`/`to_persistent` line by line: the 7-field read in order, the 3 PM→HashMap conversions
   (key/value shapes), the 4 passthroughs verbatim, the rebuild in declaration order, the error paths (no
   panic). Confirm the `WorkingMemory` is the flat `node-id → Vec` mirror (NOT keyed by join-bindings — that's
   P3).
3. `git diff --stat` — only `src/rete/kernel.rs` + `src/rete/mod.rs`. No wat, no oracle, no dispatch/TypeScheme.
4. Commit SCOPED on green; push. (Then P2: Rust fire-once on the WorkingMemory, differential-tested vs the oracle.)
