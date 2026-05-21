# SCORE — Arc 216 Stone 5 — `hashmap_key` full coverage (substrate fix)

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-20

## Result: 16/16 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `hashmap_key` audit | PASS | `src/runtime.rs:9330` — verified gap matches exactly {Vec, HashMap, WatAST}. No additional gaps found. Arms present pre-stone: String, i64, f64, bool, keyword, HolonAST, Uuid, HashSet. Missing: Value::Vec, Value::wat__std__HashMap, Value::wat__WatAST. Caller audit: 18 call sites at lines 7651, 7653, 7673, 7688, 7788, 7907, 8127, 8368, 8435, 8453, 9419, 9468, 12760, 12776, 12789, 12951, 12966, 12980 — all pass `&Value` directly without pre-filtering; all benefit uniformly from new arms. STOP-3 not triggered. |
| 2 | `Value::Vec` arm added | PASS | `src/runtime.rs` — `Value::Vec(xs)` arm: length-prefix canonical key scheme. Each element's recursive `hashmap_key` result is prefixed by its byte length, joined with commas: `"Vec:[{len1}:{k1},{len2}:{k2},...]"`. Order preserved (Vec is ordered; no sort step). Scheme chosen for collision safety (see Probe 11). |
| 3 | `Value::wat__std__HashMap` arm added | PASS | `src/runtime.rs` — `Value::wat__std__HashMap(m)` arm: sorted-pairs canonical key. Iterates `(canonical_key_string, (k_val, v_val))` pairs; recurses `hashmap_key` on both `k_val` and `v_val`; sorts by canonical key for determinism; formats as `"Map:{(k1=v1),(k2=v2),...}"`. |
| 4 | `Value::wat__WatAST` arm added | PASS | `src/runtime.rs` — `Value::wat__WatAST(ast)` arm: mirrors HolonAST pattern (lines 9337-9343) but uses `format!("{:?}", ast)` as hash input because WatAST does not implement Hash directly (only derives Debug + Clone + PartialEq). DefaultHasher; `"W:{hash:x}"`. WatAST's Debug is span-agnostic structural representation. |
| 5 | Diagnostic message updated | PASS | `src/runtime.rs:9356` — `other =>` arm now emits `"hashable value (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)"`. Enumerates the full accepted set honestly. |
| 6 | All `hashmap_key` callers audited | PASS | 18 call sites grepped; none pre-filter on Value variant before passing to `hashmap_key`. All benefit uniformly from the three new arms. No caller blocks the new arms from helping. |
| 7 | `tests/probe_verify_hashset_of_vector_gap.rs` flips GREEN | PASS | Previously: `RUNTIME FAILED: :wat::core::HashSet: expected hashable value (primitive, HolonAST, or HashSet<T>), got wat::core::Vector`. After: `RUNTIME OK: HashSet<Vector<i64>> produced value i64(2)`. Same probe, same assertion. |
| 8 | Probe 1 — HashSet<Vector<i64>> round-trip | PASS | `tests/probe_arc216_stone5_hashmap_key_coverage.rs:probe_1_hashset_of_vector_roundtrip`. Forward: 2-element HashSet<Vector<i64>> → Atom → Bundle with 2 children. Reverse: atom-value → HashSet length = 2. |
| 9 | Probe 2 — HashSet<HashMap<keyword, i64>> round-trip | PASS | `probe_2_hashset_of_hashmap_roundtrip`. Two distinct HashMaps → outer HashSet → Atom → Bundle with 2 children. atom-value → length = 2. |
| 10 | Probe 3 — HashSet<WatAST> round-trip | PASS | `probe_3_hashset_of_watast_roundtrip`. WatAST values constructed via `(:wat::core::quote :foo)` and `(:wat::core::quote :bar)`. Two distinct quoted AST nodes → HashSet<WatAST> → Atom → Bundle with 2 children. WatAST IS constructible at WAT surface via `quote`. |
| 11 | Probe 4 — HashMap<Vector<i64>, String> round-trip | PASS | `probe_4_hashmap_vector_key_roundtrip`. Vector as K; `Infer` for K type param (parameterized type keywords not valid in HashMap constructor position). 1-entry map → Atom → Bundle with 1 child; contains-key? returns true. |
| 12 | Probe 5 — HashMap<HashMap<keyword, i64>, String> round-trip | PASS | `probe_5_hashmap_hashmap_key_roundtrip`. Inner HashMap as outer K; `Infer` for K type param. 1-entry map → Atom → Bundle with 1 child; contains-key? returns true. |
| 13 | Probe 6 — HashMap<WatAST, String> round-trip | PASS | `probe_6_hashmap_watast_key_roundtrip`. Quoted AST as K. 1-entry map → Atom → Bundle with 1 child; contains-key? returns true. |
| 14 | Probe 7 — Nested HashSet<Vector<HashSet<i64>>> | PASS | `probe_7_nested_hashset_vector_hashset`. Three-deep nesting: HashSet elem → Vec arm → Vec elem → HashSet arm → HashSet elem → i64 arm. One outer element → Bundle with 1 child. All three recursive arms compose. |
| 15 | Probe 11 — Collision-safety | PASS | `probe_11_collision_safety_length_prefix`. `["a","b,c"]` → `Vec:[3:S:a,5:S:b,c]`; `["a,b","c"]` → `Vec:[5:S:a,b,3:S:c]`. Distinct canonical keys → HashSet length = 2 (not 1). Length-prefix scheme is collision-safe. |
| 16 | 216.4 Probe 3 relanded + SCORE | PASS | `tests/probe_arc216_stone4_predicate_composition.rs` — `probe_3_composite_hashset_of_hashset` renamed to `probe_3_composite_hashset_of_vector`; type flipped back from `HashSet<HashSet<i64>>` to `HashSet<Vector<i64>>` per original BRIEF; doc-comment updated to acknowledge the Stone 216.4 Delta 2 substitution and the 216.5 reland. 6/6 still pass. SCORE-STONE-216.5.md inscribed (this file). |

