# BRIEF — Stone P2: Rust `fire-once` on the WorkingMemory + the differential harness

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A RUST
stone (`src/rete/kernel.rs` grows + small `matcher.rs` exposure/refactor + dispatch/TypeScheme). Build, run the
named tests, report verbatim. Another agent weighs.

## The work
Port the wat oracle's single-pass `fire-once` (alpha → root-join → hash-join → production) into Rust, operating
on the P1 `WorkingMemory`, behind a new primitive `(:wat::rete::fire-once' <session>) -> :wat::rete::Session`.
It must be **observationally equivalent** to the wat `fire-once`: same derived facts. Still
re-run-from-scratch / O(N²) — NO keyed joins (P3), NO fixpoint/delta (P4). This is the differential harness +
removes P1's dead-code allows by using `to_transient`/`to_persistent`.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P2-native-fire-once.md` — the four passes, the reuse plan,
   Element/Token construction, node-kind dispatch, the differential contract (OBSERVABLE — derived facts, NOT
   raw Session), out-of-scope.
2. `wat/rete.wat:883-935` — `fire-once` (the algorithm: 4 folds over node-ids). Then the helpers you port
   1:1: `activate-alpha`/`activate-fact` (`:489-537`), `root-join-pass`/`seed-root-join-children`/`seed-token`/
   `append-token` (`:544-621`), `hash-join-pass`/`cross-join-node`/`token-element-compatible?`/`extend-token`/
   `alpha-feeding` (`:629-770`), `production-pass`/`fire-production`/`node-parent`/`rule-by-name` (`:779-881`).
   Also `node-kind-label` (`:139`) and the record defs `Element`/`Token`/`AlphaNode`/`RootJoinNode`/
   `HashJoinNode`/`ProductionNode`/`Rule` (`:28-102`).
3. `src/rete/kernel.rs` — `WorkingMemory` + `to_transient`/`to_persistent` (P1, what you mutate/return).
4. `src/rete/matcher.rs` — `alpha_match_inner` (`:157`), `resolve_operand` (`:325`), `read_fact_field`
   (`:519`), `fact_from_value` (`:59`), `eval_insert` (`:390`). And `src/rete/collect.rs` + `src/runtime.rs:3998`
   + `src/check.rs:18900` for the dispatch-arm + TypeScheme registration pattern.
5. `tests/probe_arc278_P2_native_fire_once.rs` — the differential contract (already live, RED). Do not modify it.

## Reuse vs reimplement (DESIGN §reuse)
- **Expose `pub(crate)` in matcher.rs** (currently private): `alpha_match_inner`, `resolve_operand`,
  `read_fact_field`, `fact_from_value`. Fire-once calls them directly.
- **Extract `build_insert_fact`** from `eval_insert`: a `pub(crate) fn build_insert_fact(insert_form: &WatAST,
  bindings: &rpds::HashTrieMapSync<Value,Value>) -> Result<Value, EvalBreak>` holding the form-validate +
  `resolve_operand` loop + `wat__Record` build. `eval_insert`'s dispatch entry then evaluates its two args and
  calls `build_insert_fact`. The production pass calls `build_insert_fact` directly (it already has the form +
  bindings). Behavior of `eval_insert` must NOT change (the 4a/5a probes stay green).
- **Reimplement (native, mirroring the wat helpers exactly)** in `src/rete/kernel.rs`: the four passes over the
  `WorkingMemory` HashMaps; `alpha-feeding`/`node-parent` reverse-lookups; `token-element-compatible?` (fold
  element.bindings, shared-var agreement); `extend-token` (conj support tuple + merge bindings).

## Mechanics
- **Element** = `Value::wat__Record { class_fqdn: Arc::new("wat::rete::Element".into()), struct_form:
  Arc::new(vec![fact, Value::wat__core__PersistentMap(bindings)]) }`. **Token** = same with class
  `"wat::rete::Token"`, struct_form `[Value::wat__core__PersistentVector(matches), Value::wat__core__PersistentMap(bindings)]`.
  Support entry = `Value::Tuple(Arc::new(vec![fact, Value::i64(alpha_id)]))` (confirm the `Tuple` variant shape
  in `value/`). Read Element/Token/node fields by `struct_form.as_slice()[i]` positionally.
- **node kind**: from the node record's `class_fqdn` last `::` segment (mirror `node-kind-label`). Read
  `id`/`children`/`tests`/`rule-name` positionally from the node's struct_form.
- **alpha condition**: an `AlphaNode`'s `tests` is `PV<WatAST>`; the single test is `tests[0]` — pass it to
  `alpha_match_inner` with the fact's class + fields + the registry field-names (see how `eval_alpha_match`
  derives `field_names` from `sym.types()`; you have `sym` in the dispatch entry — thread it into the kernel).
- **bindings keys**: `Value::String("?loc")` etc. (same as the matcher). Merge/compat exactly as the wat helpers.
- **ascending node-id order** for the hash-join pass (sort the keys) — compile assigns ids topologically.
- Remove the `#[allow(dead_code)]` on the P1 `WorkingMemory`/converters now that fire-once' uses them.

## Builder directive: build missing deps, never hack around
Deps exist (the matcher cores, rpds, the Value variants, to_transient/to_persistent). **If a primitive is
genuinely missing → STOP + name it.** Do NOT change the wat oracle. Do NOT let the WorkingMemory escape to wat.

## STOP triggers
1. A needed primitive is missing → STOP, name it.
2. You reach for keyed joins / a fixpoint / delta / retraction / the public `fire` / a bench → P3–P5; STOP.
3. The differential fails and the fix would change the wat ORACLE → STOP (the oracle is the reference; the
   native impl conforms to it, never the reverse).
4. Build the passes IN ORDER (alpha → root → hash → production); if a pass can't be made to match the oracle,
   STOP and describe which pass + the divergence (do not guess).

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored   # 4/4 GREEN (native == wat)
cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored      # 4/4 (eval_insert refactor safe)
cargo test --release -p wat --test probe_arc278_5a_defrule_query -- --include-ignored        # 4/4
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result   # 1/1
cargo test --release -p wat --test probe_arc278_3b_hash_join -- --include-ignored             # 4/4
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                             # kernel/matcher unit tests green
cargo test --release --test test_stdlib_load_order | grep result                            # 1/0
cargo test --release -p wat --lib 2>&1 | grep "test result"                                 # 935/36 (+ any new kernel tests; 36 unchanged)
cargo test --release --test test 2>&1 | grep "test result"                                  # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                         # Finished; no NEW warnings (P1 allows now removed by use)
```
Report: the Rust fire-once + the four pass fns + the Element/Token builders + `build_insert_fact` + the matcher
`pub(crate)` changes + the dispatch arm + TypeScheme; all outputs verbatim; any STOP hit. No git.

## Blast radius
`src/rete/kernel.rs` (fire-once' + passes), `src/rete/matcher.rs` (4 `pub(crate)` + extract `build_insert_fact`),
`src/runtime.rs` (1 dispatch arm), `src/check.rs` (1 TypeScheme), `tests/probe_arc278_P2_native_fire_once.rs`
(already live). NO wat changes. NO oracle change. No git.
