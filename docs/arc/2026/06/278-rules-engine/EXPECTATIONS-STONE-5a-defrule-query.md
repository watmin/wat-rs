# EXPECTATIONS — Stone 5a: `defrule` (rule macro) + `query`

Independent scorecard, fixed BEFORE the strike. Weigh the macro expansion (correct quoting of conditions) and
the query type-normalization hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | defrule + query (4 cases) | `cargo test --release -p wat --test probe_arc278_5a_defrule_query -- --include-ignored` | **4/4 GREEN** (query reads/empty; defrule Rule shape; defrule fires end-to-end) |
| 2 | retraction still green | `cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored` | 4/4 |
| 3 | cascade / production-fire | `…4b_cascade / …4a_production_fire -- --include-ignored` | 4/4 · 4/4 |
| 4 | join / matcher / compile | `…3b_hash_join / …2a_alpha_match / …1b_compile -- --include-ignored` | 4/4 · 3/3 · 2/2 |
| 5 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 6 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 (UNCHANGED) |
| 7 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 (UNCHANGED) |
| 8 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; 25 warnings (NO new — pure WAT) |

## Trap-doors named — weigh hardest

- **The macro quotes conditions, doesn't evaluate them.** The expansion must wrap each `:when` condition and
  each `:then` form in `(:wat::core::quote …)` so they land in `Rule.lhs`/`Rule.rhs` as `WatAST` data — NOT as
  evaluated calls. If a condition `(:weather::Temperature (?loc <- :location) …)` is emitted un-quoted, the
  checker reads it as a Temperature constructor call (the exact RED the probe shows at HEAD:
  `Temperature expected 2 got 3`). Probe `defrule_rule_fires_end_to_end` green proves the quoting is right
  (the rule compiles + fires). Read the expanded macro output if unsure (`write-forms` / a macroexpand check).
- **`Rule.name` = fqdn WITHOUT colon.** `:weather::cold-and-windy` → `"weather::cold-and-windy"` (probe asserts
  this). A leftover leading `:` is wrong (it must match `(:wat::core::type fact)`'s convention for downstream).
- **`query` normalizes the type keyword the same way.** `:weather::ColdAndWindy` → `"weather::ColdAndWindy"`
  (strip leading `:` from `keyword/to-string` iff present), compared against `(:wat::core::type f)`. A mismatch
  (colon vs no-colon) silently returns 0 — probe `query_reads_derived_facts_by_type` (== 1) is the canary.
- **`query` flattens ALL production nodes.** It must `foldl` over `PersistentMap/values` of production-memory
  (a derived fact can live under any ProductionNode's id), not read a single node. Empty PV if the type isn't
  derived (probe `query_empty_for_absent_type` == 0) — never raise.
- **`:then` is variadic (N inserts).** v1 north-star has one, but the macro must collect ALL forms after
  `:then` into the rhs vector (a 2-insert rule would lose inserts otherwise). The probe only covers 1 — reason
  about N by reading the parse.
- **Load order stays 1/0.** `defrule` is a stdlib macro in `rete.wat`; its expansion references `:wat::rete::Rule`
  + `:wat::core::{defn,quote,PersistentVector}` — all available at rete.wat's load point. If the macro pulls in
  something load-ordered later, `test_stdlib_load_order` goes RED.
- **No scope creep.** No `collect-rules`, no Rust, no `defquery`/`QueryNode`, no `Snapshot`.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-8 myself; 7/8 EXACTLY baseline (only row 1 flips RED→GREEN).
2. Read the `defrule` expansion (the macro source + ideally a macroexpand of the cold-and-windy form): confirm
   each condition + insert is `quote`-wrapped, name is colon-stripped, the `defn` returns `:wat::rete::Rule`.
3. Read `query`: type normalization (colon strip), flatten-all-production-nodes, filter by `=` on `type`.
4. Confirm `render-dag` fixture untouched; no Rust in the diff.
5. Commit SCOPED on green; push. (Then 5b: `collect-rules` → north-star.)
