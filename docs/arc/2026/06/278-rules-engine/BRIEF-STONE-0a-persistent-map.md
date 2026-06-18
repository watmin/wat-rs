# BRIEF — Stone 0a: `:wat::core::PersistentMap` (rpds-backed)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** Build, run the named
tests, report verbatim. Another agent weighs the kill independently against its own build.

## The work (one paragraph)

Add a new value type `:wat::core::PersistentMap` backed by **`rpds::HashTrieMapSync<Value, Value>`** — a
persistent hash map with structural sharing (O(log n) immutable `assoc`/`dissoc`; the original is never
mutated). It lives BESIDE the existing std `:wat::core::HashMap` (which is untouched). Mirror the
`wat__std__HashMap` implementation at every site: the `Value` variant + its impls, the constructor, the op
family, the generic-op dispatch (Layer-1 polymorphism), the checker inference, and the EDN round-trip. This
is a mechanical mirror of an existing complete type — the std `HashMap` code IS your worked reference.

## Read first (in order)

1. **`docs/arc/2026/06/278-rules-engine/DESIGN-STONE-0a-persistent-map.md`** — the full contract + the ONE
   pinned decision (EDN = a tagged literal `#wat.core/PersistentMap {…}`) + the exact site list. Implement it.
2. `src/value/value.rs` — `wat__std__HashMap` variant (:97) + impls `PartialEq`(:610) `Hash`(:755)
   `type_name`(:1030) `type_path`(:1129). Mirror each.
3. `src/collection/eval.rs` — the `hashmap_*` op family: `eval_hashmap_ctor`(:984), `hashmap_get_inner`(:411),
   `hashmap_assoc_inner`(:669), `hashmap_dissoc_inner`(:698), `hashmap_contains_key_q_inner`(:258),
   `hashmap_length_inner`(:51), keys/values. Mirror as `persistentmap_*`.
4. `src/runtime.rs` — op dispatch `:wat::core::HashMap/<op>`(:4282–4330) + ctor(:4330) → add
   `:wat::core::PersistentMap/<op>` + ctor. Generic ops `eval_contains`(:3829) `eval_conj`(:3834)
   `eval_get`(:3839) `eval_assoc`(:3843) → add PersistentMap arms (Layer-1).
5. `src/collection/infer.rs` — `infer_contains`(:31) `infer_get`(:214) `infer_assoc`(~308); each keys on
   `TypeExpr::Parametric { head: "wat::core::HashMap" }` — add a `"wat::core::PersistentMap"` arm (identical K/V
   unification). + ctor infer → `PersistentMap<K,V>`.
6. `src/edn_shim.rs` — `value_to_edn_notag`(:1637)/`value_to_edn`(:2387) HashMap→`OwnedValue::Map`(:1706,:1827)
   + the EDN map reader(:1048). PersistentMap writes/reads as a TAGGED form (see DESIGN STOP-2).

## Implementation sketch

- `Cargo.toml` (the `wat` crate): `rpds = "1"`. `use rpds::HashTrieMapSync;`
- rpds API you'll use (all persistent — return a NEW map, no clone): `HashTrieMapSync::new()`,
  `.insert(k, v) -> Self`, `.remove(&k) -> Self`, `.get(&k) -> Option<&V>`, `.contains_key(&k) -> bool`,
  `.size() -> usize`, `.keys()`, `.values()`, `.iter()`.
- Variant: `Value::wat__core__PersistentMap(rpds::HashTrieMapSync<Value, Value>)` — NO `Arc` wrapper (the
  rpds type is already cheap-clone/shared). `Value` derives Clone; rpds clone is cheap. `Hash` for the
  variant: iterate + XOR/sum per-entry hashes (order-independent — copy the `wat__std__HashMap` Hash strategy
  at value.rs:755). `PartialEq`: rpds `HashTrieMap` impls `PartialEq` — delegate.
- **The compiler is your checklist:** after adding the variant, `cargo build` → every exhaustive `match` on
  `Value` errors; add a `PersistentMap` arm mirroring the `wat__std__HashMap` arm at that site. Watch the
  count waterfall to zero. (~7 files touch it.)
- `assoc`: `Ok(Value::wat__core__PersistentMap(m.insert(k, v)))` — that's the whole non-redundancy win; NO
  `.clone()` of the map contents.

## STOP triggers (halt + surface; do not improvise)

1. **rpds MSRV** — if `rpds = "1"` won't build on this toolchain, pin the newest rpds that DOES build, note
   the version + why, and continue. (Do not hand-roll a HAMT.)
2. **EDN tagged literal** — if making `PersistentMap` round-trip as a tagged form is MORE than mirroring an
   existing tagged type's read/write path in `edn_shim.rs`, STOP and report what the tag plumbing needs.
3. **std `HashMap` must stay byte-identical** — if any change alters std-HashMap behavior or its tests, STOP.
4. **Floors** — if any floor count moves beyond the new probe flipping RED→GREEN (+1), STOP and report.

## Verify (paste output verbatim)

```
cargo test --release -p wat --test probe_arc278_0a_persistent_map -- --include-ignored   # 1/1 GREEN (un-ignore it)
cargo test --release -p wat --lib 2>&1 | grep "test result"                              # 929 passed / 36 failed (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                               # 264 / 1 (UNCHANGED)
cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"      # ~893 / 4 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result                         # 1 / 0 (deporder green)
cargo build --release 2>&1 | tail -2                                                      # clean
```
Report: the diff summary (files + the variant + each mirrored arm), the command outputs verbatim, any STOP
hit, any delta from expectation. Do not claim a green you did not see. Un-ignore the probe as part of the work.

## Blast radius

`Cargo.toml` (+rpds) · `src/value/value.rs` · `src/collection/eval.rs` · `src/runtime.rs` ·
`src/collection/infer.rs` · `src/edn_shim.rs` · mechanical arms in `value/observe.rs`/`ast.rs`/`closure_extract.rs`
as the compiler flags. + un-ignore the probe. NO wat-side files. NO new behavior for std HashMap. NO git.
