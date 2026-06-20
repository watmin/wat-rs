# EXPECTATIONS — Stone 6a: purity inference

Independent scorecard, fixed before the strike. Weighed against the orchestrator's *own* re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe greens | `cargo test --release -p wat --test probe_arc278_6a_purity` | **9 passed; 0 failed** |
| 2 | lib floor holds | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | 941 passed / 36 failed (unchanged) |
| 3 | deftest floor holds | `cargo test --release --test test \| grep "test result"` | 264 / 1 (unchanged) |
| 4 | deporder floor holds | `cargo test --release --test test_stdlib_load_order \| grep result` | 1 / 0 |
| 5 | build clean | `cargo build --release` | builds; warnings ≤ 25 (no NEW warnings from `purity.rs`) |

## Load-bearing assertions (weighed by eye in the kill, against the disk)

- **#4 `Uuid/v4` → false** — the non-determinism case default-allow misses.
- **#4b `Uuid/v5` → true** — the v4/v5 boundary; the fence denies *randomness*, not the Uuid family.
- **#6 transitive impurity → false** — `is_pure_fn` recurses the user-fn body; the transitive hole is closed.
- **#7 unknown head → false** — DEFAULT-DENY (the architectural choice; if this is green via default-allow the stone is wrong).

## Diff integrity (read, don't trust the report)

- `git diff --stat` shows ONLY: `src/rete/purity.rs` (new), `src/rete/mod.rs` (+1 line), `src/runtime.rs` (one `pub(crate)` + one dispatch arm), `src/check.rs` (one TypeScheme), `tests/probe_arc278_6a_purity.rs` (the probe, already committed STRIKE-READY).
- **Read `src/runtime.rs`'s diff by eye** — confirm `is_effectful_op` changed *only* in visibility and the dispatch arm is the *only* other addition (megafile forced-minimal discipline).
- Confirm the allow-list in `purity.rs` against the `dispatch_keyword_head_value` table: every listed head is a real, deterministic, effect-free op; `Uuid/v4` is NOT on it.

## Runtime prediction

5–15 min build sonnet. Trap-doors: (a) user-fn body extraction from `sym` may not match `step_user_call`'s shape → STOP-1; (b) the allow-list may be over- or under-inclusive — I weigh it against the dispatch table, not the sonnet's say-so; (c) `quote` sub-forms must be treated as data (not recursed as calls) or a quoted impure form inside a pure expr would wrongly taint it.
