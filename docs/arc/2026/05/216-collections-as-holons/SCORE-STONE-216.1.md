# SCORE — Arc 216 Stone 216.1 — HashSet round-trip

**Mode:** A
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-20

## Result: 15/15 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `value_to_atom` extended for HashSet | PASS | `src/runtime.rs:12790` — new `Value::wat__std__HashSet(s)` arm; iterates `s.values()`, recursively atomizes each via `value_to_atom`, collects into `HolonAST::bundle(items)`. Early-returns to avoid the outer match's bare-HolonAST result type. |
| 2 | `:wat::core::atom-value` reverse for HashSet | PASS | `src/runtime.rs:12783` — new `HolonAST::Bundle(items)` arm in `eval_atom_value`; iterates bundle children via `holon_item_to_value` helper (handles primitive leaves + nested Bundles recursively), inserts into `HashMap<String, Value>`, returns `Value::wat__std__HashSet`. |
| 3 | `HolonRepresentable` impl for HashSet | PASS | `src/comms/mod.rs:142` — `impl<T> HolonRepresentable for std::collections::HashSet<T> where T: HolonRepresentable + std::hash::Hash + Eq + Send + 'static`. `to_holon_ast` → Bundle of T children; `from_holon_ast` → match Bundle, reconstruct via `T::from_holon_ast` per item. |
| 4 | check.rs atomizable predicate extended | PASS | `src/check.rs:3600` — `fn is_atomizable(ty: &TypeExpr) -> bool` (DESIGN Q6 predicate). `src/check.rs:5305` — new `:wat::holon::Atom` \| `:wat::holon::leaf` special-case arm in `infer_list`; infers arg type, applies `is_atomizable(resolved)`, emits `TypeMismatch` on failure. Returns `:wat::holon::HolonAST`. |
| 5 | Probe 1 — Forward | PASS | `Bundle/children` length = 3 for `(:wat::holon::Atom #{1 2 3})`. |
| 6 | Probe 2 — Reverse | PASS | `atom-value` on the Bundle reconstructs `HashSet<i64>`; length = 3; contains element 2. |
| 7 | Probe 3 — Empty set round-trip | PASS | `#{}` → `Bundle([])` → `HashSet<T>`; length = 0. |
| 8 | Probe 4 — Single element | PASS | `#{42}` → `Bundle([I64(42)])` → `#{42}`; length = 1; contains 42. |
| 9 | Probe 5 — Multi-T types | PASS | `HashSet<i64>` (contains 20), `HashSet<String>` (length 3), `HashSet<bool>` (length 2). |
| 10 | Probe 6 — Dedupe semantic | PASS | `#{1 1 2 2 3}` deduplicates at construction → 3-element Bundle → `atom-value` reconstructs length 3. |
| 11 | Probe 7 — Nested set | PASS | `HashSet<HashSet<i64>>` via WAT surface (`(HashSet :Infer inner1 inner2)`): outer Bundle has 2 children; `atom-value` reconstructs outer length 2. `holon_item_to_value` recursion handles nested Bundles → nested HashSets. |
| 12 | Probe 8 — Check passes | PASS | `(:wat::holon::Atom #{1 2 3})` compiles and runs; `is_atomizable(HashSet<i64>)` → YES via predicate recursion. Nested `HashSet<HashSet<i64>>` also passes. |
| 13 | Probe 9 — Check fails | PASS | `(:wat::holon::Atom f)` where `f: Fn([i64])->i64` fails at check with `TypeMismatch` naming `:wat::holon::Atom`; `is_atomizable(Fn{...})` → false. |
| 14 | Probe 10 — HolonRepresentable cascade | PASS | `assert_holon_representable::<HashSet<String>>()` compiles. `to_holon_ast`/`from_holon_ast` round-trip verified: `{hello, world}` → `Bundle([String, String])` → `{hello, world}`. |
| 15 | WAT-CHEATSHEET updated | PASS | `docs/WAT-CHEATSHEET.md` — new "Atomizable types and `(:wat::holon::Atom T)`" subsection after Set literal syntax; atomizable set predicate, encoding shape, check-time examples. STOP-1 note updated to reflect HashSet atomizable (pending HashMap Stone 3). |

## Deltas from EXPECTATIONS