## Deltas from EXPECTATIONS

**Delta 1 — Canonical-key scheme choice: length-prefix for Vec.**
BRIEF said "sonnet picks; documents the choice in the doc-comment." Scheme chosen:
length-prefix. Each element's recursive `hashmap_key` result is prefixed by its
byte length before joining with commas: `"Vec:[{len1}:{k1},{len2}:{k2},...]"`.
This is provably collision-safe: given any two distinct Vec values V1 and V2, their
canonical keys differ because either (a) the lengths differ, or (b) at the first
position where V1[i] ≠ V2[i], the prefixed representations `"{len}:{k}"` differ.
Probe 11 demonstrates the specific case that motivated the choice.

**Delta 2 — WatAST uses Debug-based hashing, not Hash trait.**
BRIEF said "mirror HolonAST's `Hash`+DefaultHasher pattern." HolonAST implements
`Hash` (structural derive in holon-rs). WatAST only derives `Debug, Clone, PartialEq`
(no Hash impl anywhere in the codebase). The arm uses `format!("{:?}", ast)` as the
hash input — WatAST's Debug output is structurally complete and span-agnostic (its
PartialEq is span-agnostic per arc 1818 comment in the codebase). DefaultHasher;
`"W:{hash:x}"`. The scheme is functionally equivalent to a Hash impl for this use case.

**Delta 3 — Probes 4/5/8: `:wat::type::Infer` for collection K type parameter.**
BRIEF specified `HashMap<Vector<i64>, String>` syntax. The HashMap constructor
`(:wat::core::HashMap :K :V k v)` accepts only simple keyword type arguments in the
`:K` and `:V` positions — parameterized forms like `:wat::core::Vector<:wat::core::i64>`
are rejected with "first type argument X is not a valid type keyword." This is correct
substrate behavior (arc 214 P1 introduced the `:K :V` form; parameterized keywords in
that position were never supported). Probes use `:wat::type::Infer` for the collection
K type; the type is inferred from the provided key argument. The contains-key? behavior
(the round-trip correctness property) is verified regardless. No gap introduced; surface
syntax limitation documented.

**Delta 4 — Probe 8: both K and V use `:wat::type::Infer`.**
Same surface syntax reason as Delta 3. `HashMap<Vector<i64>, HashSet<i64>>` cannot be
spelled with parameterized type keywords in the constructor position. Both `:K` and `:V`
use `Infer`; the types are correctly inferred from the provided key and value arguments.

**Delta 5 — Probe 10 (diagnostic) uses inline fn, not a top-level define-with-params.**
The probe needs a non-hashable value to trigger the `other =>` arm at runtime. The
diagnostic probe originally defined a top-level function with params
`(:user::f -> :wat::core::i64 [n :wat::core::i64])` — that syntax is malformed
(return type annotation after name before params is rejected). Probe uses an inline
`(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)` literal, same as
Stone 216.4 Probe 5. The diagnostic arm fires identically for any non-structural value.

## Audit findings

**Gap confirmed as claimed:** exactly {Vec, HashMap, WatAST} — no additional missing arms
found. STOP-3 did not trigger.

**Caller audit:** 18 `hashmap_key` call sites, all pass `&Value` without type pre-filtering.
All benefit uniformly. No blocker found.

**WatAST constructibility:** confirmed — `(:wat::core::quote expr)` produces `Value::wat__WatAST`.
Probes 3 and 6 both exercise WatAST as set element and map key respectively. Both pass.

**HashMap constructor surface syntax:** parameterized type keywords in `:K`/`:V` position
are not valid WAT syntax (surface limitation; substrate is correct). Probes adapted to use
`:wat::type::Infer`; the hash correctness is still verified via round-trip and contains-key?.

## Canonical-key scheme choice

**Vec: length-prefix.** `"Vec:[{byte_len_of_ki}:{ki},...]"`.

