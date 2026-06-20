# EXPECTATIONS — Stone 7-a: negation (`:not`) in the oracle

Independent scorecard, fixed before the strike. Weighed against the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe greens | `cargo test --release -p wat --test probe_arc278_7a_negation_oracle` | **3 passed; 0 failed** |
| 2 | `where` not regressed (filter-pass unification) | `cargo test --release -p wat --test probe_arc278_6b_ii_a_where_oracle \| grep "test result"` | 5 / 0 |
| 3 | north-star intact (no filters) | `cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy -- --include-ignored \| grep "test result"` | 1 / 0 |
| 4 | lib floor | `cargo test --release -p wat --lib -- --test-threads=1 \| grep "test result"` | 941 / 36 |
| 5 | deftest floor (rete.wat compiles + wat suite) | `cargo test --release --test test \| grep "test result"` | 264 / 1 |
| 6 | build clean | `cargo build --release` | builds; ≤ 26 warnings (the pre-existing baseline) |

## Load-bearing assertions (weighed by eye, against the disk)

- **#1 passes_when_absent (1)** + **blocks_when_present_matching (0)** — negation actually negates: a token
  passes iff no matching fact; not "never fires" (HEAD) and not "always fires" (a no-op node).
- **#1 passes_when_present_different_binding (1)** — the SHARED-VAR join-filter: Maintenance at Bergen does
  NOT block Oslo. This proves the negation uses `token-element-compatible?` (the ?loc agreement), not a
  blanket "any fact of this type exists" check.

## Diff integrity (read, don't trust the report)

- `git diff --stat` shows ONLY `wat/rete.wat` (+ the probe, already committed). NO Rust.
- Read by eye: a `NegationNode` record + `Node` variant + `node-children` arm; a `:not`-branch in
  `compile-condition` (the `where`-branch + the alpha+join path both intact); the test-pass generalized
  into a kind-dispatching `filter-pass` in `fire-once` (TestNode + NegationNode), folding in topological
  order. The `render-dag` compound-concat fixture is UNCHANGED.

## Runtime prediction

12–20 min (a real wat strike: a new node + compile branch + the filter-pass unification). Trap-doors:
(a) `find-or-mint-alpha` on the inner negated condition (STOP-1); (b) the filter-pass generalization must
not regress `where` — assertion #2 is the guard (STOP-2); (c) reusing `token-element-compatible?` inverted,
and threading alpha-memory into the filter-pass (the negation filter needs it).
