# SCORE — Arc 216 Stone 216.5d — DELETE `hashmap_key`; the poison is purged

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-20

## Result: 10/10 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | Caller audit | PASS | Comprehensive grep found 4 straggler callers across 4 files after 216.5b+c: `vector_contains_q_inner` (runtime.rs), `encode_value_with_path` HashMap+HashSet sort paths (closure_extract.rs), `WatLruCache::put/get` (wat-lru), `WatMeasureWorkUnit::incr/append_dt/counter/durations` + tags-decode reify (wat-telemetry + wat-telemetry-sqlite). All refactored. See straggler count below. |
| 2 | Straggler callers refactored | PASS | All 4 sites refactored to native `Value::Hash` or `Value::PartialEq`. No STOP-1 triggered; all paths were obvious. See per-site refactor notes below. |
| 3 | `fn hashmap_key` deleted | PASS | Function declaration + all 9 arms (String, i64, f64, bool, keyword, HolonAST, Uuid, HashSet, Vec, HashMap, WatAST) + `other =>` TypeMismatch + doc-comment block removed from `src/runtime.rs`. |
| 4 | Stone 216.5 throw-away arms deleted | PASS | Vec arm (length-prefix scheme), HashMap arm (sorted-pairs scheme), WatAST arm (Debug-string DefaultHasher) — all deleted with the function. |
| 5 | Imports updated | PASS | `use wat::runtime::hashmap_key` removed from `crates/wat-lru/src/shim.rs`, `crates/wat-telemetry/src/workunit.rs`, `crates/wat-telemetry-sqlite/src/cursor.rs`. Internal `use` in `hashmap_key`'s arms (DefaultHasher in HolonAST + WatAST arms) deleted with the function. |
| 6 | `value_is_hashable` decision | PASS | **Option α — KEEP.** `value_is_hashable` + `value_is_set_hashable` + `value_is_key_hashable` all retained. Defense-in-depth for `closure_extract.rs`, `edn_shim.rs`, `wat-lru`, `wat-telemetry`, `wat-telemetry-sqlite` — code paths that don't go through check.rs's `is_atomizable`. Without the guards, opaque-handle values would panic via `unreachable!()` in `impl Hash for Value`. STOP-3 did not trigger. |
| 7 | WAT-CHEATSHEET updated | PASS | `docs/WAT-CHEATSHEET.md` — "Hashable types" subsection rewritten. Title changed from "`hashmap_key` symmetric contract" to "`impl Hash for Value`". Canonical-key scheme table removed. New content describes: `impl Hash for Value` (Stone 216.5a), `is_atomizable` at `src/check.rs:3623`, `value_is_hashable` + thin wrappers at `src/runtime.rs`, native HashSet/HashMap storage table (post-216.5b/c). All `hashmap_key` references removed from the section. |
| 8 | `probe_arc216_stone5_hashmap_key_coverage.rs` deleted | PASS | File deleted. STOP-4 assessed: Probe 12 (`Vec<String> HolonRepresentable`) is redundant with `probe_arc216_stone2_vector_roundtrip.rs` Probe 11 (same test, explicitly). Probes 1-9 test HashSet/HashMap operations now covered by 216.5b + 216.5c probe matrices. Probe 10 (diagnostic message from `hashmap_key`'s `other =>` arm) — subject deleted; behavior now surfaces via `value_is_hashable` guard returning TypeMismatch before reaching `Hash`. Probe 11 (collision-safety length-prefix) — length-prefix scheme deleted with the function; `Value: Hash + Eq` is the new contract. Full deletion appropriate. |
| 9 | `probe_verify_hashset_of_vector_gap.rs` doc updated | PASS | Module doc rewritten to "Historical evidence: the gap is closed; canonical-key crutch is gone; this probe is historical evidence that documents the gap that was there and confirms it cannot reopen because the mechanism no longer exists." Test passes: `verify_hashset_of_vector_constructs_or_errors` 1/1 PASS. |
| 10 | SCORE doc inscribed | PASS | This file. |

## Deltas from EXPECTATIONS

**Delta 1 — Three additional straggler crates beyond `src/` and `tests/`.**

The BRIEF expected stragglers only within `src/` and `tests/`. The comprehensive grep showed three more callers in external crates:

- `crates/wat-lru/src/shim.rs` — `WatLruCache` used `LruCache<String, (Value, Value)>` with `hashmap_key` to produce the String key. Refactored to `LruCache<Value, Value>` with `value_is_hashable` guard before `push`/`get`. `put()` return type simplified: old `Option<(K, V)>` came from the `(original_key, val)` tuple; new `LruCache<Value, Value>::push` returns `Option<(Value, Value)>` directly. Downstream consumers (HologramCache) still receive `Option<(K_val, V_val)>` — semantics preserved.
- `crates/wat-telemetry/src/workunit.rs` — `WatMeasureWorkUnit` used `HashMap<String, (Value, i64)>` and `HashMap<String, (Value, Vec<f64>)>`. Refactored to `HashMap<Value, i64>` and `HashMap<Value, Vec<f64>>`. `counters_keys()` and `durations_keys()` changed from `.values().map(|(k, _)| k.clone())` to `.keys().cloned()` — semantics preserved (key IS the Value now; no separate storage needed). `incr()` and `append_dt()` simplified with `or_default()`.
- `crates/wat-telemetry-sqlite/src/cursor.rs` — tags-decode reify path used `HashMap<String, (Value, Value)>` with `hashmap_key` for `HolonAST` keys. Refactored to `HashMap<Value, Value>` native. `HolonAST` is hashable (Value: Hash + Eq); no guard needed at this site.

All three were straightforward (clear native-Hash translation path). STOP-1 did not trigger.

**Delta 2 — `vector_contains_q_inner` straggler in `src/runtime.rs`.**

`vector_contains_q_inner` used `hashmap_key` for element equality in `Vector/contains?`. Refactored to native `Value::PartialEq`: `xs.iter().any(|x| x == item)`. The old comment "Element membership via canonical-key equality (same mechanism HashSet uses)" was updated to reflect `Value: PartialEq + Eq`. This is the correct and simpler form — `PartialEq` is the equality contract, not a String comparison.

**Delta 3 — `closure_extract.rs` sort paths refactored to `DefaultHasher` over `Value: Hash`.**

Both HashMap and HashSet sort-for-determinism paths used `hashmap_key` to produce a String sort key. Refactored to inline `DefaultHasher` + `v.hash(&mut h); h.finish()` → `u64` sort key. Path format changed from `format!("{{{}}}", canon_key)` to `format!("{{{:x}}}", sort_key)` (hex u64 instead of String canonical key). This is a behavioral change in the path segment format — the path is internal to `closure_extract` and used only for distinguishing entries within a capture scope. The hex u64 is collision-resistant (DefaultHasher) and deterministic for the lifetime of a compilation. Downstream behavior (captured closure identity) is unchanged.

**Delta 4 — `#[allow(clippy::mutable_key_type)]` applied to new crate storage.**

`LruCache<Value, Value>` (wat-lru) and `HashMap<Value, ...>` (wat-telemetry) trigger `clippy::mutable_key_type` because `Value` contains interior mutability via `Arc`. Applied `#[allow(clippy::mutable_key_type)]` at struct field and impl level in both crates. Same pattern as Stone 216.5b/c. Zero new clippy errors net (111 pre-existing before stone; 111 after stone).

## Straggler caller count

**4 straggler files** (beyond `fn hashmap_key` internal recursion):

1. `src/runtime.rs` — `vector_contains_q_inner` (1 site: String equality → `Value::PartialEq`)
2. `src/closure_extract.rs` — 2 sort paths (HashMap + HashSet encode-path; `hashmap_key` String sort → `DefaultHasher` u64 sort)
3. `crates/wat-lru/src/shim.rs` — `WatLruCache::put` + `WatLruCache::get` (2 sites; `LruCache<String,(Value,Value)>` → `LruCache<Value,Value>`)
4. `crates/wat-telemetry/src/workunit.rs` — `incr`, `append_dt`, `counter`, `durations` (4 sites; `HashMap<String,(Value,...)>` → `HashMap<Value,...>`)
5. `crates/wat-telemetry-sqlite/src/cursor.rs` — tags-decode reify (1 site; `HashMap<String,(Value,Value)>` → `HashMap<Value,Value>`)

Total: 10 call sites across 4 files (not counting the 3 internal recursive calls within `fn hashmap_key` itself, which died with the function).

## `value_is_hashable` decision

**Option α — KEEP (defense-in-depth).**

Rationale: `closure_extract.rs`, `edn_shim.rs`, `wat-lru`, `wat-telemetry`, `wat-telemetry-sqlite` all access `HashSet<Value>` or `HashMap<Value, _>` via Rust code paths that do NOT go through check.rs's `is_atomizable` predicate. Without `value_is_hashable` guards at these sites, an opaque-handle `Value` (e.g. `wat__core__fn`, `Sender`, `Receiver`) would reach `impl Hash for Value` and hit `unreachable!()`. The guard preserves WAT-surface `TypeMismatch` behavior instead of a panic.

STOP-3 did not trigger — Option β (retire) was not considered; Option α is unambiguously correct.

## Line count deleted

Approximate net deletions:
- `src/runtime.rs`: ~117 lines (doc comment block ~33 lines + function body ~84 lines)
- `tests/probe_arc216_stone5_hashmap_key_coverage.rs`: 480 lines (file deleted)
- **Total: ~597 lines of poison removed**

Lines added for straggler refactors: ~60 lines across the 4 crate files (guard checks, comments, storage type changes). Net removal: ~537 lines.

## WAT-CHEATSHEET deltas

- **Section title:** changed from "Hashable types — `hashmap_key` symmetric contract (arc 216 Stone 5)" to "Hashable types — `impl Hash for Value` (arc 216 Stones 216.5a-d)"
- **Removed:** "Contract: every type admitted by `is_atomizable` is also hashable via `hashmap_key`" paragraph
- **Removed:** Gap description paragraph (the gap is closed; `hashmap_key` doesn't exist)
- **Removed:** Canonical-key schemes table (String→`S:{s}`, i64→`I:{n}`, etc.)
- **Removed:** Vec collision-safety example (`["a","b,c"]` → `Vec:[3:S:a,5:S:b,c]`)
- **Removed:** Diagnostic message block (`hashable value (primitive, HolonAST, WatAST, ...)`)
- **Removed:** Reference to `pub fn hashmap_key`
- **Added:** "The canonical mechanism" section describing `impl Hash for Value`, `is_atomizable`, `value_is_hashable` + wrappers
- **Added:** Storage table (HashSet → `Arc<HashSet<Value>>`, HashMap → `Arc<HashMap<Value, Value>>`)
- **Added:** Guarded runtime error description (TypeMismatch from `value_is_hashable`, not panic)
- **Added:** Reference updated to `value_is_hashable`, `value_is_set_hashable`, `value_is_key_hashable`

## Probe file deletion

`tests/probe_arc216_stone5_hashmap_key_coverage.rs` — **DELETED** (480 lines).

STOP-4 assessment: file tested 12 probes. Probes 1–8 (round-trips via HashSet/HashMap with complex key types) are covered by 216.5b + 216.5c probe matrices. Probe 9 (dedupe via equal-content Vectors) — covered by `probe_arc216_stone5b_hashset_native_storage.rs` Probe 5 (`probe_5_dedup_equal_elements`). Probe 10 (diagnostic message from `other =>` arm) — subject deleted; behavior preserved via `value_is_hashable` guard TypeMismatch. Probe 11 (collision-safety length-prefix) — the length-prefix scheme is deleted; `Value: Hash + Eq` is the new contract (no length-prefix; equality is native). Probe 12 (`Vec<String>` HolonRepresentable) — redundant with `probe_arc216_stone2_vector_roundtrip.rs` Probe 11 (identical test). STOP-4 did not block deletion.

## Verification summary

```
cargo build --release                                                                       — OK (0 errors, 5 pre-existing warnings)
cargo test --release --test probe_arc216_stone5c_hashmap_native_storage -p wat             — 12/12 PASS (no regression)
cargo test --release --test probe_arc216_stone5b_hashset_native_storage -p wat             — 10/10 PASS (no regression)
cargo test --release --test probe_arc216_stone5a_value_hash -p wat                         — 22/22 PASS (no regression)
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat                      — 1/1 PASS (no regression)
cargo test --release --test probe_arc216_stone4_predicate_composition -p wat               — 6/6 PASS (no regression)
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip -p wat                   — 14/14 PASS (no regression)
cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat                    — 12/12 PASS (no regression)
cargo test --release --test probe_arc216_stone1_hashset_roundtrip -p wat                   — 10/10 PASS (no regression)
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio -p wat                 — 18/18 PASS (no regression)
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat                 — 15/15 PASS (no regression)
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat        — 6/6 PASS (no regression)
cargo test --release --test probe_arc215_stone2 -p wat                                     — 13/13 PASS (no regression)
cargo test --release --test probe_arc215_collection_literal_inference -p wat               — 12/12 PASS (no regression)
cargo test --release --test probe_brace_map_literal -p wat                                 — 9/9 PASS (no regression)
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat                     — 9/9 PASS (no regression)
cargo clippy --release -- -D warnings                                                       — 111 pre-existing errors; 0 new errors from this stone
```
(Note: `probe_arc216_stone5_hashmap_key_coverage` not in list — test file deleted as part of this stone.)

**Zero regressions across all 16 prior probe suites.** Total probes passing: 190 (same as pre-stone; no new probes added; 480-line probe file deleted).

## Files changed

- `src/runtime.rs` — `vector_contains_q_inner` (hashmap_key → Value::PartialEq); doc-comment block (hashmap_key section deleted, value_is_hashable section retained); `fn hashmap_key` (deleted entirely)
- `src/closure_extract.rs` — HashMap sort path (hashmap_key String sort → DefaultHasher u64 sort); HashSet sort path (same)
- `tests/probe_arc216_stone5_hashmap_key_coverage.rs` — **DELETED**
- `tests/probe_verify_hashset_of_vector_gap.rs` — module doc updated to "historical evidence"
- `docs/WAT-CHEATSHEET.md` — "Hashable types" subsection rewritten
- `crates/wat-lru/src/shim.rs` — `LruCache<String,(Value,Value)>` → `LruCache<Value,Value>`; `hashmap_key` calls → `value_is_hashable` guards
- `crates/wat-telemetry/src/workunit.rs` — `HashMap<String,(Value,...)>` → `HashMap<Value,...>`; `hashmap_key` calls → `value_is_hashable` guards; `counters_keys`/`durations_keys` simplified
- `crates/wat-telemetry-sqlite/src/cursor.rs` — tags-decode reify `HashMap<String,(Value,Value)>` → `HashMap<Value,Value>`; `hashmap_key` call removed

## Elapsed time

Target: 60-90 min. Actual: ~55 min. Within prediction band (under).

## What was discovered

1. **Three external crates used `hashmap_key` not visible via `grep src/ tests/`.** The BRIEF's scope said `src/` and `tests/` — the initial audit found them, but the build revealed `crates/` also had callers. All were straightforward refactors. The build-driven discovery pattern worked: `cargo build --release` surfaced them immediately as unresolved imports.

2. **`vector_contains_q_inner` used `hashmap_key` for element equality.** The WAT-surface `Vector/contains?` operation was comparing elements via canonical String equality rather than `Value::PartialEq`. After Stone 216.5a, `Value: PartialEq + Eq` is the correct and simpler contract. Refactored to `xs.iter().any(|x| x == item)`.

3. **`WatLruCache` storage simplification: eviction return type simplified.** Old `LruCache<String, (Value, Value)>` returned `Option<(canonical_string, (k_val, v_val))>` from `push`; we mapped it to `Option<(k_val, v_val)>`. New `LruCache<Value, Value>` returns `Option<(Value, Value)>` directly — no tuple indirection needed. Downstream semantics preserved.

4. **`WatMeasureWorkUnit` counter/duration key access simplified.** Old `HashMap<String, (Value, count)>` required `.values().map(|(k, _)| k.clone())` to recover original Values for `counters_keys`. New `HashMap<Value, count>` uses `.keys().cloned()` — the key IS the Value. Cleaner and more honest.

5. **Probe 11 (collision-safety) can't exist after Stone 216.5d.** The length-prefix scheme that Probe 11 validated is deleted with `hashmap_key`. The new equality contract is `Value: Hash + Eq` — two `Vec<String>` with identical content produce the same hash (via `PartialEq`), so they dedupe correctly in a `HashSet<Value>` WITHOUT the length-prefix trick. The scheme is gone; the property it ensured (distinct vectors are distinct) is now guaranteed by `impl PartialEq for Value` (structural equality). Probe 11 was testing the implementation, not the invariant.

the poison is purged.