Rejected: naive comma-join — fails probe 11 (collides `["a","b,c"]` with `["a,b","c"]`).

Accepted: length-prefix — provably collision-safe. The length byte count is a separator
that disambiguates element boundaries. Two different Vec values cannot produce the same
canonical key because the length prefix encodes the exact byte width of each element key,
making the parsing of the composite string unambiguous.

**HashMap: sorted-pairs.** `"Map:{(k1=v1),(k2=v2),...}"` sorted by canonical key string.
Deterministic (HashMap has no order; sort produces stable composite). Recursive on both K and V.

**WatAST: Debug-based hash.** `"W:{DefaultHasher_over_format_debug:x}"`.
Honest: WatAST has no Hash impl. The Debug output is the only span-agnostic structural
representation available. Two structurally distinct WatAST values produce distinct Debug
strings and thus (with overwhelming probability) distinct hash values.

## Verification summary

```
cargo build --release                                                              — OK (0 errors, 5 pre-existing warnings)
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat             — 1/1 PASS (was RED; now GREEN)
cargo test --release --test probe_arc216_stone5_hashmap_key_coverage -p wat       — 12/12 PASS (new file)
cargo test --release --test probe_arc216_stone4_predicate_composition -p wat      — 6/6 PASS (Probe 3 relanded)
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip -p wat          — 14/14 PASS (no regression)
cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat           — 12/12 PASS (no regression)
cargo test --release --test probe_arc216_stone1_hashset_roundtrip -p wat          — 10/10 PASS (no regression)
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio -p wat        — 18/18 PASS
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat        — 15/15 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat — 6/6 PASS
cargo test --release --test probe_arc215_stone2 -p wat                            — 13/13 PASS
cargo test --release --test probe_arc215_collection_literal_inference -p wat      — 12/12 PASS
cargo test --release --test probe_brace_map_literal -p wat                        — 9/9 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat            — 9/9 PASS
cargo clippy --release -- -D warnings                                              — 111 pre-existing errors; 0 new errors from this stone
```

**Pre-existing failure:** `wat_arc170_slice_1f_alpha_helpers` — crossbeam vs wat typed_channel
mismatch. Pre-existing before arc 216; not introduced by this stone.

## Files changed

- `src/runtime.rs` — `hashmap_key` function: three new arms (`Value::Vec`, `Value::wat__std__HashMap`, `Value::wat__WatAST`); updated doc-comment with canonical-key scheme descriptions; updated `other =>` diagnostic message
- `tests/probe_arc216_stone5_hashmap_key_coverage.rs` — 12 probes (new file)
- `tests/probe_verify_hashset_of_vector_gap.rs` — unchanged (flips green by the runtime fix)
- `tests/probe_arc216_stone4_predicate_composition.rs` — Probe 3 relanded: `probe_3_composite_hashset_of_hashset` → `probe_3_composite_hashset_of_vector`; type flipped to `HashSet<Vector<i64>>`; doc-comment updated
- `docs/WAT-CHEATSHEET.md` — new "Hashable types" subsection after the atomizable-types section; canonical-key scheme table; collision-safety example; updated diagnostic message
- `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5.md` — this file

## Elapsed time

Target: 60-90 min. Actual: ~70 min. Within prediction band.

## What was discovered

1. **WatAST is constructible at WAT surface via `(:wat::core::quote expr)`.** Probes 3 and 6
   both work without any substrate change. The `quote` form produces `Value::wat__WatAST`
   at runtime.

2. **WatAST has no Hash impl.** Only `Debug, Clone, PartialEq`. The Debug-based hashing
   approach is the honest path. The doc-comment explains why.

3. **HashMap constructor surface syntax does not accept parameterized type keywords.**
   `(:wat::core::HashMap :wat::core::Vector<:wat::core::i64> :wat::core::String k v)` is
   rejected. `:Infer` is the correct form for collection type arguments. This is not a gap
   in the substrate; it is correct behavior (simple keyword args only in `:K`/`:V` position).

4. **Length-prefix scheme is the right Vec canonical-key choice.** Probe 11 gates this
   explicitly. Naive comma-join fails probe 11 (demonstrated by the fact that
   `["a","b,c"]` and `["a,b","c"]` would produce the same string under comma-join).
   Length-prefix is provably safe.

5. **The arc 216 thesis is now true on the branch.** `HashSet<Vector<T>>`,
   `HashSet<HashMap<K,V>>`, `HashSet<WatAST>`, `HashMap<Vector<T>, V>`,
   `HashMap<HashMap<K,V>, V>`, `HashMap<WatAST, V>` all round-trip cleanly at the
   WAT surface. Every atomizable type is hashable via `hashmap_key`. The
   predicate→runtime contract holds.

6. **Stone 216.4 Delta 2 substitution reversed.** `probe_3_composite_hashset_of_vector`
   now tests the original BRIEF's type. The gap that forced the substitution is closed.
