# EXPECTATIONS — Stone 6b-ii-a: `where`/TestNode in the oracle + fence

Independent scorecard, fixed before the strike. Weighed against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe greens | `cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle` | **5 passed; 0 failed** |
| 2 | north-star still green (no where = unchanged) | `cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored \| grep "test result"` | 1 / 0 |
| 3 | 6a + 6b-i probes still green | `cargo test --release -p wat --test probe_arc278_6a_purity` ; `… --test probe_arc278_6b_eval_test` | 19/0 ; 7/0 |
| 4 | lib floor holds | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | 941 / 36 (unchanged) |
| 5 | deftest floor (rete.wat compiles + the wat suite) | `cargo test --release --test test \| grep "test result"` | 264 / 1 |
| 6 | build clean | `cargo build --release` | builds; warnings ≤ 25 |

## Load-bearing assertions (weighed by eye, against the disk)

- **#1 where_passes (Temp 5 → 1 Gate)** + **#2 where_blocks (Temp -5 → 0)** — the filter actually filters;
  not "rule never fires" (the HEAD failure) and not "rule always fires" (a no-op TestNode).
- **#3 user-fn predicate passes/blocks** — the TestNode carries an arbitrary pure∧det expr through the
  network and `eval-test` reaches a user fn at fire.
- **#4 fence rejects impure where at compile** — confirm the error is the *fence* raising at compile
  (a `pure?∧det?` failure), NOT some unrelated parse/type error. Read the error kind.

## Diff integrity (read, don't trust the report)

- `git diff --stat` shows ONLY `wat/rete.wat` (+ the probe, already committed). NO Rust files touched.
- Read the `wat/rete.wat` diff by eye: a `TestNode` record + `Node` variant; a `where`-branch at the top of
  `compile-condition` (the existing alpha+join path unchanged below it); a test-pass fold added to
  `fire-once` between hash-join and production. The `render-dag` compound-concat fixture is UNCHANGED.

## Runtime prediction

10–20 min (a real wat strike across compile + fire). Trap-doors: (a) the WatAST child accessor for
extracting `<expr>` from `(where <expr>)` (STOP-1); (b) calling `eval-test`/`pure?` with a WatAST *value*
not a quote (STOP-2); (c) `node-parent` reverse-lookup reuse for the test-pass's parent read; (d) the
test-pass fold slotting cleanly into `fire-once` (STOP-3).
