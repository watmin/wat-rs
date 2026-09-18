# DESIGN — Stone P2: Rust `fire-once` on the WorkingMemory + the differential harness

The keystone of the perf close. Port the wat oracle's single-pass `fire-once` (alpha → root-join → hash-join →
production) into Rust, operating on the P1 `WorkingMemory`, and stand up the **differential harness**: the
native fire-once must produce the **same derived facts** as the wat oracle's `fire-once`. Still
re-run-from-scratch / O(N²) (keyed joins are P3) — P2 is about *correct + native + differentially-proven*, the
net under P3/P4. This also removes P1's `#[allow(dead_code)]` by *using* `to_transient`/`to_persistent`.

## The differential contract (the important correction)
NOT raw `Session` equality. P3 will restructure the memories (flat `node-id→[…]` → keyed
`node-id→{join-bindings→[…]}`), so the internal memories *stop* matching the oracle by design. The durable,
P3/P4-stable contract is **observable equivalence**: for every input session,
`query(native-fire-once(s), T) ≡ query(wat-fire-once(s), T)` as a multiset of derived facts (same count, same
records) for each derived type T. The internal `WorkingMemory` layout is implementation.

## What P2 delivers (Rust — `src/rete/kernel/` grows)

A new primitive **`(:wat::rete::fire-once' <session>) -> :wat::rete::Session`** (bring-up name — the native
single-pass; P5 promotes the *looped* version to the public `:wat::rete::fire`). It:
1. `to_transient(session)` → `WorkingMemory` (P1).
2. Runs the four passes, mutating the native `HashMap` memories (faithful 1:1 port of the wat algorithm —
   `wat/rete.wat:883-935` + the pass helpers, quoted in the BRIEF):
   - **Alpha** (`activate-alpha`): for each `AlphaNode`, test every fact against its single condition via
     `alpha_match_inner` → on `Some(bindings)`, push an `Element` Value to `alpha[alpha-id]`.
   - **Root-join** (`root-join-pass`/`seed-token`): for each `AlphaNode`'s Elements, seed one `Token`
     (`matches=[Tuple(fact, alpha-id)]`, `bindings=Element.bindings`) into each `RootJoinNode` child's `beta`.
   - **Hash-join** (`hash-join-pass`/`cross-join-node`): in ascending node-id order, for each Root/HashJoin
     node's tokens, cross against the feeding alpha's Elements (`alpha-feeding` reverse-lookup), keep
     compatible pairs (`token-element-compatible?` = shared-var agreement), `extend-token` (conj the support
     tuple, merge bindings) → push to the `HashJoinNode` child's `beta`.
   - **Production** (`production-pass`/`fire-production`): for each `ProductionNode`, read its parent's
     (`node-parent` reverse-lookup) `beta` tokens; for each token × each rhs insert-form, build the derived
     fact and push to `production[prod-id]`.
3. `to_persistent(wm)` → the fired `Session` Value.

### Reuse vs reimplement (grounded)
- **Reuse (expose `pub(crate)` in matcher.rs):** `alpha_match_inner`, `resolve_operand`, `read_fact_field`,
  `fact_from_value`. These are the pure matching/operand cores — fire-once calls them directly.
- **Extract + reuse:** `eval_insert`'s inner. Today `eval_insert(args, env, sym)` evals the two arg *forms*;
  the production pass already HAS the insert-form (`&WatAST` from `rule.rhs`) and the bindings (a
  `HashTrieMapSync`), so split out `build_insert_fact(insert_form: &WatAST, bindings: &HashTrieMapSync<Value,Value>)
  -> Result<Value, EvalBreak>` (the form-validate + `resolve_operand` + `wat__Record` build), and have the
  dispatch entry call it after evaluating its args. Fire-once calls `build_insert_fact` directly.
- **Reimplement (native, mirroring the wat helpers):** the four passes + `alpha-feeding`/`node-parent`
  reverse-lookups + `token-element-compatible?` + `extend-token`, as Rust over the `WorkingMemory` HashMaps.
- **Build Element/Token Values directly:** `Value::wat__Record { class_fqdn: Arc::new("wat::rete::Element"|
  "wat::rete::Token"), struct_form: Arc::new(vec![...]) }`; support entry = `Value::Tuple(vec![fact, Value::i64(alpha_id)])`.
  (Read node records the same way — `struct_form.as_slice()` positional; node kind via the class_fqdn last segment.)

### Node-kind dispatch
The network is `node-id → Node` (a `Value::wat__Record` of the variant). Determine kind from the record's
`class_fqdn` last segment ("AlphaNode"/"RootJoinNode"/"HashJoinNode"/"ProductionNode") — mirror
`node-kind-label` (`wat/rete.wat:139`). Read `children`/`tests`/`rule-name`/`id` positionally from struct_form.

## The one contract decision (pinned)
`fire-once'` is observationally equivalent to wat `fire-once`: same derived facts (`query` multiset) for every
input. Registered like `eval-insert` (dispatch arm + TypeScheme `[:wat::rete::Session] -> :wat::rete::Session`).
Internal mutation stays sealed (the `WorkingMemory` never escapes; only a frozen `Session` returns).

## Files touched
- `src/rete/kernel/` — the four-pass Rust fire-once + Element/Token builders + node-kind helpers; remove the
  `#[allow(dead_code)]` on the P1 seam (now used).
- `src/rete/matcher.rs` — `pub(crate)` on `alpha_match_inner`/`resolve_operand`/`read_fact_field`/
  `fact_from_value`; extract `build_insert_fact` inner from `eval_insert`.
- `src/runtime.rs` — one dispatch arm (`:wat::rete::fire-once'`).
- `src/check.rs` — one TypeScheme.
- `tests/probe_arc278_P2_native_fire_once.rs` — the differential probe (native == wat, observable).

## The differential probe
Mirror the 3b/4a setup. For cold-and-windy: `collect-rules :weather` → compile → insert Temp(Oslo,15) +
Wind(<loc>,45) → run BOTH `(fire-once' s)` and `(fire-once s)` → compare `(:wat::core::length (query · :weather::ColdAndWindy))`:
- Oslo: both **1** (+ assert the native-derived fact is `ColdAndWindy` at "Oslo").
- Bergen (mismatched loc): both **0**.
- 2×2 (2 Temps × 2 Winds / 2 locs): both **2** (no cross-loc leakage in the native join either).
RED at HEAD: `fire-once'` is `UnknownFunction`.

## Out of scope = REJECTED
- **Keyed joins** (`join-bindings` map keying / the O(N²)→O(match) win) — P3.
- **The fixpoint / delta / cascade / retraction** — P4 (P2 is single-pass, matching wat `fire-once`, not
  `fire-rules`).
- **The public `:wat::rete::fire` + the bench** — P5.
- No change to the wat oracle (it stays the reference). No keyed memory. No mutation escaping to wat.
