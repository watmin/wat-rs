# EXPECTATIONS — Stone 3a: `RootJoinNode` seeding

Independent scorecard, fixed BEFORE the strike. Orchestrator re-runs each row + reads the diff.

| # | what | command | expected |
|---|---|---|---|
| 1 | root-join seeds Tokens into beta-memory | `cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored` | **3/3 GREEN** |
| 2 | alpha pass still green (fire-rules extended, not broken) | `cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored` | 3/3 |
| 3 | 1a still constructs (Token changed) | `cargo test --release -p wat --test probe_arc278_1a_data_model -- --include-ignored` | 1/1 |
| 4 | compile | `cargo test --release -p wat --test probe_arc278_1b_compile -- --include-ignored` | 2/2 |
| 5 | matcher | `cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored` | 3/3 |
| 6 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 7 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 |
| 8 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 |
| 9 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings |

## Trap-doors named (weigh hard)
- **Root-join SEED only.** `fire-rules` must extend the existing alpha pass with token-seeding ONLY — no
  HashJoinNode cross, no `join-bindings` keying, no production firing (those are 3b/4). Confirm the diff adds
  beta-memory writes for RootJoinNodes and nothing else; production-memory still passes through unchanged.
- **Tuple, not vec.** `Token.matches` entries must be `(:wat::core::Tuple fact alpha-id)` typed
  `(:wat::Record, :wat::core::i64)`. A vec-in-vec (the original sketch) is the error this stone fixes — reject it.
- **Bindings carried, not recomputed.** The seeded Token's bindings === the Element's bindings (root-join adds
  none). Probe row "?t=25" is the canary.
- **Support chain length 1.** One condition → one tuple in matches. (Grows to N as joins chain in 3b.)
- **Edges actually used.** The seeding must follow `AlphaNode/children` → RootJoinNode (the edges 1b wired) —
  not a reverse scan or a guess. (If the worker can't reach root-joins via the edges, that's a real finding.)
- **1a Token construction.** Grep Token build sites; row 3 (1a) must stay green after the matches retype.

## Weigh (orchestrator)
1. Re-run rows 1-9 myself; rows 7/8 EXACTLY baseline.
2. Read the diff: Token.matches = PV of tuples; fire-rules seeds root-joins only (no hash-join/production);
   beta-memory flat `node-id → [Token]`; lint-clean (fixture untouched).
3. Commit SCOPED on green; push.
