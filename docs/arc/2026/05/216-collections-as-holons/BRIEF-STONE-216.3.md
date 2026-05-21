# BRIEF — Arc 216 Stone 216.3 — HashMap round-trip

**Stone:** mint bidirectional round-trip for `HashMap<K, V>` through `HolonAST::Bundle` of arbitrary-K Binds. Capstone collection of arc 216; combines positional-Bind shape (216.2) with arbitrary K (arc 057 slice 3's `hashmap_key accepts HolonAST` machinery). Two type params instead of one — both K and V must atomize recursively.
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 105 min STOP.
**Depends on:** Stones 216.1 (`b478ff4`, HashSet), 216.2 (`e4a63ed`, Vector) — pattern templates established + nested-value support enables `HashMap<K, Vec<T>>`, `HashMap<K, HashSet<T>>`, etc.
**Unblocks:** Stone 216.4 (atomizable predicate consolidation — may be no-op if piecemeal landed all entries), Stone 216.5 (sandbox walker validation), Stone 216.6 (INSCRIPTION + closure).

## Goal

Extend `value_to_atom` for `Value::wat__std__HashMap(m)` → `HolonAST::Bundle(vec![Bind(K_holon, V_holon), Bind(K_holon, V_holon), ...])` (arbitrary K; not constrained to i64 positional). Mint reverse: `atom-value` extracts `HashMap<K, V>` from a Bundle of arbitrary-K Binds. Add `HolonRepresentable` impl for `HashMap<K, V>`. Verify `is_atomizable` predicate entry exists for `HashMap` (may be pre-landed per Stone 216.1/216.2 bonuses; flag if so).

Per DESIGN Q2: HashMap = "Bundle of Binds with any K type." Discriminator from Vector: Vec uses sequential i64 keys; HashMap uses anything else (or non-sequential i64 keys).

## Pre-flight verified

- Stone 216.1 shipped (`b478ff4`): value_to_atom HashSet arm; atom-value bare-atom path; HolonRepresentable for HashSet; is_atomizable HashSet entry; Vector entry pre-landed
- Stone 216.2 shipped (`e4a63ed`): value_to_atom Vec arm (positional-Bind Bundle); eval_atom_value Bundle shape-dispatch (Bind(I64)→Vec; bare-atoms→HashSet); holon_item_to_value extended for nested Vec<Vec<T>>; HolonRepresentable for Vec<T>; is_atomizable Vector entry already present
- arc 057 slice 3: `hashmap_key accepts HolonAST` — substrate primitive for arbitrary-K HashMap; this stone uses it directly for K_holon
- Baseline tests green (all 9 probe suites + 824 lib unit tests)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

1. **Extend `value_to_atom` for HashMap** in `src/runtime.rs`:
   - Add a match arm adjacent to Vec/HashSet arms: `Value::wat__std__HashMap(m) => HolonAST::bundle(m.iter().map(|(k, v)| Ok(HolonAST::bind(value_to_atom(k)?, value_to_atom(v)?))).collect::<Result<_, _>>()?)`
   - Note: HashMap iteration is unordered; the produced Bundle's Bind order is therefore non-canonical (HashMap → Bundle is order-lossy by definition; reverse trip reconstructs as a HashMap which is also order-agnostic)
   - K must already be atomizable (per is_atomizable predicate); if not, the value_to_atom call on k errors with the existing diagnostic

2. **Extend `eval_atom_value` Bundle shape-dispatch** for HashMap:
   - Stone 216.2 added Bind(I64) → Vec path with sequential-key validation
   - Stone 216.3 adds the THIRD path: arbitrary K Binds (not all I64 OR I64 keys not sequential) → HashMap
   - Discriminator: when consumer's `-> :T` annotation declares `T = :wat::core::HashMap<K, V>`, take the HashMap path regardless of bundle's actual shape (consumer-declared); validate each child is a Bind; extract K_holon → K via atom-value reverse; extract V_holon → V same; insert into HashMap
   - The atom-value's existing shape-dispatch (216.2's path) handles Vec vs Set; HashMap path is the new third arm activated by the consumer's expected type

