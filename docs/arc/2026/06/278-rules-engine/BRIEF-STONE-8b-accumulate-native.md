# BRIEF — Stone 8-b: the AccumulateNode in the NATIVE kernel + the differential

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

The oracle already accumulates (8-a, `wat/rete.wat`); compile (shared) mints AccumulateNodes. The NATIVE
delta engine (`src/rete/kernel.rs` `fire_fixpoint_delta`) has no accumulate-pass → the result-var is never
bound → the differential is RED. Add a **native accumulate-pass**: for each AccumulateNode, for each NEW
token at `d_beta[parent]`, gather the token-compatible elements from the FULL `wm.alpha[from_alpha_id]`,
compute the aggregate in **Rust** (mirroring the wat folds), and — if a result — extend the token's bindings
with `result-var → <aggregate Value>` and push to `wm.beta[acc]` + `d_beta[acc]`; `min`/`max`/`mean` on
empty → drop. The differential (`fire-rules` native == `fire-rules-spec` oracle) is the gate. Contract:
`DESIGN-STONE-8-accumulators.md`.

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-8-accumulators.md` — the contract.
2. **`wat/rete.wat` `accumulate-pass-for-token`** (8-a, ~`:1754`) — the ORACLE model your native port must
   MATCH (the per-fold dispatch + empty behavior: count/sum/distinct/all/group-by always emit; min/max/mean
   emit only Some). Mirror its values exactly.
3. **`src/rete/kernel.rs`**:
   - `node_kind_label` (`:409`) — add `"AccumulateNode"` arm.
   - `node_children` (`:442`) — add the AccumulateNode arm (children at the last slot — AccumulateNode is
     `id(0), result-var(1), acc-form(2), from-alpha-id(3), children(4)` → `sf[4]`).
   - `fire_fixpoint_delta` — the 7-b negation filter + the 6b-ii-b test filter (step 3.5) are the MODEL for
     the gather + delta integration. Add the **accumulate-pass** BEFORE the filter pass (so a `where` on the
     result sees the binding). For each AccumulateNode: extract `result-var` (`sf[1]`), `acc-form` (`sf[2]`,
     a `Value::wat__WatAST`), `from_alpha_id` (`sf[3]`). For each NEW token in `d_beta[parent]`, gather the
     `wm.alpha[from_alpha_id]` elements that are `token_element_compatible` (`:722`) with the token, compute
     the aggregate (below), and — if a value — push the EXTENDED token to `wm.beta[acc]` + `d_beta[acc]`.
   - `token_element_compatible` (`:722`) reuse; `element_fact_bindings` (`:534`) to read an element's
     `?var` value + its fact; how a native `Token` is built (its `bindings` + `matches`) — the extended
     token = same `matches`, `bindings` + `{result-var → aggregate}`.

## The native folds (match the wat folds + honest typing)

Dispatch on `acc-form`'s head keyword (it's a `WatAST::List`; head = `items[0]` keyword `.as_str()`; the
`?var` arg = `items[1]`'s symbol name for value-folds). Over the gathered elements:
- `acc::count` → `gathered.len() as i64` (always a value).
- `acc::sum`   → Σ of each element's `bindings[?var]` (i64); empty → 0 (always a value).
- `acc::min` / `acc::max` → fold with `<` / `>`; **empty → None (drop the token)**.
- `acc::mean` → `sum / count` (i64 div); **empty → None (drop)**.
- `acc::distinct` → dedup the `?var` values into a `PV` (always; empty → `[]`).
- `acc::all` → a `PV` of each element's fact (always; empty → `[]`).
- `acc::group-by` → a `PM` keyed by `?var` value → `PV<fact>` (always; empty → `{}`).
The aggregate is a `Value` (i64 / PV / PM) assoc'd into the token bindings (a `HashTrieMapSync<Value,Value>`).
NOTE: there is NO uniform `Option<Value>` — wat parametric types are invariant; just compute per-fold (this
mirrors the oracle's inline dispatch). In Rust you may use `Option<Value>` freely (Rust IS covariant) — the
constraint was only on the WAT side.

## Scope / v1

Re-accumulate (no retract-fn — replay). Gather the FULL `wm.alpha[from_alpha_id]` (the absence/aggregate
needs all matching facts, like 7-b negation), filter NEW tokens from `d_beta[parent]`. Accumulate before the
filter pass. Keep the token's `matches` (per-element accumulate provenance banked, `8-perf`).

## Blast radius (bounded)

- `src/rete/kernel.rs` ONLY (`node_kind_label` + `node_children` arms + the accumulate-pass + the Rust
  folds). NO `wat/rete.wat` (the oracle is the frozen reference). NO `matcher.rs`/`runtime.rs`/`check.rs`.

## STOP triggers (halt + surface; do not improvise)

1. If `token_element_compatible` / `element_fact_bindings` can't be reused for the gather + var-read — STOP, report.
2. If extending a native Token's bindings (assoc result-var → Value) has no clean path — STOP, report how
   the hash-join builds extended tokens.
3. If the differential stays RED (native ≠ oracle) and you can't localize it — STOP, report native vs oracle
   counts + hypothesis; do NOT weaken the probe.
4. If greening needs editing `wat/rete.wat` — STOP (the oracle is frozen).

## Done = green

`cargo test --release -p wat --test probe_arc278_8b_accumulate_native_differential` → 5/5 (native==oracle).
AND `--test probe_arc278_8a_accumulate_oracle` → 5/5 ; `--test probe_arc278_7b_negation_native_differential`
→ 4/4 ; `--test probe_arc278_6b_ii_b_where_native_differential` → 4/4 ; `--test
probe_arc278_northstar_cold_and_windy -- --include-ignored` → 1/0. Then `cargo build --release` clean +
`cargo test --release -p wat --lib -- --test-threads=1 | grep result` → 941/36.
