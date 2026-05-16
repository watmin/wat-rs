# Arc 200 Slice 1 SCORE

**Date:** 2026-05-16
**Slice:** 1 (single slice; closes arc 200)
**Branch:** main (HEAD pre-commit at `64cc793`)
**Status:** PASS — 5/5 rows YES

## Scoring

| Row | What | YES/NO | Evidence |
|-----|------|--------|----------|
| A | Gap 1 fixed — `splice_argument` accepts `WatAST::Vector` | YES | `src/macros.rs` — new arm `WatAST::Vector(items, _) => Ok(items.clone())` mirrors the List arm; positive test `splice_of_vector_bound_symbol_succeeds` (calls `(:my::splice-vec [10 20 30])`) returns `Vec[i64(10), i64(20), i64(30)]` instead of erroring with `SpliceNotList`. |
| B | Gap 2 fixed — `walk_template` Vector branch dispatches unquote-splicing | YES | `src/macros.rs` — Vector branch now mirrors the List branch's splice-dispatch (fire at depth 1, preserve+peel at depth > 1); positive test `splice_inside_vector_template_fires` builds a `fn`-sig via `[~@params]` and the resulting function computes `7 + 35 = 42`. |
| C | D2 probes flipped from expected-failure to expected-success | YES | `tests/probe_stone_d2_splice_vector.rs` rewritten as regression tests (3 positive cases). File rename was blocked by harness permissions; original filename preserved and module doc explicitly explains the history + concept-anchored search terms. |
| D | D1 still passes; positive vector-splice test passes | YES | `cargo test --release -p wat --test wat_run_threads_d1` → 1/1 pass; `cargo test --release -p wat --test probe_stone_d2_splice_vector` → 3/3 pass (Gap 1, Gap 2, combined). |
| E | Workspace test failure count ≤ baseline (4) | YES | `cargo test --release --workspace --no-fail-fast` → 4 failures total, all matching the documented baseline at commit `64cc793`: `lifeline_pipe_zero_orphans_across_100_trials` (lifeline flake), `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`. No new failures. |

## Honest deltas

- **Mirror compiled cleanly.** Both relaxations were near-mechanical. Gap 1 was a one-arm addition; Gap 2 was a paste of the List branch's splice-dispatch loop into the Vector branch with `WatAST::Vector(...)` reconstruction. No surprises, no urge to refactor — the optional helper extraction would have muddied the diff for negligible gain (a ~25-line shared loop body whose only difference is the constructor at the end). Skipped the refactor per the BRIEF's explicit guidance.
- **File rename blocked by harness permissions.** `git mv` and `mv` both denied; preserved the original filename `probe_stone_d2_splice_vector.rs` and updated the module doc to explain the lineage and surface the concept-anchored search terms (`vector splice`, `arc 200 regression`) inline. Test names also carry the concept (`splice_of_vector_bound_symbol_succeeds`, `splice_inside_vector_template_fires`, `vector_splice_round_trip_matches_list_splice`). Future agents searching for vector splice work hit the file via name OR header OR test names.
- **Gap 2 test design surfaced an arc-167 cascade.** Naively mirroring the original Gap 2 probe (splice into `[...]` at value position) hit arc 167's "vectors at value position not supported" runtime error — that is the correct cascade per DESIGN.md § Out of scope (Gap 3). To isolate the macro-layer fix from arc 167, the Gap 2 positive test splices into a `fn` signature (consumed at expand time). This is honest scope discipline: the macro-layer fix is proven; the runtime-layer Gap 3 stays out of scope.
- **Type-syntax friction on the first draft.** Initial test used legacy `:fn(...)` and Rust-style `:T,:T` argument prefixes; substrate diagnostics correctly directed me to `:wat::core::Fn(T,T)->R` with bare argument types per `feedback_wat_colon_quote`. Substrate diagnostics paid the cost; one-shot correction landed the third test.
- **Workspace baseline preserved exactly.** Same 4 failures, same names, same modules. No regression.

## Concrete changes

| File | Change | Lines |
|------|--------|-------|
| `src/macros.rs` | Gap 1 — added `WatAST::Vector` arm to `splice_argument` | +6 (incl. comment) |
| `src/macros.rs` | Gap 2 — mirrored List branch's splice-dispatch loop into Vector branch of `walk_template` | +37 (incl. comment expansion) |
| `tests/probe_stone_d2_splice_vector.rs` | Rewrote as arc 200 regression with 3 positive tests; updated module doc to record the flip | full rewrite (~170 lines) |

## What the slice proved

The macro substrate now treats `WatAST::Vector` and `WatAST::List` uniformly for unquote-splicing in both directions:
- Splice INPUT (Gap 1): `~@xs` fires whether `xs` is bound to a List or a Vector at the call site.
- Splice CONTEXT (Gap 2): `[~@xs]` template fires the splice the same way `(~@xs)` template does.

The combined regression `vector_splice_round_trip_matches_list_splice` proves the two paths produce identical runtime values — the substrate is now Lisper-uniform on this dimension.

## Closure

Arc 200 closes on this slice. The D2 stone unblocks: the `(:wat::kernel::run-threads [[:I :O f] ...] client-fn)` call shape now expands cleanly through both relaxations. Arc 167's Gap 3 (vectors at value position) remains a separate concern surfaced by the DESIGN but explicitly out of scope; the macro-layer fix did not require touching it.
