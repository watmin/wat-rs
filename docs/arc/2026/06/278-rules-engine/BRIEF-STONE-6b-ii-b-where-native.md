# BRIEF — Stone 6b-ii-b: `where`/TestNode in the NATIVE kernel + the differential

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

The wat ORACLE already filters tokens through TestNodes (6b-ii-a, `wat/rete.wat`); the compile step (shared)
already mints them. The NATIVE delta engine (`src/rete/kernel.rs`) does NOT yet honor a TestNode, so it
under-derives → the differential (native==oracle) is RED. Add a **native test-pass**: for each `TestNode`,
filter `wm.beta[parent]` by `eval-test(expr, token.bindings)` into `wm.beta[test-id]`, placed AFTER the
hash-join pass and BEFORE the production pass — mirroring the oracle's `test-pass`. The differential probe
(`fire-rules` native == `fire-rules-spec` oracle, for a rule with a `where`) is the gate. Contract:
`DESIGN-STONE-6b-where-test.md` (6b-ii-b entry).

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-6b-where-test.md` — the 6b-ii-b entry.
2. **`wat/rete.wat` `test-pass`** (just shipped, ~`:1020`) + the `where`-branch in `compile-condition` — the
   ORACLE model the native must match. The native test-pass is its 1:1 port over the native `WorkingMemory`.
3. `src/rete/matcher.rs` `eval_test` (~`:869`, 6b-i) — **extract a core**:
   `pub(crate) fn eval_test_core(expr: &WatAST, bindings: &rpds::HashTrieMapSync<Value,Value>, env: &Environment, sym: &SymbolTable) -> Result<bool, EvalBreak>`
   (build the child env from `bindings`, `eval_inner`, require `Value::bool`). `eval_test` (the dispatch
   wrapper) then extracts its two args and calls the core. The native test-pass calls the core directly with
   the TestNode's `expr`, the Token's bindings, and a fresh `&Environment::new()` (a where sees only its
   `?vars` + `sym`'s user fns — no caller scope).
4. `src/rete/kernel.rs` `node_kind_label` (`:409`) — add the `"TestNode"` arm (map the TestNode class fqdn →
   `"TestNode"`). Confirm `node_children`/`kind_of` then handle it.
5. `src/rete/kernel.rs` `fire_fixpoint_delta` (`:1204`) — THE production path (`fire-rules'` → this). The
   hash-join pass ends ~`:1430`, production pass ~`:1546`. Insert the test filter between them: for each
   TestNode id, read `wm.beta[node_parent(test)]`, keep tokens where `eval_test_core(expr, &tok.bindings,
   &Environment::new(), sym)?` is true, write to `wm.beta[test-id]`. Model the token read/write on how
   `hash_join_pass`/`production_pass` touch `wm.beta`.
6. `src/rete/kernel.rs` `Token` (`:152`/`:551`) — how a native Token's `bindings` (`HashTrieMapSync`) are
   read, and `make_token`/`extend_token` — how tokens are stored in `wm.beta`.
7. `tests/probe_arc278_6b_ii_b_where_native_differential.rs` — the 4 assertions to green (do NOT edit it).

## Scope / correctness note (v1)

- The native test-pass may filter the FULL `wm.beta[parent]` each round (a non-incremental TestNode) — that
  is CORRECT (differential-green); a delta-incremental TestNode is a perf follow-on (banked `6b-perf`).
- REQUIRED in `fire_fixpoint_delta` (the path `fire-rules'`/`fire-rules` uses — the differential gates it).
  If `fire_once_session`/`fire_fixpoint` (the non-delta references) are reachable by any test, mirror the
  filter there too; if they are `#[allow(dead_code)]` references, a one-line comment that TestNode handling
  lives in the delta path is acceptable — do NOT expand the blast radius chasing dead refs.

## Blast radius (bounded)

- `src/rete/matcher.rs` (extract `eval_test_core`; `eval_test` delegates to it) + `src/rete/kernel.rs`
  (`node_kind_label` arm + the test-pass in `fire_fixpoint_delta`). NO `wat/rete.wat` (oracle already done),
  NO `runtime.rs`/`check.rs` (eval-test already dispatched), NO `purity.rs`. No new `Value` variant.

## STOP triggers (halt + surface; do not improvise)

1. If the native fire cannot construct an `Environment` or read a Token's `bindings` as a `HashTrieMapSync`
   to call `eval_test_core` — STOP, report what the native Token/fire actually exposes.
2. If inserting the test filter into `fire_fixpoint_delta` requires restructuring the delta round loop (not
   just a filter pass between hash-join and production) — STOP, describe the delta-flow obstacle.
3. If the differential stays RED because native ≠ oracle in a way you cannot localize — STOP, report the
   native vs oracle counts + your hypothesis; do NOT weaken the probe.
4. If greening needs editing `wat/rete.wat` (the oracle is the reference — it must not move) — STOP.

## Done = green

`cargo test --release -p wat --test probe_arc278_6b_ii_b_where_native_differential` → 4/4 (the differential
holds: native == oracle, both 1 on pass, both 0 on block). Then the floors (EXPECTATIONS).
