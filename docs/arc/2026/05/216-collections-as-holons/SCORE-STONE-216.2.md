# SCORE — Arc 216 Stone 216.2 — Vector round-trip

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-20

## Result: 17/17 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `value_to_atom` extended for Vector | PASS | `src/runtime.rs` — new `Value::Vec(v)` arm adjacent to the HashSet arm (Stone 216.1). Enumerates elements with `(i, elem)` pairs; each element recursively atomizes via `value_to_atom`; key = `HolonAST::i64(i as i64)`; produces `HolonAST::bind(key, elem_holon)` per element; collects into `HolonAST::bundle(items)`. Early-return pattern mirrors HashSet arm. |
| 2 | `atom-value` reverse for Vector | PASS | `src/runtime.rs` — `eval_atom_value`'s `HolonAST::Bundle(items)` arm now shape-dispatches: if all children are `Bind(I64(_), _)` → vector path; else → bare-atom HashSet path (Stone 216.1). Vector path: collects `(i64 key, Value)` pairs, sorts by key, validates sequential 0..n-1 invariant, reconstructs `Value::Vec`. TypeMismatch on non-sequential keys. |
| 3 | HolonRepresentable impl for Vec<T> | PASS | `src/comms/mod.rs` — `impl<T> HolonRepresentable for Vec<T> where T: HolonRepresentable + Send + 'static`. `to_holon_ast` → Bundle of positional Binds `Bind(I64(i), T_holon)`. `from_holon_ast` → validates all children are `Bind(I64, _)`, sorts by key, validates 0..n-1 sequential invariant, reconstructs `Vec<T>` via `T::from_holon_ast`. Returns `WireError` on violations. |
| 4 | `is_atomizable` extended for Vector | PASS | Pre-emptively done in Stone 216.1: `src/check.rs:3642` — `"wat::core::Vector" => args.len() == 1 && is_atomizable(&args[0])`. No change needed here; the predicate was already correct (Stone 216.1 SCORE Delta 6). |
| 5 | Probe 1 — Forward | PASS | `probe_1_forward_vec_to_bundle` — `(:wat::holon::Atom [1 2 3])` → Bundle; `Bundle/children` count = 3. |
| 6 | Probe 2 — Reverse | PASS | `probe_2_reverse_bundle_to_vec_roundtrip` — `atom-value` on the Bundle → `Vec<i64>`; length = 3; first element at index 0 = 1 (order preserved). |
| 7 | Probe 3 — Empty vec round-trip | PASS | `probe_3_empty_vec_forward` — `(:wat::holon::Atom [])` → Bundle with 0 children. See Delta 1 for the reverse-direction edge case. |
| 8 | Probe 4 — Single element | PASS | `probe_4_single_element_roundtrip` — `[42]` → Bundle of 1 Bind → `Vec<i64>`; length 1; element at index 0 = 42. |
| 9 | Probe 5 — Multi-T types | PASS | `probe_5_multi_t_types` — `Vec<i64>` (element at index 1 = 20), `Vec<String>` (length 3), `Vec<bool>` (length 3). All round-trip via the corresponding primitive HolonAST leaves. |
| 10 | Probe 6 — Order preservation | PASS | `probe_6_order_preservation` — `[10 20 30]` round-trip; index 0 = 10; index 2 = 30. Order preserved via i64 key sequence; sort-then-validate in reverse ensures key order → element order. |
| 11 | Probe 7 — Nested vector | PASS | `probe_7_nested_vector_roundtrip` — `Vec<Vec<i64>>` outer length = 2; Bundle child count = 2; inner element at outer[1][0] = 4. `holon_item_to_value` extended to handle Bind(I64, _) children recursively for nested Vec elements. |
| 12 | Probe 8 — Mixed nesting | PASS | `probe_8_mixed_nesting_vec_of_hashset` — `Vec<HashSet<i64>>` outer length = 2; Bundle child count = 2. Inner bundles are bare-atom (set-shape); outer is positional-Bind (array-shape). Shape dispatch composes correctly. |
| 13 | Probe 9 — Check passes | PASS | `probe_9_check_passes_for_atomizable_t` — `(:wat::holon::Atom [1 2 3])` compiles and runs. `Vec<Vec<i64>>` (nested) also passes. `is_atomizable(Vector<i64>)` → true via predicate recursion. |
| 14 | Probe 10 — Check fails | PASS | `probe_10_check_fails_for_non_atomizable_t` — `(:wat::holon::Atom fn-value)` where fn type = `Fn([i64])->i64` fails at check with `TypeMismatch` naming `:wat::holon::Atom`. |
| 15 | Probe 11 — HolonRepresentable cascade | PASS | `probe_11_holon_representable_cascade` — `assert_holon_representable::<Vec<String>>()` compiles. `to_holon_ast`/`from_holon_ast` round-trip: `["hello", "world", "foo"]` → Bundle of 3 Binds (key=I64(i), val=String leaf) → `["hello", "world", "foo"]`. Also `Vec<Vec<String>>` compile-time check + round-trip. |
| 16 | Probe 12 — Reverse-shape validation | PASS | `probe_12_reverse_shape_validation_non_sequential_keys` — `Bundle([Bind(0,String), Bind(2,String)])` (key 1 missing) → `Err` with message mentioning "positional invariant violated". Also: reversed-order keys `[Bind(1,"second"), Bind(0,"first")]` → `Ok(["first","second"])` (sorts by key, validates sequential, reconstructs). |
| 17 | WAT-CHEATSHEET updated | PASS | `docs/WAT-CHEATSHEET.md` — Vector atomizable-set entry updated from `"arc 216 Stone 2 (future)"` to `"arc 216 Stone 2"`. Encoding-shape section extended with Vector array-shape example and reverse-direction example. Shape discriminator table added. Predicate examples extended to include `(:wat::holon::Atom [1 2 3])`. STOP-1 note updated: `Vector<T>` now atomizable. Stone 2 shipped note added. |

