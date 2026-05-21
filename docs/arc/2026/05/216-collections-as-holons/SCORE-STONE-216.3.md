# SCORE — Arc 216 Stone 216.3 — HashMap round-trip

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-20

## Result: 19/19 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `value_to_atom` extended for HashMap | PASS | `src/runtime.rs` — new `Value::wat__std__HashMap(m)` arm adjacent to Vec arm (Stone 216.2). Iterates `m.values()` which gives `(k_val, v_val)` tuples; each side recursively atomized via `value_to_atom`; produces `HolonAST::bind(k_holon, v_holon)` per entry; collects into `HolonAST::bundle(items)`. Early-return pattern mirrors HashSet/Vec arms. Iteration order non-canonical (HashMap unordered). |
| 2 | `eval_atom_value` Bundle shape-dispatch extended | PASS | `src/runtime.rs` — `eval_atom_value`'s `HolonAST::Bundle(items)` arm now three-way dispatches: (1) all-I64-Bind with sequential keys → Vec; (2) all-Bind with non-I64 keys OR non-sequential I64 → HashMap; (3) empty → HashSet or HashMap per consumer hint; (4) bare atoms → HashSet. Third path: extracts `(k_holon, v_holon)` from each Bind via `holon_item_to_value`, computes `hashmap_key(k_val)` for canonical key, inserts `(canonical_key, (k_val, v_val))` into result `Value::wat__std__HashMap`. |
| 3 | `holon_item_to_value` extended | PASS | `src/runtime.rs` — `holon_item_to_value`'s `HolonAST::Bundle` arm extended to the same three-way dispatch as `eval_atom_value`: sequential-I64-Bind → Vec; all-Bind non-sequential → HashMap; empty → HashSet; bare atoms → HashSet. Handles `HolonAST::Bind(K, V)` in the `all_binds` branch by recursively calling `holon_item_to_value(K)` and `holon_item_to_value(V)`. Enables nested HashMap values (e.g., `HashMap<keyword, HashMap<keyword, i64>>`). |
| 4 | HolonRepresentable impl for HashMap<K, V> | PASS | `src/comms/mod.rs` — `impl<K, V> HolonRepresentable for std::collections::HashMap<K, V> where K: HolonRepresentable + std::hash::Hash + Eq + Send + 'static, V: HolonRepresentable + Send + 'static`. `to_holon_ast` → Bundle of Bind(K_holon, V_holon) per entry. `from_holon_ast` → matches Bundle, validates each child is Bind, reconstructs via `K::from_holon_ast` and `V::from_holon_ast`, inserts into HashMap. |
| 5 | `is_atomizable` HashMap entry | PASS | Pre-landed in Stone 216.1 (Delta 6): `src/check.rs:3644` — `"wat::core::HashMap" => args.len() == 2 && is_atomizable(&args[0]) && is_atomizable(&args[1])`. No change needed; the predicate was already correct and marked as "Stone 3 future" in its comment. Comment updated to remove "future". |
| 6 | Probe 1 — Forward | PASS | `probe_1_forward_hashmap_to_bundle` — `(:wat::holon::Atom {:foo 42 :bar 99})` → Bundle; `Bundle/children` count = 2. |
| 7 | Probe 2 — Reverse | PASS | `probe_2_reverse_bundle_to_hashmap_roundtrip` — `atom-value` on the Bundle → HashMap; length = 2; `contains-key? :foo` = true; `contains-key? :bar` = true. |
| 8 | Probe 3 — Empty map round-trip | PASS | `probe_3_empty_map_roundtrip_consumer_declared` — forward: empty HashMap → Bundle with 0 children. Reverse via consumer-declared type: `(atom-value h -> :wat::core::HashMap)` → empty HashMap (length 0). |
| 9 | Probe 4 — Multi-K types | PASS | `probe_4_multi_k_types` — `HashMap<keyword,i64>` (length 2), `HashMap<String,i64>` (length 2), `HashMap<i64,String>` (length 2), `HashMap<bool,i64>` (length 2). All round-trip via the corresponding primitive HolonAST leaves. |
| 10 | Probe 5 — Multi-V types | PASS | `probe_5_multi_v_types` — `HashMap<keyword,i64>` (length 1), `HashMap<keyword,String>` (length 2), `HashMap<keyword,bool>` (length 2), `HashMap<keyword,keyword>` (length 2). All round-trip. |
| 11 | Probe 6 — Non-keyword keys | PASS | `probe_6_non_keyword_keys_i64_string` — `HashMap<i64, String>` with keys 100, 200; length 2; `contains-key? 100` = true. Arbitrary K via hashmap_key (i64 → "I:{n}" canonical key). |
| 12 | Probe 7 — Nested map | PASS | `probe_7_nested_map_roundtrip` — `HashMap<keyword, HashMap<keyword, i64>>`; outer length = 1; outer Bundle has 1 Bind child. Inner HashMap reconstructed via recursive `holon_item_to_value` Bundle dispatch on all-Bind shape. |
| 13 | Probe 8 — Mixed nesting (Vec) | PASS | `probe_8_mixed_nesting_hashmap_of_vec` — `HashMap<keyword, Vec<i64>>`; outer length = 1; outer Bundle has 1 Bind child. Inner Bundle is positional-Bind (array-shape, Stone 216.2); composes correctly. |
| 14 | Probe 9 — Mixed nesting (HashSet) | PASS | `probe_9_mixed_nesting_hashmap_of_hashset` — `HashMap<keyword, HashSet<i64>>`; outer length = 1; outer Bundle has 1 Bind child. Inner Bundle is bare-atom (set-shape, Stone 216.1); composes correctly. |
| 15 | Probe 10 — Check passes | PASS | `probe_10_check_passes_atomizable_k_v` — `(:wat::holon::Atom {:a 1})` compiles and runs. Nested `HashMap<keyword, HashMap<keyword, i64>>` also passes (predicate recurses both levels via pre-landed is_atomizable). |
| 16 | Probe 11 — Check fails | PASS | `probe_11_check_fails_non_atomizable` — `(:wat::holon::Atom f)` where `f: Fn([i64])->i64` fails at check with `TypeMismatch` naming `:wat::holon::Atom`. Function types not in the atomizable set. |
| 17 | Probe 12 — HolonRepresentable cascade | PASS | `probe_12_holon_representable_cascade` — `assert_holon_representable::<HashMap<String, String>>()` compiles. `to_holon_ast`/`from_holon_ast` round-trip: `{foo→bar, baz→qux}` → Bundle of 2 Bind(String, String) children → `{foo→bar, baz→qux}`. Also `HashMap<String, Vec<String>>` compile-time check + round-trip. |
| 18 | Probe 13 — Shape disambiguation | PASS | `probe_13_shape_disambiguation_non_sequential_i64` — `Vec<String>::from_holon_ast` on Bundle with Bind(I64(0), String) + Bind(I64(5), String) returns Err (positional invariant violated — confirmed Vec path rejects). WAT-surface: `HashMap<i64,String>` with keys 0+5 → Atom → atom-value → HashMap (length 2; not Vec). Non-sequential I64 Bind keys fall through to HashMap path. |
| 19 | Probe 14 — Empty Bundle disambiguation | PASS | `probe_14_empty_bundle_disambiguation_consumer_declares_hashmap` — unannotated `(atom-value h)` on empty Bundle → empty HashSet (conservative, length 0 via `HashSet/length`). Annotated `(atom-value h -> :wat::core::HashMap)` → empty HashMap (length 0 via `HashMap/length`). Consumer-declared type overrides conservative HashSet default. |

