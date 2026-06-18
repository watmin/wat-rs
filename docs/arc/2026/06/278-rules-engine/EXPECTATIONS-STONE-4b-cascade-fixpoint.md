# EXPECTATIONS — Stone 4b: cascade-to-fixpoint

Independent scorecard, fixed BEFORE the strike. Orchestrator re-runs each row + reads the diff. Weigh the
fixpoint termination + the re-run-from-scratch shape (not incremental) hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | the cascade fires + terminates | `cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored` | **4/4 GREEN** (B fires on A's output; A still 1; total 2; no-root → 0) |
| 2 | single-rule fire still green | `cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored` | 4/4 |
| 3 | hash-join still green | `cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored` | 4/4 |
| 4 | root-join / alpha still green | `…3a_root_join / …2b_insert_alpha -- --include-ignored` | 3/3 · 3/3 |
| 5 | matcher / data model / compile | `…2a_alpha_match / …1a_data_model / …1b_compile -- --include-ignored` | 3/3 · 1/1 · 2/2 |
| 6 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 7 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 (UNCHANGED) |
| 8 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 (UNCHANGED) |
| 9 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; 25 warnings (NO new — pure WAT, no Rust) |

## Trap-doors named — weigh hardest

- **Termination is the headline.** The fixpoint MUST stop. The guard is `merge-facts`'s dedup: `conj` only
  facts not already in the set (`PersistentVector/contains?` before `conj`), and `fire-rules` recurses ONLY
  when `length(new-facts) > length(old-facts)`. If `merge-facts` appends without the contains? guard, `facts`
  grows every round (re-derived facts re-added) → infinite loop / hang. Row 1's `no_cascade_without_the_root_fact`
  and `fixpoint_total_is_exactly_two_derived` both probe termination (a hang shows as a timeout/no-return).
- **No cross-round inflation.** `fire-once` recomputes `production-memory` from empty each round, so the FINAL
  `production-memory` holds each derived fact exactly once (the last round's derivations). The `total == 2` row
  asserts this. If the code ACCUMULATES production-memory across rounds (instead of recomputing), you'd see
  ColdAndWindy derived N times → total > 2. Confirm `fire-once` is the clean per-round recompute.
- **`fire-once` is a VERBATIM extraction.** It must be exactly today's `fire-rules` body (the 4 passes), no
  behavior change. Read the diff: the extracted `fire-once` should be byte-equivalent to the old `fire-rules`
  body modulo the name; all the 4a/3b/3a/2b behavior is preserved (rows 2-5 prove it).
- **Re-run-from-scratch, NOT incremental.** Confirm the driver calls `fire-once` again on an enlarged fact set
  — it does NOT splice derived facts/tokens into existing alpha/beta memories. (Incremental is the deferred
  perf path.)
- **The recursion reconstructs facts correctly.** The recursive call passes a Session with `facts = new-facts`;
  `network`/`rules`/`next-id` carried through; memories may be anything (recomputed). A driver that recurses
  with the OLD facts (forgetting to thread new-facts) would loop forever OR never cascade.
- **The 1-condition rule path (rule B).** Rule B has a single condition `(:weather::ColdAndWindy (?loc <-
  :location))` → its ProductionNode's parent is a RootJoinNode (4a's `node-parent` must find it). Row 1 proves
  4a's kind-agnostic parent lookup works for the 1-condition case (which 4a's own probe didn't cover).
- **No scope creep.** No TM/retraction, no support store, no `Snapshot`, no `query`/`defrule`, no incremental,
  no Rust change, no arbitrary round cap.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-9 myself; 8/8 EXACTLY baseline (only row 1 flips RED→GREEN).
2. Read the diff: `fire-once` is a verbatim extraction of the old body; `collect-derived` flattens
   `PersistentMap/values`; `merge-facts` has the contains?-before-conj dedup; `fire-rules` recurses on the
   enlarged fact set with the length-equality fixpoint check.
3. Reason about termination explicitly: facts is monotone-growing, dedup-bounded, finite domain → stops. Confirm
   no path appends without dedup.
4. Confirm `render-dag` compound-concat FIXTURE untouched; no Rust files in the diff.
5. Commit SCOPED on green; push.
