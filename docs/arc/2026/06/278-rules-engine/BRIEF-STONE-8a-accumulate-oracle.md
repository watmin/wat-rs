# BRIEF — Stone 8-a: the AccumulateNode in the ORACLE + apply-accumulator

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `./target/release/wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

Teach the wat oracle (`wat/rete.wat`) the accumulate condition `(?result <- (<acc-form>) :from (:FactType
<clause>…))`. An AccumulateNode is the JOIN family that **extends** the token with an aggregate: for each
parent token, gather the token-compatible `:from` elements (shared `?vars`, via `token-element-compatible?`),
fold them with `apply-accumulator` (over the 8-i `acc::*` folds), and — if a result — extend the token's
bindings with `?result` and pass it; if `None` (empty min/max/mean), drop the token. This is the ORACLE
(native + differential are 8-b). Contract: `DESIGN-STONE-8-accumulators.md` (the 8-a bullet + surface).

## Read in order (the rooms — all in `wat/rete.wat` unless noted)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-8-accumulators.md` — the surface (bound `?var` primitive)
   + the 8-a bullet.
2. The 8-i fold library `:wat::rete::acc::*` (`~:1491`+) — `apply-accumulator` dispatches to these. Note the
   return shapes: `count`/`sum`→`i64`, `distinct`→`PV`, `all`→`PV`, `group-by`→`PM` (bare); `min`/`max`/
   `mean`→`Option<i64>`.
3. `compile-condition` — the `where`-branch + the `:not`-branch (7-a) are the MODEL. Add an
   **accumulate-branch**: detect a `?`-symbol head (via `ast->children` + `ast-name` starts with `"?"`) +
   `<-` as items[1]. Extract: `result-var` (head's name string), `acc-form` (items[2]), assert items[3] is
   `:from`, `inner` (items[4]). `find-or-mint-alpha` for `inner` → `from-alpha-id`; mint an `AccumulateNode`
   carrying `result-var` + `acc-form` + `from-alpha-id`; wire `parent → accumulate` (parent must be ≥ 0 —
   an accumulate needs a left token; if `< 0` raise "accumulate must follow a binding condition"); advance
   parent = accumulate-id.
4. The node records + `Node` defenum + `node-children` arm — add `AccumulateNode`
   `[id <- :wat::core::i64  result-var <- :wat::core::String  acc-form <- :wat::WatAST  from-alpha-id <- :wat::core::i64  children <- :wat::core::PersistentVector<wat::core::i64>]`.
5. `token-element-compatible?` (`:805`) + the negation branch of `filter-pass` (7-a) — the MODEL for the
   gather (collect the compatible elements for a token). Accumulate gathers the SAME way, then folds.
6. `extend-token` / how a Token's bindings get a new key (the hash-join extends bindings) — the accumulate
   extends the token with `result-var → <aggregate Value>`. Keep the token's `matches` (per-element
   accumulate provenance is banked).
7. `fire-once` (`~:1200`+) — add an **accumulate-pass** fold over node-ids BETWEEN the hash-join pass and
   the `filter-pass` (so a `where` on `?result` sees the extended binding). Thread alpha-memory (for the
   gather) + beta-memory.

## apply-accumulator (new wat fn)

```
(:wat::rete::apply-accumulator [acc-form <- :wat::WatAST  els <- :wat::core::PersistentVector<wat::rete::Element>])
  -> :wat::core::Option<wat::core::Value>
```
Dispatch on `acc-form`'s head keyword (`ast->children` → first → `ast-name`):
- `:wat::rete::acc::count` → `(:wat::core::Some (:wat::rete::acc::count els))`  (wrap the bare i64 as Value)
- `:wat::rete::acc::sum`   → extract `?var` = (ast-name of items[1]); `(Some (acc::sum var els))`
- `min`/`max`/`mean`       → map their `Option<i64>` to `Option<Value>` (Some→Some, None→None) — these can drop the token
- `distinct`/`all`/`group-by` → `(Some (acc::… [var] els))`
- unknown head → raise "apply-accumulator: unknown accumulator"
The bare folds always yield `Some` (they always have a value); only `min`/`max`/`mean` can yield `None`.
The `Option<wat::core::Value>` return leans on STONE-Value (i64/PV/PM all up-cast to `Value`).

## Blast radius (bounded)

- `wat/rete.wat` ONLY. NO Rust (the folds + `token-element-compatible?` exist). Do NOT touch the
  `render-dag` compound-concat fixture. v1: accumulate runs after joins, before filters (a `where` on the
  result follows the accumulate — the natural order); a where-BEFORE-accumulate interleave is banked.

## STOP triggers (halt + surface; do not improvise)

1. If detecting the `?`-symbol-headed accumulate condition in `compile-condition` collides with another
   clause shape — STOP, report.
2. If `extend-token` (adding `result-var → Value` to a token's bindings) has no reusable helper — STOP,
   report how the hash-join extends bindings.
3. If `Option<wat::core::Value>` doesn't type-check (STONE-Value up-cast) — STOP, report.
4. If greening needs Rust / `kernel.rs` — STOP (that's 8-b).

## Done = green

`cargo test --release -p wat --test probe_arc278_8a_accumulate_oracle` → 5/5. AND `--test
probe_arc278_8i_accumulator_folds` → 10/10 (folds unregressed). AND `--test
probe_arc278_northstar_cold_and_windy -- --include-ignored` → 1/0. AND `--test
probe_arc278_7a_negation_oracle` → 3/3 (filter-pass unregressed). Then `cargo test --release --test test |
grep result` → 264/1.