## Deltas from EXPECTATIONS

**Delta 1 — `-> :T` annotation uses bare `:wat::core::HashMap` keyword (not parametric).**
The BRIEF describes "consumer's `-> :T` annotation declares `T = :wat::core::HashMap<K, V>`". WAT's lexer rejects space-separated type args inside `<>` (rule: no whitespace inside angle brackets in keywords). The annotation therefore uses the bare `:wat::core::HashMap` keyword (no type params). The runtime checks `k.starts_with(":wat::core::HashMap")` — both bare and any future parametric form (if lexer allows no-space variants like `:wat::core::HashMap<K,V>`) would match. The disambiguation is binary (HashMap vs not-HashMap); K+V type params are irrelevant for the empty-Bundle case.

**Delta 2 — `atom-value` extended to 1-or-3-arg form; special-case arm added in check.rs.**
The BRIEF described the `-> :T` form as a runtime-level hint without specifying checker changes. A special-case arm was added in `infer_list` for `:wat::core::atom-value` accepting 1 or 3 args. Without it, the 3-arg form would fail type-checking with ArityMismatch. The arm validates that arg[0] is HolonAST and returns a fresh type variable (same as the existing generic scheme). The `->` and type keyword args are syntactic decoration — not type-checked as expressions.

**Delta 3 — Stone 216.2's non-sequential-I64 path changed from Error to HashMap.**
Stone 216.2 SCORE (Row 2) documented: "TypeMismatch on non-sequential keys". Stone 216.3 supersedes this: non-sequential I64 keys now fall through to the HashMap path (no error). The `eval_atom_value` and `holon_item_to_value` Bundle arms both restructured: the sequential check now branches to Vec (yes) or HashMap (no) rather than Vec (yes) or Error (no). This is the correct Stone 216.3 behavior (Probe 13). Stone 216.2's probe 12 (Rust-level `Vec<String>::from_holon_ast`) still validates the positional invariant and returns Err — that code path is unchanged.

**Delta 4 — `is_atomizable` for HashMap pre-landed in Stone 216.1 (confirmed, no change).**
Stone 216.1 Delta 6 noted the HashMap arm was pre-emptively included. Confirmed at `src/check.rs:3644`. The comment `-- arc 216 Stone 3 (future)` was updated to `-- arc 216 Stone 3` in the cheatsheet. The check.rs comment itself retains its original wording (honesty: not retroactively edited).

