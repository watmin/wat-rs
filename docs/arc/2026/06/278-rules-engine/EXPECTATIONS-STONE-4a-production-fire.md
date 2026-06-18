# EXPECTATIONS — Stone 4a: production-fire (token → RHS → derived fact)

Independent scorecard, fixed BEFORE the strike. Orchestrator re-runs each row + reads the diff. The rule
finally FIRES — weigh the RHS-eval purity + the production-pass shape hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | the rule fires + derives correctly | `cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored` | **4/4 GREEN** (1 fires, 1 fact-shape, 1 no-fire guard, 1 no-leakage 2×2) |
| 2 | hash-join still green | `cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored` | 4/4 |
| 3 | root-join still green | `cargo test --release -p wat --test probe_arc278_3a_root_join -- --include-ignored` | 3/3 |
| 4 | alpha pass still green | `cargo test --release -p wat --test probe_arc278_2b_insert_alpha -- --include-ignored` | 3/3 |
| 5 | matcher / data model / compile | `…2a_alpha_match / …1a_data_model / …1b_compile -- --include-ignored` | 3/3 · 1/1 · 2/2 |
| 6 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 7 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 (UNCHANGED) |
| 8 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 (UNCHANGED) |
| 9 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings |

## Trap-doors named — weigh hardest

- **RHS-eval is PURE (the headline).** `eval_insert` must resolve fact-args ONLY via `resolve_operand`
  (`?var`→bindings, literal→Value) — NO `eval_inner`, NO `Environment`-driven evaluation of the fact-arg, NO
  `macro_eval`. Read the body: the only `eval_inner` calls allowed are the 2 mandatory arg-evals (the form and
  the bindings-map), exactly as `eval_alpha_match` does. If a fact-arg flows through `eval_inner` → wrong (the
  surface grew past 4a; it would silently admit non-pure RHS).
- **`resolve_operand` REUSED, not reimplemented.** A second copy of operand resolution inside `eval_insert` is a
  divergence class — confirm it calls the existing `resolve_operand` with empty fact-fields/names.
- **Parent reverse-lookup is kind-agnostic.** `node-parent` must find the parent via `node-children-ids`
  (works for a `RootJoinNode` parent on a 1-condition rule AND a `HashJoinNode` parent on cold-and-windy). A
  lookup hard-coded to `HashJoinNode` would pass the 2-condition probe but silently break 1-condition rules.
  Reason about a hypothetical 1-condition rule even though the probe is 2-condition.
- **One fact per activation — no clobber, no cross.** The 2×2 row asserts EXACTLY 2 (not 4 = a blind cross,
  not 1 = an accumulator that overwrites instead of `conj`-ing). The `production-memory` accumulation must
  thread through the fold (each `conj` returns a new map; a dropped intermediate = lost facts).
- **No re-entry / no cascade.** Derived facts go into `production-memory` ONLY — they must NOT be appended to
  `Session.facts` and must NOT re-run the network. If the derived `ColdAndWindy` re-enters and tries to match a
  rule, that is 4b — out of scope; confirm `facts` is threaded through unchanged.
- **Flat `production-memory`.** `production-memory[P-id]` is a flat `PV<:wat::Record>` in 4a. NOT the
  `{token → [facts]}` support map (that is 4c). Confirm the `:121` comment was adjusted to say so.
- **Registration mirrors 2a exactly.** One dispatch arm (`runtime.rs`), one TypeScheme (`check.rs`,
  `params: [:wat::WatAST, :wat::core::PersistentMap]`, `ret: :wat::Record`), one `mod.rs` note. A missing
  TypeScheme → the checker can't type `eval-insert` call sites (but the probe doesn't call it directly, so
  watch for a silently-unregistered primitive that only the WAT pass exercises).
- **No scope creep.** No `Snapshot`, no `query`, no `defrule`/`collect-rules`, no nested-pure-expr fact-args,
  no record-field validation, no 1a/1b/2/3 record change.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-9 myself; 7/8 EXACTLY baseline (only row 1 flips RED→GREEN).
2. Read `eval_insert` line by line: the 2 arg-evals (form + bindings), the insert-form validation, the
   `resolve_operand` reuse with empty fact-fields, the `wat__Record` build, the error paths (no panic, no silent
   drop). Confirm NO third `eval_inner` touches a fact-arg.
3. Read the WAT production pass: `node-parent` (kind-agnostic via `node-children-ids`), `rule-by-name`,
   `fire-production` (beta-memory[parent] tokens → per-token per-insert-form `eval-insert` → `conj` into
   production-memory), the fold threading in `fire-rules`, and that `facts` passes through unchanged.
4. Mentally run a 1-condition rule → confirm `node-parent` finds the RootJoinNode (the probe only covers
   2-condition; if the code looks risky, add a quick check).
5. Confirm the `render-dag` compound-concat FIXTURE is untouched.
6. Commit SCOPED on green; push.
