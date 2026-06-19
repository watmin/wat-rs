# BRIEF — Stone P9: hot-path allocation reduction (close the fan-out-40k Clara gap)

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** RUST,
`src/rete/kernel.rs` + small `src/rete/matcher.rs` helper-signature changes. Build, run the named tests, report
verbatim. Another agent weighs. **Behavior-preserving** — the differential gates are the net; do NOT change
observable results. No RED probe (the nets already exist + are green).

## Why
The `temperare` + `struere` perf spells found the rete fire hot path is allocation-bound: a no-GC Rust engine
loses to a JVM RETE at 40k fan-out tokens (143ms vs 96ms) because we over-allocate per-token / per-derived-fact.
This stone kills the confirmed waste. The bench: `wat-scripts/perf/matrix/fanout-join.wat` (`echo '[100 20]' | …`).

## Apply these (CONFIRMED findings — implement in this order; re-run the differential after EACH group)

### Group A — constant-string Arcs (the #1 clean kill)
`make_element` (`kernel.rs:~304`) and `make_token` (`~313`) do `Arc::new("wat::rete::Element"/"Token".into())`
per instance. Hoist to module-level `static` via `once_cell::sync::Lazy<Arc<String>>` (confirm `once_cell` is a
dep; if not, use `std::sync::OnceLock<Arc<String>>`); clone the Arc (pointer bump) instead of allocating. Same
for `build_insert_fact` (`matcher.rs:~449`) `Arc::new(<type-name>.to_string())` — cache per type-name in a
`OnceLock<Mutex<HashMap<String,Arc<String>>>>` OR leave it if interning adds complexity (note which).

### Group B — hoist per-round-invariant lookups OUT of the `fire_fixpoint_delta` round loop
- **field_names** (`kernel.rs:~1076-1086`): build `field_names_cache: HashMap<String, Vec<String>>` (fact-class
  → field names) ONCE before `loop {`; inside, `field_names_cache.get(fact_class)`.
- **rule_by_name** (`~1298`): build `HashMap<String, Value>` (rule-name → rule) once before the loop; replace
  the `rules.iter().find(...)` per production per round with an O(1) lookup.
- **alpha_id constant** (`extend_token` / its callers): hoist `Value::i64(alpha_id)` out of the per-token loop.
- (Optional, if clean) prod-node static info (`~1285`): precompute `Vec<(prod_id, parent_id, Vec<WatAST> rhs)>`
  once; iterate it instead of re-extracting per round.

### Group C — borrow instead of clone (needs borrow-checker care; keep the clone at any site that won't compile cleanly, and NOTE it)
- `element_fact_bindings` (`kernel.rs:~327`) + `token_matches_bindings` (`~342`): return BORROWS
  (`(&Value, &rpds::HashTrieMapSync<…>)` and `(&rpds::VectorSync<…>, &rpds::HashTrieMapSync<…>)`); callers that
  only read (key computation) use the borrow; `extend_token` clones on entry (it needs to own). This removes the
  per-match map clone (40k × bucket).
- `dl`/`dr` (`~1206-1207`): borrow `d_beta.get(node_id).map(Vec::as_slice)` instead of `.cloned()` — the reads
  (steps 3/4) end before the mutation (step 6); restructure the borrow scope so it compiles.
- `node.clone()` per node per round (`~1105/1118/1154/1165/1286` + `alpha_pass:446`): `get_node` returns
  `&Value` — drop the `.clone()`, use the ref via `node_record` (network borrow vs alpha/beta mutation are
  separate fields).
- `jk` (`~1203`): `&join_keys_cache[child_id]` instead of `.clone()`.
- move-not-clone the double-push (`~1096/1131/1277`): `push(x)` (move) into one collection, `x.clone()` into the
  other — not both clones.

### Group D — extend_token binding merge (CORRECT idempotent-skip only)
`extend_token` (`kernel.rs:~532`) folds `el_bindings` into `tok_bindings` with `insert(k.clone(), v.clone())`
per key. The shared join-keys are already present (idempotent); the element's OWN vars are new and MUST be
added. Optimize ONLY by skipping keys already present with an equal value (`if new_bindings.get(k) != Some(v)`),
keeping the new-var inserts. **Do NOT skip the merge entirely** (that drops the element's new vars — a
correctness bug).

## REJECTED — do NOT do these (correctness / scope)
- ❌ Skip the `extend_token` merge entirely (drops new bound vars).
- ❌ `seen: HashSet<u64>` storing hashes (collision → a real fact silently dropped). Keep `HashSet<Value>`.
- ❌ Change `Token.matches` from rpds `VectorSync` to `Vec`, or change the support-`Tuple` representation — both
  are architectural, OUT OF SCOPE (banked).
- ❌ Any change to the wat oracle (`wat/rete.wat`), the public surface, or observable query results.

## STOP triggers
1. A needed primitive missing → STOP, name it.
2. A borrow restructure won't compile cleanly → KEEP the clone at that site, note it, move on (do NOT reach for
   `unsafe` or `RefCell` to force it).
3. Any differential goes red → STOP; you changed behavior. Revert that change; the rejects above are rejected
   for this reason.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_deep_cascade -- --include-ignored                 # 2/2 (native==wat==closure, depth 10+20)
cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored          # 4/4
cargo test --release -p wat --test probe_arc278_P4c_native_retraction                               # 3/3
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored             # 4/4 (keyed_join/fire-once' intact)
for t in 2b_insert_alpha 3a_root_join 3b_hash_join 4a_production_fire 4b_cascade 4c_retraction 5a_defrule_query; do cargo test --release -p wat --test probe_arc278_$t -- --include-ignored 2>&1 | grep "test result"; done   # all green
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result          # 1/1
cargo test --release -p wat --lib 2>&1 | grep "test result"                                          # 935/36 (the 36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                           # 264/1 (UNCHANGED)
cargo build --release 2>&1 | tail -2                                                                  # Finished; no NEW warnings
```
Report: each group's edits (the code), every test result verbatim, any borrow site you left cloned + why, any
STOP hit. No git. (The orchestrator runs the fan-out + Clara benches to measure the win.)

## Blast radius
`src/rete/kernel.rs` (the groups above) + `src/rete/matcher.rs` (`element_fact_bindings`/`token_matches_bindings`
return-type → borrows if done; `build_insert_fact` const-Arc). NO oracle change, NO public-surface change, NO
`WorkingMemory` stored-shape change, NO Token/support repr change. No git.