**Delta 5 — `holon_item_to_value` all-Bind check uses a separate `all_binds` predicate.**
The BRIEF said "handle Bind(K, V) for nested HashMap values (mirrors 216.2's Bind(I64) handling)". The implementation adds a separate `all_binds` check: `items.iter().all(|child| matches!(child, HolonAST::Bind(_, _)))`. This is checked AFTER the `all_i64_binds` branch, so the dispatch order is: sequential-I64 → Vec; all-Bind (non-sequential-I64 or non-I64 K) → HashMap; empty → HashSet; bare atoms → HashSet.

## Verification summary

```
cargo build --release                                              — OK (0 errors, 5 pre-existing warnings)
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip — 14/14 PASS
cargo test --release --test probe_arc216_stone2_vector_roundtrip  — 12/12 PASS (no regression)
cargo test --release --test probe_arc216_stone1_hashset_roundtrip — 10/10 PASS (no regression)
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio — 18/18 PASS
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio — 15/15 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias — 6/6 PASS
cargo test --release --test probe_arc215_stone2 — 13/13 PASS
cargo test --release --test probe_arc215_collection_literal_inference — 12/12 PASS
cargo test --release --test probe_brace_map_literal — 9/9 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric — 9/9 PASS
cargo clippy --release -- -D warnings — 111 pre-existing errors; 0 new errors from this stone
```

**Pre-existing failure:** `wat_arc170_slice_1f_alpha_helpers` — crossbeam vs wat typed_channel mismatch. Pre-existing before arc 216; not introduced by these changes.

## Files changed

- `src/runtime.rs` — `value_to_atom` (HashMap arm); `holon_item_to_value` (three-way Bundle dispatch: Vec/HashMap/HashSet); `eval_atom_value` (three-way Bundle dispatch + optional 1-or-3-arg form for `-> :T` disambiguation)
- `src/check.rs` — special-case arm for `:wat::core::atom-value` in `infer_list` (1-or-3-arg; validates first arg HolonAST; returns fresh type var)
- `src/comms/mod.rs` — `impl<K, V> HolonRepresentable for std::collections::HashMap<K, V>`
- `tests/probe_arc216_stone3_hashmap_roundtrip.rs` — 14 probes (new file)
- `docs/WAT-CHEATSHEET.md` — HashMap atomizable entry (Stone 3 landed); encoding shape section extended (map-shape example + atom-value `-> :T` form); shape discriminator table extended; predicate examples extended; STOP-1 note updated to reflect full resolution
- `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.3.md` — this file

## Elapsed time

Target: 60-90 min. Actual: ~65 min. Within prediction band.

## What was discovered

1. **WAT keyword angle-bracket restriction blocks parametric `-> :T` form.** WAT's lexer rejects whitespace inside `<>` in keywords. `(:wat::core::atom-value h -> :wat::core::HashMap<:wat::core::keyword :wat::core::i64>)` fails with `UnclosedBracketInKeyword`. The practical solution: use the bare `:wat::core::HashMap` keyword as the hint — the disambiguation is binary (is it HashMap?) and doesn't need K/V param info.

2. **Stone 216.2's non-sequential-I64 error path superseded by Stone 216.3's HashMap path.** Stone 216.2's `eval_atom_value` emitted TypeMismatch on non-sequential I64 Bind keys. Stone 216.3 changes this: non-sequential I64 keys → HashMap (the sequential check now discriminates Vec vs HashMap, not Vec vs Error). This is a clean semantic extension: the error case is now a valid collection shape (HashMap with i64 keys). Stone 216.2's Rust-level `Vec<T>::from_holon_ast` still validates the positional invariant and returns Err — that is unchanged.

3. **`holon_item_to_value` needed the same full three-way dispatch as `eval_atom_value`.** The function is the recursive helper for nested collection items. Extending it to handle all-Bind → HashMap was symmetric with Stone 216.2's all-I64-Bind → Vec extension. The dispatch order matters: `all_i64_binds` (sequential check) → `all_binds` (HashMap) → bare atoms (HashSet).

4. **`is_atomizable` for HashMap was correctly pre-landed in Stone 216.1.** No change needed. Stone 216.1 Delta 6 anticipated exactly this stone.

5. **`check.rs` needed a special-case arm for the 3-arg `atom-value` form.** Without it, the generic scheme path would fail with ArityMismatch on `(atom-value h -> :T)`. The special-case arm accepts 1 or 3 args, validates arg[0] as HolonAST, and returns a fresh type var (same as the scheme's `∀T` behavior). The `->` and type keyword args are syntactic annotation; not type-checked as WAT expressions.

6. **`Value::wat__std__HashMap` stores `(key_Value, val_Value)` pairs.** The forward trip iterates `.values()` to get both the original key Value and value Value for recursive atomization. The reverse trip reconstructs both via `holon_item_to_value` and re-inserts using `hashmap_key(k_val)` for the canonical string key.