**Delta 1 — `Fn` type for Probe 9, not `HashSet<Function>`.**
The BRIEF says "non-atomizable-set where T is not atomizable." `HashSet<Function<...>>` is structurally impossible in Rust (functions aren't `Hash`). The probe uses a `Fn([i64])->i64` value directly — a simpler and equally honest non-atomizable type. The predicate fires on ANY non-atomizable T, not only collection types.

**Delta 2 — Probe 7 at WAT surface, not Rust-level `HashSet<HashSet<i64>>`.**
`HashSet<HashSet<T>>` is not a valid Rust type (HashSet doesn't implement Hash). The probe exercises nesting via WAT syntax `(HashSet :Infer inner1 inner2)` where each inner is a `HashSet<i64>`. The runtime's `HashMap<String, Value>` representation handles this correctly via `hashmap_key` for `HashSet` values (canonical "Set:{sorted-element-keys}" scheme). The HolonRepresentable probe (10) tests `HashSet<String>` (valid Rust) rather than `HashSet<HashSet<String>>`.

**Delta 3 — `:wat::holon::Atom` | `:wat::holon::leaf` joint special-case.**
Both `Atom` and `leaf` now share the atomizable-predicate check arm in `infer_list`. `leaf` is the named sibling (arc 065) for the primitive-only case; checking it against the same predicate is strictly correct (leaf only accepts primitives, which are all atomizable — the check adds no false positives for leaf and no behavioral change for its existing users).

**Delta 4 — `hashmap_key` extended for `Value::wat__std__HashSet`.**
Required for nested set support (outer HashSet keyed by inner set canonical key). Canonical key = "Set:{sorted element keys}" — deterministic because HashSet has no order. Added at `src/runtime.rs:9344`.

**Delta 5 — `atom-value` on Bundle now SUCCEEDS (was error before).**
Previously `eval_atom_value` rejected all composite shapes including `Bundle` with TypeMismatch. Now `Bundle` of bare atoms → `HashSet`. This IS a behavior change: code that previously received a TypeMismatch from `atom-value` on a `Bundle(primitive_leaves)` now gets a HashSet. No existing test asserted the old Bundle rejection (only `Bind` rejection was tested). This is the correct design behavior per DESIGN Q3.

**Delta 6 — Atomizable predicate includes `Vector<T>` and `HashMap<K,V>` pre-emptively.**
The `is_atomizable` function includes Vector and HashMap arms (returning false unless T/K/V are atomizable) to correctly handle future stones without false-negative predicates when those types appear as type arguments to HashSet. Example: `HashSet<Vector<i64>>` — the predicate correctly admits it even though Vector atomization is a future stone (the check fires NOW; if runtime hits it, runtime would fail with TypeMismatch). Honest: the predicate is slightly ahead of the runtime.

## Verification summary

```
cargo build --release                                              — OK (0 errors, 5 pre-existing warnings)
cargo test --release --test probe_arc216_stone1_hashset_roundtrip — 10/10 PASS
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio — 18/18 PASS
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio — 15/15 PASS
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias — 6/6 PASS
cargo test --release --test probe_arc215_stone2 — 13/13 PASS
cargo test --release --test probe_arc215_collection_literal_inference — 12/12 PASS
cargo test --release --test probe_brace_map_literal — 9/9 PASS (via probe_brace_map_literal)
cargo test --release --test probe_hashmap_ctor_vector_symmetric — 9/9 PASS
cargo test --release -p wat --lib — 824/824 PASS (0 failures)
cargo clippy --release -- -D warnings — 111 pre-existing errors (all pre-arc-216); 0 new errors from this stone
```

**Pre-existing failure:** `wat_arc170_slice_1f_alpha_helpers` — crossbeam vs wat typed_channel mismatch. Pre-existing before this stone; not introduced by these changes.

## Files changed

- `src/runtime.rs` — `value_to_atom` (HashSet arm), `hashmap_key` (HashSet case), `holon_item_to_value` helper, `eval_atom_value` (Bundle arm)
- `src/comms/mod.rs` — `impl<T> HolonRepresentable for HashSet<T>`
- `src/check.rs` — `fn is_atomizable`, `:wat::holon::Atom`|`:wat::holon::leaf` special-case in `infer_list`
- `tests/probe_arc216_stone1_hashset_roundtrip.rs` — 10 probes (new file)
- `docs/WAT-CHEATSHEET.md` — atomizable-set subsection; STOP-1 update
- `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.1.md` — this file

## Elapsed time

Target: 45-60 min. Actual: ~55 min. Within prediction band.

## What was discovered

1. **`HashSet` internal structure is `HashMap<String, Value>`** — the canonical-key map approach requires extending `hashmap_key` to handle `Value::wat__std__HashSet` for nested set support. The "Set:{sorted}" scheme is deterministic and composable.

2. **Atom's polymorphism (∀T) requires a special-case in `infer_list`** to enforce the atomizable predicate post-inference. The generic scheme path doesn't have hooks for value-domain constraints. The arc-009 "bypass pattern" (infer for side-effects, return concrete type) is the right shape.

3. **`HashSet<HashSet<T>>` is impossible in Rust type system** (HashSet doesn't implement Hash). The WAT runtime works fine (HashMap<String, Value> representation handles it). HolonRepresentable impl requires `T: Hash + Eq`; this naturally excludes `HashSet<HashSet<T>>` at the Rust type level, which is the honest constraint.

4. **`atom-value` on `Bundle` semantic change is clean** — no existing test asserted Bundle rejection. The design's "Bundle-of-bare-Atoms = set-shape" is unambiguous (DESIGN Q9); making it succeed is the correct arc 216 behavior.