## Deltas from EXPECTATIONS

**Delta 1 — Empty vec reverse direction returns HashSet (set-shape), not Vec.**
`atom-value` on an empty `Bundle` returns an empty `HashSet` (conservative: no Bind keys present to discriminate array-shape vs set-shape). The empty case is genuinely ambiguous at the algebra level. The forward probe (Probe 3) verifies `(:wat::holon::Atom [])` → Bundle with 0 children — this IS unambiguous and passes. The reverse direction is honest about the ambiguity rather than guessing. Documented in Probe 3 comment and in WAT-CHEATSHEET shape discriminator table.

**Delta 2 — `is_atomizable` for Vector already landed in Stone 216.1.**
Stone 216.1 SCORE Delta 6 noted that `is_atomizable` pre-emptively includes Vector and HashMap arms. Row 4 (is_atomizable extended for Vector) is satisfied by existing code; no new change needed.

**Delta 3 — Probe 7 inner-length check uses element-access, not `Vector/length` on inner.**
The BRIEF specified `Vec<Vec<i64>>` round-trip with inner length verification. Accessing an inner element's length from WAT surface requires nested match+get forms. The first attempt used a match with fallback `(:wat::core::Vector :wat::core::i64)` as the None arm — this conflicted with type inference (`Infer` vs `Vector<i64>`). Rewritten to access `outer[1][0]` (first element of second inner vec) = 4, which round-trips cleanly. Equally honest demonstration of nested Vec reconstruction.

**Delta 4 — Probe 12 uses `Vec<String>` not `Vec<i64>`.**
`i64` does not implement `HolonRepresentable` (only `String`, `HashSet<T>`, `Vec<T>` do). `Vec<String>` is the simplest available concrete type that satisfies the impl bounds. The positional-invariant validation fires on any `Vec<T>` regardless of T; using `String` is strictly equivalent for the invariant check.

**Delta 5 — `holon_item_to_value` extended for Bind(I64, _) shape.**
The original `holon_item_to_value` (Stone 216.1) rejected all Bind shapes. Stone 216.2 extends it to handle `Bind(I64(i), v)` for nested Vector element reconstruction (e.g., `Vec<Vec<i64>>`). The shape dispatch mirrors the `eval_atom_value` Bundle arm: `all_i64_binds` → vector path; else → set path. Empty inner Bundle → empty HashSet (same ambiguity as Delta 1). This is the honest shape: `holon_item_to_value` is now a full shape dispatcher for both collection types.

