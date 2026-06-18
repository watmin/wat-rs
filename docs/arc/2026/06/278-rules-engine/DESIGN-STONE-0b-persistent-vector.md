# DESIGN — Stone 0b: `:wat::core::PersistentVector` (rpds-backed persistent vector)

> Arc 278 stone 0, part b — the VECTOR mirror of stone 0a (`:wat::core::PersistentMap`, SHIPPED `70835637`).
> The rete engine needs it: a `Token` carries a `matches` provenance VECTOR (grown at each join);
> alpha/beta memories hold lists of elements/tokens — all incrementally extended during fire. std `Vector`
> is `Arc<Vec>` clone-on-write (O(n)/push); rpds `VectorSync` gives O(log n) structural-sharing `push_back`.

## What it delivers

`:wat::core::PersistentVector` backed by **`rpds::VectorSync<Value>`** (Arc/Sync; `Value` is `Send+Sync`).
Structural sharing: `conj` returns a NEW vector sharing most nodes; the original is unchanged. Op surface
mirrors std `Vector`; the generic ops (`conj`/`get`/`first`/`rest`/`count`) dispatch on it (Layer-1). EDN
round-trips distinct from std `Vector`.

## This is a MIRROR of stone 0a — same shape, Vector deltas

**Read `DESIGN-STONE-0a-persistent-map.md` + the SHIPPED `persistentmap_*` code as the worked template.**
0b is the same strike with these substitutions:

| 0a (map) | 0b (vector) |
|---|---|
| `wat__std__HashMap` mirror | `Value::Vec(Arc<Vec<Value>>)` mirror (value.rs:606/728/1040/1146) |
| `rpds::HashTrieMapSync<Value,Value>` | `rpds::VectorSync<Value>` (`new_sync()`, `push_back(v)->Self`, `get(i)->Option<&V>`, `set(i,v)->Option<Self>`, `len()`, `iter()`) |
| variant `wat__core__PersistentMap` | variant `wat__core__PersistentVector` |
| Hash: order-INdependent (sorted pairs) | Hash: **order-DEPENDENT** — sequence hash, mirror `hash_sequence` (value.rs:728); a vector's order is semantic |
| ops `persistentmap_*` in collection/eval.rs | ops `persistentvector_*` mirroring `vector_*`: `length`(:27) `empty?`(:127) `contains?`(:225) `get`(:353, index→Option) `conj`(:542, `push_back`) + `first`/`rest` |
| generic `get`/`assoc`/`contains?`/count arms | generic `conj`(runtime.rs:3834) `get`(:3839) `first`(:4016) `rest`(:4028) `count` arms |
| checker `infer_*` Map arms (PersistentMap<K,V>) | checker arms mirroring the `wat::core::Vector` arms: `infer_conj`(infer.rs:158) `infer_get`(:252, index=i64) `infer_contains`(:60) → `PersistentVector<T>` |
| ctor `(:wat::core::PersistentMap k v …)` | ctor `(:wat::core::PersistentVector e1 e2 …)` — bare elements, infer `T`; mirror `eval_vector_ctor` + the bare-head registration |
| EDN tag `#wat.core/PersistentMap {…}` | EDN tag `#wat.core/PersistentVector [...]` — over an EDN VECTOR body (mirror the 0a tag write/read in edn_shim.rs) |

## The ONE contract decision

EDN = a TAGGED literal `#wat.core/PersistentVector [e1 e2 …]` (writes tagged, reads the tag back to a
PersistentVector — distinct from a bare `[…]` which reads to std `Vector`). Identical mechanism to 0a's
`#wat.core/PersistentMap`; mirror it.

## Method — the compiler is the checklist

Add the variant → `cargo build` → mirror the `wat__core__PersistentMap` arm (and where relevant the std
`Vec` arm) at every exhaustive match it flags (same ~7-file ripple as 0a; 0a's committed arms show exactly
where). Logic in the `collection/`+`value/` homes; only forced thin arms in the megafiles.

## Proof (FM-2-bis + the EDN test FROM THE START — 0a's gap, not repeated)

- `tests/probe_arc278_0b_persistent_vector.rs` (RED at HEAD, un-ignore on green): ctor + length; `get` by
  index; `conj` returns a NEW vector, original unchanged (structural sharing); `first`/`rest`; generic
  `(:wat::core::conj pv x)` / `(:wat::core::get pv 0)`.
- **A lib unit test in `edn_shim::tests`** (in this stone, NOT deferred — 0a's lesson): `#wat.core/PersistentVector`
  round-trips via `value_to_edn_string`→`edn_string_to_value` to an EQUAL `PersistentVector` (not a std Vector).

## Out of scope (affirmative cuts)

- The transient pair (deferred — persistent push is non-wasteful; add if fire profiling demands).
- The `:Seq` protocol (Layer 2 — arc 285).
- `set`/`assoc`-at-index beyond what the engine needs now (add when a stone needs index-update).

## Four questions

- **Obvious?** YES — `PersistentVector` beside `Vector`, mirrors the shipped PersistentMap.
- **Simple?** YES — a mechanical mirror of 0a + std Vector; compiler-driven.
- **Honest?** YES — order-dependent hash (a vector's order is semantic); tagged round-trip distinct from std Vector.
- **Good UX?** YES — generic ops work on it (Layer-1); opt-in; std Vector untouched.

## Blast radius

`Cargo.toml` (rpds already added by 0a — no change) · `src/value/value.rs` · `src/collection/eval.rs` ·
`src/collection/infer.rs` · `src/check.rs` · `src/runtime.rs` · `src/edn_shim.rs` (+ the lib test) ·
forced arms in `observe.rs`/`ast.rs`/`closure_extract.rs`. + the probe. No wat-side changes. No git in the worker.
