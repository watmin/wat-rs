# BRIEF — Arc 216 Stone 216.5b — `Value::wat__std__HashSet` storage refactor

**Stone:** retire the canonical-key crutch from HashSet storage. `Value::wat__std__HashSet` switches from `Arc<HashMap<String, Value>>` (where String is the canonical key from `hashmap_key`) to `Arc<HashSet<Value>>` (native, leveraging Stone 216.5a's `impl Hash for Value`). All HashSet constructors, accessors, and polymorphic dispatch sites refactor to use the native form. `hashmap_key` still exists for the other variants (Vec, HashMap, WatAST) — its HashSet arm updates to iterate Values directly. Stone 216.5c does the same for HashMap; Stone 216.5d deletes `hashmap_key` entirely.
**Type:** Sonnet Mode A.
**Time budget:** 75-105 min target; 120 min STOP.
**Depends on:** Stone 216.5a (`e404056` — impl Hash for Value), Stone 216.5 (`8a6c12f` — caller audit + probe matrix that gates this refactor).
**Unblocks:** Stone 216.5c (HashMap storage refactor; parallel pattern), Stone 216.5d (delete `hashmap_key`).

## Why this stone exists

Read DESIGN.md "Antidote stones (216.5a-d)" section. The short version: 216.5a minted the antidote molecule (`impl Hash for Value`); 216.5b applies it to HashSet's storage. The poison is being purged systemically; each stone removes one layer of crutch.

## The transformation

```rust
// Before (the crutch):
Value::wat__std__HashSet(Arc<HashMap<String, Value>>)
//                              ^^^^^^  canonical key from hashmap_key
//                                     ^^^^^^  actual element

// After (native):
Value::wat__std__HashSet(Arc<HashSet<Value>>)
//                              ^^^^^^^^^^^  uses Value's new impl Hash + impl PartialEq
```

The canonical-key String is gone. Hash + dedupe happen natively via Stone 216.5a's `impl Hash for Value`.

## Pre-flight verified

- Stone 216.5a SHIPPED — `impl Hash + PartialEq + Eq for Value` at `src/runtime.rs:620+`; HashSet/HashMap arms use sort-then-hash for set/map semantics
- Stone 216.5 SHIPPED — `hashmap_key` extension + 12-probe matrix (`probe_arc216_stone5_hashmap_key_coverage.rs`) + caller audit (18 sites identified)
- Stone 216.1 SHIPPED — HashSet round-trip through Atom/atom-value (`probe_arc216_stone1_hashset_roundtrip.rs`)
- All probe suites GREEN at commit `e404056`

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

### Part A — Storage refactor

1. **Update the Value enum variant** in `src/runtime.rs`:
   ```rust
   // FROM:
   wat__std__HashSet(Arc<HashMap<String, Value>>),
   // TO:
   wat__std__HashSet(Arc<HashSet<Value>>),
   ```

2. **Update the PartialEq arm for Value::wat__std__HashSet** in Stone 216.5a's manual impl:
   - Current (216.5a): iterates `a.iter()` matching keys + comparing values
   - New: native `Arc<HashSet<Value>>` equality (`HashSet` derives PartialEq from element PartialEq); reduces to `a == b`

3. **Update the Hash arm for Value::wat__std__HashSet** in Stone 216.5a's manual impl:
   - Current (216.5a): iterates `s.values()`, hashes each Value, sorts, hashes the sorted list
   - New: same sort-then-hash semantics, but iterates `s.iter()` (Values directly); slight simplification

### Part B — Constructor + accessors

4. **`eval_hashset_ctor`** at `src/runtime.rs` (~line 9444):
   - Currently calls `hashmap_key` per element to compute canonical key + inserts into HashMap
   - New: directly `set.insert(v)` on `HashSet<Value>` (Stone 216.5a's impl Hash makes this work natively)
   - Dedupe semantics preserved (HashSet::insert is idempotent on equal Values)

5. **Audit all HashSet accessor verbs** via grep:
   - `grep -n "wat__std__HashSet" src/runtime.rs`
   - `grep -n "HashSet/contains\|HashSet/conj\|HashSet/dissoc\|HashSet/length\|HashSet/empty\|HashSet/keys\|HashSet/values" src/`
   - Per-verb refactor:
     - `contains?` → `set.contains(&v)` (native; replaces hashmap_key lookup)
     - `conj` (insert) → `set.insert(v)` (native; arc Mutex-free via Arc::make_mut OR build new Arc<HashSet>)
     - `dissoc` (remove) → `set.remove(&v)`
     - `length` → `set.len()` (unchanged signature; just different inner type)
     - `empty?` → `set.is_empty()`
     - any iteration verbs (map/fold/for-each) → native iteration

6. **`value_to_atom` for HashSet** (from Stone 216.1) at `src/runtime.rs`:
   - Currently iterates `s.values()` (the actual Values via HashMap interface)
   - New: iterate `s.iter()` (the actual Values via HashSet interface)
   - Atomization output unchanged: Bundle of bare atoms

7. **`hashmap_key` HashSet arm** at `src/runtime.rs:9351-9355`:
   - Currently iterates `s.keys()` (the canonical Strings, already pre-computed)
   - New: iterates `s.iter()` (Values directly); computes canonical key via recursive `hashmap_key(op, v)` per element; sorts; joins
   - Same external behavior; new internal path
   - This arm STILL EXISTS until Stone 216.5d deletes hashmap_key entirely

### Part C — Caller sweep (use 216.5 audit)

8. **Stone 216.5 SCORE audit identified 18 `hashmap_key` call sites.** Re-grep for current callers: `grep -n "hashmap_key(" src/`. For each caller that operates on a HashSet Value:
   - Determine if it still needs `hashmap_key` (e.g., it's hashing the outer set into a HashMap-of-Sets key) or if it can switch to native `Value: Hash`
   - For HashSet-internal operations: switch to native Hash
   - For HashSet-as-key-in-other-collection operations: keep `hashmap_key` (still needed; switching is Stone 216.5c work for HashMap callers)

9. **`assoc` / `keys` / `values` / `get` / `contains-key?` polymorphic dispatch arms for HashSet** — these may not exist for HashSet (sets don't have key/value distinction), but if they do, audit + refactor. Reference `arc 146` dispatch mechanism.

### Part D — Probes

10. **New probe suite** at `tests/probe_arc216_stone5b_hashset_native_storage.rs` (~10 probes) — focused on the refactor's INVARIANTS:
    - Probe 1: HashSet construction with primitive elements (i64, String, bool, keyword) — same observable behavior
    - Probe 2: `HashSet/contains?` works for all primitive types
    - Probe 3: `HashSet/length` works
    - Probe 4: `HashSet/empty?` works (true for empty, false for non-empty)
    - Probe 5: `HashSet/conj` returns a new HashSet with the element added (dedupe semantic)
    - Probe 6: `HashSet/dissoc` returns a new HashSet without the element
    - Probe 7: Nested HashSet — `HashSet<HashSet<i64>>` construction + element lookup works
    - Probe 8: HashSet round-trip through `:wat::holon::Atom` + `atom-value` (Stone 216.1's contract preserved)
    - Probe 9: HashSet inside HashMap as VALUE — `HashMap<keyword, HashSet<i64>>` works (HashMap still uses hashmap_key for its keys; HashSet uses native Hash for its elements; the boundary works)
    - Probe 10: HashSet inside HashMap as KEY — `HashMap<HashSet<i64>, String>` works (HashMap's hashmap_key arm for HashSet element calls native Hash internally)

11. **All prior probe suites stay green** — especially `probe_arc216_stone1_hashset_roundtrip` (10/10), `probe_arc216_stone5_hashmap_key_coverage` (12/12), `probe_arc216_stone5a_value_hash` (22/22). These are the contracts that gate the refactor.

### Part E — Documentation

12. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5b.md` — scorecard matching EXPECTATIONS row count; document any judgment calls (Arc::make_mut vs build-new-Arc for conj/dissoc; accessor verbs renamed if any; deltas).

13. **No DESIGN/WAT-CHEATSHEET updates** — the refactor is internal; user-facing behavior preserved.

## NOT your scope

- **HashMap storage refactor** — Stone 216.5c
- **Delete `hashmap_key`** — Stone 216.5d
- **Sandbox-walker validation** — Stone 216.6
- **INSCRIPTION** — Stone 216.7
- **Touching Value variants OTHER than `wat__std__HashSet`** — leave HashMap variant alone
- **Touching `hashmap_key` arms OTHER than the HashSet arm** — leave Vec, HashMap, WatAST arms alone
- **Extending `is_atomizable`** — out of scope (the Tuple/Option/Result observation from 216.5a is for a future arc)

## STOP triggers (sharpened)

- **STOP-1: HashMap storage temptation.** If you find yourself wanting to refactor `Value::wat__std__HashMap` because the parallel pattern is so obvious, STOP. That's 216.5c.
- **STOP-2: `hashmap_key` deletion temptation.** If you see `hashmap_key` arms that look unused after this stone, STOP. Don't delete. That's 216.5d.
- **STOP-3: probe substitution.** If a probe fails, fix the impl or surface the constraint; do NOT change the probe's subject.
- **STOP-4: caller behavior change.** If a verb's WAT-surface semantics change (different return type, different error behavior, different ordering) — surface; orchestrator decides.
- **STOP-5: dispatch arm count surprise.** If grep surfaces FAR more HashSet dispatch sites than expected (e.g., the polymorphism is deeper than 216.5's audit captured) — surface; consider sub-decomposition.
- **STOP-6: any existing probe fails** — surface; do not push through.
- **STOP-7: 120 min elapsed.**

## Verification

Single commands per line:

```
cargo build --release
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

Report: pass count out of 13, deltas (NOT substitutions), verification summary, elapsed time, caller refactor count (how many sites touched), Arc::make_mut vs new-Arc decision for conj/dissoc.

Don't commit. Orchestrator commits after review.