3. **Extend `holon_item_to_value`** to handle Bind(K, V) for nested HashMap values (mirrors 216.2's Bind(I64) handling but for arbitrary K)

4. **Add HolonRepresentable trait impl** in `src/comms/mod.rs`:
   - Mirror 216.1/216.2 patterns
   - `impl<K, V> HolonRepresentable for HashMap<K, V> where K: HolonRepresentable + Hash + Eq + Send + 'static, V: HolonRepresentable + Send + 'static`
   - `to_holon_ast`: iterate map; produce Bundle of Binds
   - `from_holon_ast`: validate Bundle of Bind shape (any K); extract pairs; insert into HashMap

5. **Verify `is_atomizable` predicate entry for HashMap**:
   - Per Stones 216.1/216.2 SCORE deltas — bonus pre-landings may have already added HashMap entry
   - If exists: confirm via inspection; document in SCORE
   - If missing: add: `HashMap<K, V>` atomizable iff `K` atomizable AND `V` atomizable

6. **Probe matrix** — `tests/probe_arc216_stone3_hashmap_roundtrip.rs` with ~14 probes:
   - Probe 1: Forward — `(value_to_atom {:foo 42 :bar 99})` → Bundle of Binds with keyword K and i64 V
   - Probe 2: Reverse — `(atom-value <bundle> -> :wat::core::HashMap<:wat::core::keyword, :wat::core::i64>)` → HashMap with :foo→42, :bar→99
   - Probe 3: Empty map round-trip — `{}` → Bundle([]) → empty HashMap (consumer declares HashMap type to disambiguate from empty HashSet/Vec)
   - Probe 4: Multi-K types — HashMap<keyword, V>, HashMap<String, V>, HashMap<i64, V>, HashMap<bool, V>
   - Probe 5: Multi-V types — HashMap<K, i64>, HashMap<K, String>, HashMap<K, bool>, HashMap<K, keyword>
   - Probe 6: Non-keyword keys — HashMap<i64, String> round-trips (arbitrary K)
   - Probe 7: Nested map — `HashMap<keyword, HashMap<keyword, i64>>` round-trips
   - Probe 8: Mixed nesting — `HashMap<keyword, Vec<i64>>` round-trips (composes with 216.2)
   - Probe 9: Mixed nesting — `HashMap<keyword, HashSet<i64>>` round-trips (composes with 216.1)
   - Probe 10: Check passes — `(:wat::holon::Atom my-hashmap)` for atomizable K + V type-checks
   - Probe 11: Check fails — non-atomizable K or V fails at check; diagnostic
   - Probe 12: HolonRepresentable cascade — Rust compile-time check HashMap<String, i64>: HolonRepresentable
   - Probe 13: Shape disambiguation — Bundle of Bind(I64, V) with NON-sequential i64 keys (e.g., [Bind(I64(0), v), Bind(I64(5), v)]) → HashMap<i64, V> (NOT Vec; Stone 216.2's sequential-key check fails; falls through to HashMap)
   - Probe 14: Empty Bundle disambiguation — empty Bundle with consumer declaring HashMap → empty HashMap (overrides 216.2's "default to HashSet" for empty Bundle)

7. **WAT-CHEATSHEET update** — extend atomizable-set section for HashMap

8. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.3.md` — 19-row scorecard

## NOT your scope

- Consolidated predicate refactor — Stone 216.4 (likely no-op)
- Sandbox-walker validation — Stone 216.5
- INSCRIPTION — Stone 216.6
- WARD-PASS, INTERSTITIAL — orchestrator post-ship

## STOP triggers

- STOP-1: `hashmap_key` machinery (arc 057 slice 3) doesn't compose cleanly for arbitrary K_holon → Value::HashMap key — flag if needed
- STOP-2: Shape-discriminator interaction between consumer-declared T and Bundle shape is subtler than expected (e.g., consumer declares HashMap<i64, V> but Bundle is sequential-i64-keyed which Vec path would claim) — flag the resolution
- STOP-3: any existing test fails — surface
- STOP-4: 105 min elapsed

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip -p wat
cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat
cargo test --release --test probe_arc216_stone1_hashset_roundtrip -p wat
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio -p wat
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat
cargo test --release --test probe_arc215_stone2 -p wat
cargo test --release --test probe_arc215_collection_literal_inference -p wat
cargo test --release --test probe_brace_map_literal -p wat
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat
cargo clippy --release -- -D warnings
```

## When you finish

Report: pass count out of 19, deltas, verification summary, elapsed time, anything discovered.

Don't commit. Orchestrator commits after review.
