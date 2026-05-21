# EXPECTATIONS — Arc 216 Stone 216.5c — HashMap storage refactor

Mode A target: 14/14 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Value enum variant updated | `Value::wat__std__HashMap(Arc<HashMap<Value, Value>>)` in `src/runtime.rs`; canonical-key String storage gone |
| 2 | Stone 216.5a PartialEq + Hash arms simplified | Native `a == b` for PartialEq; `m.iter()` for Hash; sort-then-hash semantics preserved |
| 3 | `eval_hashmap_ctor` refactored | Direct `map.insert(k, v)` on `HashMap<Value, Value>`; `hashmap_key` call removed; `value_is_key_hashable` guard added |
| 4 | `HashMap/get` refactored | `map.get(&k).cloned()` returns Option<V> |
| 5 | `HashMap/assoc` refactored | New-Arc strategy (mirrors 216.5b); overwrite semantic preserved |
| 6 | `HashMap/dissoc` refactored | New-Arc strategy; remove returns new HashMap without key |
| 7 | `HashMap/keys` SEMANTIC CORRECTION | Returns Vec<K> with actual K Values (NOT canonical String keys); verified via Probe 5 |
| 8 | `HashMap/values` + `contains-key?` + `length` + `empty?` refactored | Native HashMap ops |
| 9 | `value_to_atom` HashMap arm refactored | Iterates `m.iter()`; Bundle of Bind(K_holon, V_holon) output unchanged |
| 10 | `hashmap_key` HashMap arm refactored | Iterates `m.iter()`; recursive `hashmap_key` per (k, v); sorted+joined output unchanged |
| 11 | `value_is_key_hashable` added | Parallel to `value_is_set_hashable`; unification decision documented (collapse to `value_is_hashable` OR keep separate); 14 opaque-handle variants rejected |
| 12 | Caller sweep complete | closure_extract.rs + edn_shim.rs HashMap arms refactored; other sites audited; `#[allow(clippy::mutable_key_type)]` applied parallel to 216.5b |
| 13 | Probes 1-12 from BRIEF | All new probes pass; HashMap-of-HashMap works; HashSet-as-K HashMap works; round-trip through Atom preserved |
| 14 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5c.md` — scorecard + caller count + unification decision + `keys` correction verification + deltas |

## Independent prediction (calibration record)

**Target runtime:** 75-105 min Mode A
**Upper bound:** 120 min
**Confidence:** medium-high

**Rationale:**
- Direct parallel of Stone 216.5b — same pattern, same Arc strategy, same guard predicate shape
- HashMap has MORE accessor verbs than HashSet (get, assoc, dissoc, keys, values, contains-key?, length, empty? — 8 vs HashSet's 4-5), so more mechanical touches
- Risk: `keys` semantic correction — if a caller depends on the canonical-String-keys behavior (which would be a bug), the correction surfaces it; STOP-6 fires
- Risk: 216.5b's `value_is_set_hashable` and this stone's `value_is_key_hashable` likely have identical bodies; unification decision needs to be made
- Risk: HashMap-of-HashMap (Probe 10) and HashSet-as-K HashMap (Probe 11) exercise the recursive Hash via 216.5a; gate the refactor's correctness end-to-end
- All 216.x probe matrices stay green (especially 216.3 HashMap round-trip + 216.5 hashmap_key coverage)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites 216.5 (caller audit), 216.5a (impl Hash), 216.5b (HashSet pattern + value_is_set_hashable template), 216.3 (HashMap round-trip contract).

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- `hashmap_key` deletion — Stone 216.5d
- Sandbox-walker validation — Stone 216.6
- INSCRIPTION — Stone 216.7
- Any Value variant OTHER than `wat__std__HashMap`
- Any `hashmap_key` arm OTHER than the HashMap arm
- HashSet refactor revisit

## Honesty deltas accepted

- `value_is_key_hashable` unification (collapse with `value_is_set_hashable` into shared `value_is_hashable` OR keep separate) — sonnet picks; documents
- New-Arc vs Arc::make_mut for assoc/dissoc — sonnet picks; documents (likely matches 216.5b)
- Caller count surprises (more or fewer than 216.5 audit) — sonnet surfaces
- If `keys` semantic correction breaks a caller — surface via STOP-6; document the dependency
- If a HashMap dispatch verb has subtle semantics (e.g., `assoc` with duplicate keys returns specific ordering) — surface; preserve unless the orchestrator says otherwise

## Honesty deltas NOT accepted

- **Probe substitution — STOP-3.**
- **`hashmap_key` deletion leak — STOP-1.**
- **HashSet re-refactor leak — STOP-2.**
- **Silent caller behavior change other than `keys` — STOP-4.**
- **Silent dispatch site skip — STOP-5.**
- **Papering over `keys` correction breaking a real caller — STOP-6.**
