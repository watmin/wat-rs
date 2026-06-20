# BRIEF — Stone P11: native token (cheap support chain), `src/rete/kernel.rs` only

Single-hop **sonnet** Shadowdancer in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** RUST.
Behavior-preserving — the differential suite is the net; do NOT change observable results. Another agent weighs
your work against its own re-run + the diff, then benches. Build, run the named tests, report VERBATIM.

## Read FIRST (in order)
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-P11-native-ephemeral-token.md` — the full contract, the
   property-graph framing, why beta can be ephemeral, the three acceptance gates, out-of-scope.
2. `src/rete/kernel.rs` — `WorkingMemory` (`:36-46`, `beta:44`), `make_token` (`:327`), `make_element` (`:317`),
   `element_fact_bindings` (`:342`), `token_matches_bindings` (`:359`), `to_transient` (`:140`), `to_persistent`
   (`:205`), `root_join_pass` (`:454`, support build `:488-490`), `extend_token` (`:550`, support `:557-558`),
   `keyed_join` (`:578`), `hash_join_pass` (`:645`), `production_pass` (`:719`, token read ~`:769-780`),
   `fire_once_session` (`:797`), `fire_fixpoint_delta` (`:1015`, final freeze `:1413`).

## The work
Replace the per-token `Value` machinery with a cheap native struct, and drop the ephemeral tokens at freeze.

1. **Define** `struct Token { matches: Vec<(Value, i64)>, bindings: rpds::HashTrieMapSync<Value, Value> }`
   (the `(fact, alpha_id)` pairs ARE the property-graph's condition-labeled edges — KEEP them; they are
   non-negotiable, the operator diagnostic depends on them).
2. **`WorkingMemory.beta`**: `HashMap<i64, Vec<Value>>` → `HashMap<i64, Vec<Token>>`.
3. **Converters stay lossless** (round-trip contract): `to_transient` reads a Session's `beta-memory` (Value
   Token Records: `struct_form = [PV<Tuple(fact,i64)>, PM bindings]`) → native `Token`; `to_persistent` writes
   native `Token` → the same Value Token Record shape. A Value Tuple `[fact, i64]` ↔ a `(Value, i64)` pair; the
   `PM` bindings unwrap/wrap `Value::wat__core__PersistentMap`. Confirm `round_trip_*` tests stay green.
4. **The passes operate on native `Token`**: `root_join_pass` seeds `Token { matches: vec![(fact.clone(),
   node_id)], bindings: el_bindings.clone() }` (drop the `Tuple`/`VectorSync`/`make_token`); `extend_token` takes
   + returns native `Token`, pushes `(el_fact.clone(), alpha_id)` onto a cloned `matches` Vec and folds
   `el_bindings` into `bindings` (idempotent-skip kept); `keyed_join` builds native tokens; `hash_join_pass`
   stores them; `production_pass` reads `tok.bindings` (still `HashTrieMapSync`) → `build_insert_fact` UNCHANGED.
5. **Drop beta at freeze (the win)**: in `fire_once_session` AND `fire_fixpoint_delta`, add `wm.beta.clear();`
   immediately BEFORE the final `to_persistent(wm)` call. The derived facts live in production-memory; beta is
   intermediate and not read from a fired Session (see DESIGN "why beta can be ephemeral").
6. **Add the guiding-light probe** (a `#[test]` in the `rete::kernel` tests module): run `to_transient` + the
   four passes (NOT the clear-at-freeze path) on a small 2-condition cascade, reach into `wm.beta`, and assert a
   production-reaching token's `matches` has the expected `(fact, alpha_id)` edges (2 for a 2-condition rule,
   the facts being the input facts). This proves the cheap repr keeps the chain walkable.

## STOP triggers (halt + surface, do NOT improvise)
1. Any differential goes RED → you changed observable behavior. STOP, report which.
2. The support chain (`matches`) ends up empty or lossy for a production-reaching token → guiding-light breach,
   same severity as a differential RED. STOP.
3. A test asserts a fired Session's `beta` is non-empty (clearing beta would break it) → STOP, name the test.
4. You reach to change the wat oracle (`wat/rete.wat`), a `Value` variant, `matcher.rs`
   `resolve_operand`/`build_insert_fact`, the Element representation, or to build the fact→token walk index
   (that is the NEXT stone) → out of scope. STOP.

## Verify (run each; paste VERBATIM)
```
cargo build --release 2>&1 | tail -3
cargo test --release -p wat --lib rete 2>&1 | grep "test result"                                  # incl round-trip + your new probe
cargo test --release -p wat --test probe_arc278_deep_cascade -- --include-ignored 2>&1 | grep result   # 2/2
cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules -- --include-ignored 2>&1 | grep result  # 4/4
cargo test --release -p wat --test probe_arc278_P2_native_fire_once -- --include-ignored 2>&1 | grep result    # 4/4
cargo test --release -p wat --test probe_arc278_P4c_native_retraction 2>&1 | grep result           # 3/3
cargo test --release -p wat --test probe_arc278_northstar_cold_and_windy 2>&1 | grep result         # 1/1
for t in 2b_insert_alpha 3a_root_join 3b_hash_join 4a_production_fire 4b_cascade 5a_defrule_query; do cargo test --release -p wat --test probe_arc278_$t -- --include-ignored 2>&1 | grep "test result"; done
cargo test --release -p wat --lib 2>&1 | grep "test result"                                        # 935-ish/36 (the 36 pre-existing UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                         # 264/1 UNCHANGED
```
Report: the `Token` struct + the edits to each pass + the converters + the two `wm.beta.clear()` sites + your
guiding-light probe; every test result verbatim; any STOP hit. Do NOT bench (orchestrator-only). No git.

## Blast radius
`src/rete/kernel.rs` only (the struct, the converters, the four passes, `keyed_join`/`extend_token`, the two
clear sites, the new probe). NO oracle, NO `Value`, NO `matcher.rs`, NO Element change, NO walk-index. No git.