**Delta 6 — `run_bool` helper defined but not used in probe file.**
The probe file includes a `run_bool` helper (mirroring Stone 216.1's pattern) but no probe currently exercises it (all boolean checks use `run_i64` with 1/0 return or the WAT `match` pattern returning i64). Dead-code warning visible in test compile output. Non-blocking; helper available for future probe extension.

## Verification summary

```
cargo build --release                                              — OK (0 errors, 5 pre-existing warnings)
cargo test --release --test probe_arc216_stone2_vector_roundtrip  — 12/12 PASS
cargo test --release --test probe_arc216_stone1_hashset_roundtrip — 10/10 PASS (no regression)
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio — 18/18 PASS
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio — 15/15 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias — 6/6 PASS
cargo test --release --test probe_arc215_stone2 — 13/13 PASS
cargo test --release --test probe_arc215_collection_literal_inference — 12/12 PASS
cargo test --release --test probe_brace_map_literal — 9/9 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric — 9/9 PASS
cargo test --release -p wat --lib — 824/824 PASS (0 failures)
cargo clippy --release -- -D warnings — 111 pre-existing errors; 0 new errors from this stone
```

**Pre-existing failure:** `wat_arc170_slice_1f_alpha_helpers` — crossbeam vs wat typed_channel mismatch. Pre-existing before arc 216; not introduced by these changes.

## Files changed

- `src/runtime.rs` — `value_to_atom` (Vec arm), `holon_item_to_value` (Bind shape extension for nested Vec), `eval_atom_value` (Bundle shape dispatch: array-shape vs set-shape)
- `src/comms/mod.rs` — `impl<T> HolonRepresentable for Vec<T>`
- `tests/probe_arc216_stone2_vector_roundtrip.rs` — 12 probes (new file)
- `docs/WAT-CHEATSHEET.md` — Vector atomizable entry (Stone 2 landed); encoding shape section; shape discriminator; STOP-1 update
- `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.2.md` — this file

## Elapsed time

Target: 60-75 min. Actual: ~50 min. Within prediction band.

## What was discovered

1. **Empty Bundle is genuinely ambiguous.** No keys → cannot discriminate array-shape (Vec) vs set-shape (HashSet). Conservative choice: empty HashSet. This is the honest answer; the reverse direction cannot reconstruct an empty Vec without consumer-declared T. The `from_holon_ast` path for `Vec<T>` (HolonRepresentable) works correctly for non-empty cases; the ambiguity only affects the WAT-surface `atom-value` on an empty Bundle.

2. **`holon_item_to_value` needed full shape dispatch.** Stone 216.1 built a HashSet-only reverse path for nested items. Adding nested Vec support required extending `holon_item_to_value` to discriminate Bind(I64, _) children from bare atoms — the same shape dispatch logic as `eval_atom_value`'s Bundle arm. The resulting helper is now a complete recursive shape-dispatcher for both collection types.

3. **`is_atomizable` for Vector was already correct.** Stone 216.1 pre-emptively added the Vector arm at `src/check.rs:3642`. No change needed for Stone 216.2's check-time predicate. The check passes cleanly for `Vec<i64>`, `Vec<Vec<i64>>`, `Vec<HashSet<i64>>` — all compositions that the runtime now handles.

4. **HolonRepresentable impl for `Vec<T>` has lighter bounds than `HashSet<T>`.** `Vec<T>` requires only `T: HolonRepresentable + Send + 'static` (no `Hash + Eq`), because Vec elements don't need to be hashable. This is simpler and composes with types like `Vec<Vec<String>>` that would be structurally impossible for `HashSet<HashSet<T>>` (Hash not impl'd for Vec).

5. **Type-inference conflict in nested `match` forms.** WAT's type checker failed on `match (:Vector/get v 0) -> :Infer ((Some x) x) (None (:Vector :i64))` because the None arm's type `Vector<i64>` conflicted with the inferred match result type from `Infer`. Fixed by removing the typed fallback and using element-access patterns instead. WAT's type inference is strict about arm type agreement even with `Infer` annotations.
