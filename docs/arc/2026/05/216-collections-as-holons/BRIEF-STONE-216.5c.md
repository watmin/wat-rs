# BRIEF — Arc 216 Stone 216.5c — `Value::wat__std__HashMap` storage refactor

**Stone:** retire the canonical-key crutch from HashMap storage. `Value::wat__std__HashMap` switches from `Arc<HashMap<String, (Value, Value)>>` (canonical_key, (original_K, V)) to `Arc<HashMap<Value, Value>>` (native, leveraging Stone 216.5a's `impl Hash for Value`). All HashMap constructors, accessors, and polymorphic dispatch sites refactor to use the native form. `hashmap_key` still exists for Vec only after this stone — Stone 216.5d deletes it entirely.
**Type:** Sonnet Mode A.
**Time budget:** 75-105 min target; 120 min STOP.
**Depends on:** Stone 216.5a (`e404056` — impl Hash for Value), Stone 216.5b (`ff5f86d` — HashSet storage refactor; parallel pattern + `value_is_set_hashable` template).
**Unblocks:** Stone 216.5d (delete `hashmap_key` entirely + cleanup).

## Why this stone exists

Read DESIGN.md "Antidote stones (216.5a-d)" section. Stone 216.5c is the parallel of 216.5b for HashMap. The canonical-key crutch leaves the HashMap code path; both K and V are stored natively.

## The transformation

```rust
// Before (the crutch):
Value::wat__std__HashMap(Arc<HashMap<String, (Value, Value)>>)
//                              ^^^^^^  canonical key of K (from hashmap_key)
//                                     ^^^^^^^^^^^^  (original_K, V)

// After (native):
Value::wat__std__HashMap(Arc<HashMap<Value, Value>>)
//                              ^^^^^^^^^^^^^  K → V; native Hash + PartialEq on Value
```

The redundant canonical-key String is gone. The original K Value becomes the actual HashMap key directly.

## Pre-flight verified

- Stone 216.5a SHIPPED — `impl Hash + PartialEq + Eq for Value` (foundation)
- Stone 216.5b SHIPPED — HashSet storage refactor (parallel pattern); `value_is_set_hashable` defensive guard
- Stone 216.5 SHIPPED — `hashmap_key` HashMap arm + 12-probe matrix + caller audit
- All probe suites GREEN at commit `ff5f86d`

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

### Part A — Storage refactor

1. **Update the Value enum variant** in `src/runtime.rs`:
   ```rust
   // FROM:
   wat__std__HashMap(Arc<HashMap<String, (Value, Value)>>),
   // TO:
   wat__std__HashMap(Arc<HashMap<Value, Value>>),
   ```

2. **Update the PartialEq arm for Value::wat__std__HashMap** in Stone 216.5a's manual impl:
   - Current (216.5a): iterates `a.iter()` matching canonical keys + comparing (K,V) tuples
   - New: native `a == b` on `Arc<HashMap<Value, Value>>` (std lib HashMap PartialEq via element PartialEq)

3. **Update the Hash arm for Value::wat__std__HashMap** in Stone 216.5a's manual impl:
   - Current (216.5a): iterates `m.values()` for (k_val, v_val) tuples; hashes pairs; sort
   - New: iterate `m.iter()` for (k, v) directly; hashes pairs; sort
   - Same sort-then-hash semantics

### Part B — Constructor + accessors

4. **`eval_hashmap_ctor`** at `src/runtime.rs` (~line 9376):
   - Currently calls `hashmap_key` per K to compute canonical key + inserts (canonical_k, (k, v)) into HashMap
   - New: directly `map.insert(k, v)` on `HashMap<Value, Value>` (Stone 216.5a's impl Hash on Value makes this work natively)
   - Duplicate-key semantics preserved (HashMap::insert overwrites)
   - Guard: reject non-hashable K via new `value_is_key_hashable` predicate (parallel to 216.5b's `value_is_set_hashable`) — preserves WAT-surface TypeMismatch instead of `unreachable!()` panic

5. **Audit all HashMap accessor verbs** via grep:
   - `grep -n "wat__std__HashMap" src/`
   - `grep -n "HashMap/get\|HashMap/assoc\|HashMap/dissoc\|HashMap/keys\|HashMap/values\|HashMap/contains-key\|HashMap/length\|HashMap/empty" src/`
   - Per-verb refactor:
     - `get` → `map.get(&k).cloned()` (returns Option<V>)
     - `assoc` (insert) → `map.insert(k, v)` (new-Arc strategy mirroring 216.5b)
     - `dissoc` (remove) → `map.remove(&k)` (new-Arc strategy)
     - `keys` → `map.keys().cloned().collect()` (returns Vec<K>; was returning the canonical Strings before — this is a SEMANTIC CORRECTION, since users expect actual K Values not String keys)
     - `values` → `map.values().cloned().collect()` (unchanged in spirit; just iterates differently)
     - `contains-key?` → `map.contains_key(&k)`
     - `length` → `map.len()`
     - `empty?` → `map.is_empty()`

6. **`value_to_atom` for HashMap** (from Stone 216.3) at `src/runtime.rs`:
   - Currently iterates `m.values()` to get (k_val, v_val) tuples
   - New: iterate `m.iter()` to get (k, v) directly
   - Atomization output unchanged: Bundle of Bind(K_holon, V_holon) pairs

7. **`hashmap_key` HashMap arm** at `src/runtime.rs` (the arm added in Stone 216.5):
   - Currently iterates `m.values()` to get (k_val, v_val); computes canonical key string per (k, v) pair via recursive hashmap_key; sorted+joined
   - New: iterate `m.iter()` to get (k, v) directly; same recursive call + sort + join
   - Same external behavior; new internal path
   - This arm STILL EXISTS until Stone 216.5d

### Part C — `value_is_key_hashable` predicate

8. **Add `value_is_key_hashable`** in `src/runtime.rs` (parallel to `value_is_set_hashable` from Stone 216.5b):
   - Returns `false` for the 14 opaque-handle variants
   - Called by `eval_hashmap_ctor` and `hashmap_assoc_inner` BEFORE `HashMap::insert`
   - Preserves WAT-surface TypeMismatch behavior
   - **Question for SCORE: should `value_is_key_hashable` and `value_is_set_hashable` collapse into ONE `value_is_hashable` function?** They likely have IDENTICAL bodies (same 14 opaque-handle variants); sonnet's call to unify or keep separate. Document.

### Part D — Caller sweep (use 216.5 + 216.5b audits)

9. **Stone 216.5 SCORE audit + 216.5b's discovery of closure_extract.rs + edn_shim.rs.** Re-grep:
   - `grep -n "wat__std__HashMap" src/`
   - For each caller: refactor to native HashMap<Value, Value> operations
   - closure_extract.rs likely has a HashMap arm parallel to its HashSet arm — refactor
   - edn_shim.rs likely has HashMap-related arms — refactor

10. **`#[allow(clippy::mutable_key_type)]`** — parallel to 216.5b; HashMap<Value, Value> triggers the same lint; suppress at affected sites with the same explanatory comment

### Part E — Probes

11. **New probe suite** at `tests/probe_arc216_stone5c_hashmap_native_storage.rs` (~12 probes):
    - Probe 1: HashMap construction with primitive K + V — same observable behavior
    - Probe 2: `HashMap/get` returns Option<V>; Some on hit, None on miss
    - Probe 3: `HashMap/assoc` inserts; overwrite semantic preserved
    - Probe 4: `HashMap/dissoc` removes; returns new HashMap without the key
    - Probe 5: `HashMap/keys` returns Vec<K> with actual Values (NOT canonical String keys; SEMANTIC CORRECTION verified)
    - Probe 6: `HashMap/values` returns Vec<V>
    - Probe 7: `HashMap/contains-key?` works
    - Probe 8: `HashMap/length` works
    - Probe 9: `HashMap/empty?` works (true for empty, false for non-empty)
    - Probe 10: Nested HashMap — `HashMap<keyword, HashMap<keyword, i64>>` construction + get works
    - Probe 11: HashMap with non-primitive K — `HashMap<HashSet<i64>, String>` (HashSet as K; uses native Hash on HashSet from 216.5b)
    - Probe 12: HashMap round-trip through `:wat::holon::Atom` + `atom-value` (Stone 216.3's contract preserved)

12. **All prior probe suites stay green** — especially `probe_arc216_stone3_hashmap_roundtrip` (14/14), `probe_arc216_stone5_hashmap_key_coverage` (12/12), `probe_arc216_stone5a_value_hash` (22/22), `probe_arc216_stone5b_hashset_native_storage` (10/10).

### Part F — Documentation

13. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5c.md` — scorecard matching EXPECTATIONS row count; document `keys` semantic correction (now returns actual K Values, not canonical Strings); document `value_is_key_hashable` unification decision; deltas.

14. **No DESIGN/WAT-CHEATSHEET updates** — internal refactor.

## NOT your scope

- **`hashmap_key` deletion** — Stone 216.5d
- **HashSet storage refactor** — Stone 216.5b (already shipped)
- **Sandbox-walker validation** — Stone 216.6
- **INSCRIPTION** — Stone 216.7
- **Touching Value variants OTHER than `wat__std__HashMap`** — leave HashSet alone (already done)
- **Touching `hashmap_key` arms OTHER than the HashMap arm** — leave Vec, WatAST arms alone
- **Extending `is_atomizable`** — out of scope

## STOP triggers (sharpened)

- **STOP-1: `hashmap_key` deletion temptation.** If you see `hashmap_key` arms that look unused after this stone, STOP. Don't delete. That's 216.5d.
- **STOP-2: HashSet re-refactor.** If you find HashSet refactor opportunities that 216.5b missed, surface; don't expand scope silently.
- **STOP-3: probe substitution.** If a probe fails because the impl doesn't quite work, STOP. Do NOT change the probe's subject; fix the impl or surface the constraint.
- **STOP-4: caller behavior change.** If a verb's WAT-surface semantics change (other than `keys` returning actual K Values vs canonical Strings, which IS the intended correction) — surface; orchestrator decides.
- **STOP-5: dispatch arm count surprise.** If grep surfaces FAR more HashMap dispatch sites than expected — surface.
- **STOP-6: `keys` semantic correction breaks a real caller.** If a caller depends on `keys` returning canonical Strings (which would be the bug), surface; don't paper over.
- **STOP-7: any existing probe fails** — surface.
- **STOP-8: 120 min elapsed.**

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone5c_hashmap_native_storage -p wat
cargo test --release --test probe_arc216_stone5b_hashset_native_storage -p wat
cargo test --release --test probe_arc216_stone5a_value_hash -p wat
cargo test --release --test probe_arc216_stone5_hashmap_key_coverage -p wat
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat
cargo test --release --test probe_arc216_stone4_predicate_composition -p wat
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

Report: pass count out of 14, deltas (NOT substitutions), verification summary, elapsed time, caller refactor count, `value_is_key_hashable` unification decision, `keys` semantic correction verification.

Don't commit. Orchestrator commits after review.
