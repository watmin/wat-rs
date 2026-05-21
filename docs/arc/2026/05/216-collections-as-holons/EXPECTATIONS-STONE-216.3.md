# EXPECTATIONS — Arc 216 Stone 216.3 — HashMap round-trip

Mode A target: 19/19 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `value_to_atom` extended for HashMap | New match arm adjacent to Vec/HashSet; produces `Bundle` of Bind(K_holon, V_holon) pairs; iteration order non-canonical (HashMap unordered) |
| 2 | `eval_atom_value` Bundle shape-dispatch extended | Third path: arbitrary-K Binds → HashMap when consumer declares `T = :wat::core::HashMap<K, V>`; uses arc 057 slice 3's hashmap_key for K |
| 3 | `holon_item_to_value` extended | Handles Bind(K, V) for nested HashMap values (mirrors 216.2's Bind(I64) handling) |
| 4 | HolonRepresentable impl for HashMap<K, V> | `src/comms/mod.rs` — bounds `K: HolonRepresentable + Hash + Eq + Send + 'static, V: HolonRepresentable + Send + 'static` |
| 5 | `is_atomizable` HashMap entry | Verify entry exists (may be pre-landed per 216.1/216.2 bonuses); if missing, add: HashMap<K,V> atomizable iff K atomizable AND V atomizable |
| 6 | Probe 1 — Forward | `(value_to_atom {:foo 42 :bar 99})` → Bundle of Binds with keyword K, i64 V |
| 7 | Probe 2 — Reverse | `(atom-value <bundle> -> :wat::core::HashMap<keyword, i64>)` → HashMap{:foo→42, :bar→99} |
| 8 | Probe 3 — Empty map round-trip | `{}` → Bundle([]) → empty HashMap (consumer-declared type disambiguates from empty HashSet/Vec) |
| 9 | Probe 4 — Multi-K types | Works for HashMap<keyword,V>, HashMap<String,V>, HashMap<i64,V>, HashMap<bool,V> |
| 10 | Probe 5 — Multi-V types | Works for HashMap<K,i64>, HashMap<K,String>, HashMap<K,bool>, HashMap<K,keyword> |
| 11 | Probe 6 — Non-keyword keys | HashMap<i64, String> round-trips (arbitrary K via hashmap_key) |
| 12 | Probe 7 — Nested map | HashMap<keyword, HashMap<keyword, i64>> round-trips |
| 13 | Probe 8 — Mixed nesting (Vec) | HashMap<keyword, Vec<i64>> round-trips |
| 14 | Probe 9 — Mixed nesting (HashSet) | HashMap<keyword, HashSet<i64>> round-trips |
| 15 | Probe 10 — Check passes | `(:wat::holon::Atom my-hashmap)` for atomizable K+V type-checks |
| 16 | Probe 11 — Check fails | Non-atomizable K or V fails at check; diagnostic |
| 17 | Probe 12 — HolonRepresentable cascade | Compile-time HashMap<String, i64>: HolonRepresentable |
| 18 | Probe 13 — Shape disambiguation | Bundle of Bind(I64) with non-sequential keys → HashMap<i64, V> (Vec path fails seq check; HashMap takes it) |
| 19 | Probe 14 — Empty Bundle disambiguation | Empty Bundle + consumer declares HashMap → empty HashMap (overrides 216.2's default-to-HashSet for empty) |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 105 min
**Confidence:** medium

**Rationale:**
- Capstone collection — two type params (K + V) vs one for HashSet/Vec
- Pattern templates from 216.1 + 216.2 directly apply; sonnet shipping consistently under target
- New mechanic: arbitrary K via arc 057 slice 3 hashmap_key (existing primitive)
- New mechanic: third shape-dispatch path in atom-value (HashMap discriminated from Vec by K type / sequential-key check failure / consumer annotation)
- Empty Bundle disambiguation: consumer-declared type overrides 216.2's HashSet default
- Risk: shape-dispatch ordering subtleties (when does HashMap path win vs Vec path vs HashSet path)
- Risk: HolonRepresentable bounds for K (Hash + Eq + Send + 'static) may surface trait bound conflicts

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Predicate consolidation — Stone 216.4
- Sandbox-walker validation — Stone 216.5
- INSCRIPTION — Stone 216.6

## Honesty deltas accepted

- is_atomizable HashMap entry: may already exist as a 216.1/216.2 bonus (per their SCORE deltas); document
- Shape-dispatch ordering choice — sonnet picks; documents
- Empty Bundle case: HashMap takes precedence over HashSet when consumer declares HashMap; document the precedence rule
