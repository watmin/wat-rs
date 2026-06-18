# EXPECTATIONS — Stone 2b: `insert` + `fire-rules` (alpha slice)

Independent scorecard, fixed BEFORE the strike. The orchestrator re-runs each row + reads the diff; the
worker's report is a hypothesis until the disk confirms it.

| # | what | command | expected |
|---|---|---|---|
| 1 | insert + fire-alpha populate alpha-memory correctly | `cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored` | **3/3 GREEN** |
| 2 | 1a still constructs + renders (Element changed) | `cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored` | 1/1 |
| 3 | compile still green | `cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored` | 2/2 |
| 4 | matcher still green | `cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored` | 3/3 |
| 5 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 6 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 |
| 7 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 |
| 8 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings (baseline 25) |

## Runtime prediction
~25-40 min. Two pure WAT fns + the Element field change + updating Element construction sites. The nested
fold (alphas × facts) is the substance; the alpha-match call + Element build are mechanical.

## Trap-doors named (weigh these hard)
- **Zero activation in `insert`.** `insert` must ONLY stage (conj onto facts) — no alpha-match, no memory
  write. If `insert` touches alpha-memory, reject (violates "zero activation until fire-rules").
- **`fire-rules` is the alpha slice ONLY.** No beta-memory writes, no production firing, no derived facts, no
  loop. Confirm the diff doesn't sneak in beta/production (that's stones 3/4).
- **Constraint honored, not just type.** Probe row 1 test 2 (only 25, not 15) is the canary — fire must store
  an Element only when the FULL alpha-match (incl `> 20`) succeeds.
- **Element.fact = record everywhere.** Grep for any remaining `Element` site passing a map; the 1a probe
  (row 2) must still build/render. A missed site → compile error or a red 1a.
- **Purity / value-semantics.** `insert`/`fire-rules` return NEW Sessions; no mutation. (WAT records are
  immutable, so this is structural — but confirm no surprise.)
- **No hack-around.** If the worker reports a STOP (missing dep), that's correct — do NOT accept an
  improvised workaround in its place; build the dep.

## Weigh (orchestrator, against own re-run + the diff)
1. Re-run rows 1-8 myself; rows 6/7 EXACTLY baseline.
2. Read the `rete.wat` diff: `Element.fact` is `:wat::Record`; `insert` stages only; `fire-rules` does alpha
   only (no beta/production); alpha-memory is `node-id → [Element]`; lint-clean (fixture untouched).
3. Dogfood-read for any sneaked beta/production/cascade.
4. Commit SCOPED on green; push.
