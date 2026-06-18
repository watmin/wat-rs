# DESIGN — Stone 0a: `:wat::core::PersistentMap` (rpds-backed persistent hash map)

> Arc 278 stone 0 (the RUST prerequisite), part a (the map; the vector is 0b, a mirror). The keystone: the
> rete working-memory's four memories are all maps, updated incrementally during fire — std `HashMap` is
> `Arc<std>` clone-on-write (O(n)/update); this gives O(log n) structural-sharing updates. Additive,
> opt-in, beside the std `HashMap` (which stays). Language-wide win, not rete-only.

## Why now / what it delivers

A new value type `:wat::core::PersistentMap` backed by **`rpds::HashTrieMapSync<Value, Value>`** (Arc/Sync
variant — `Value` is `Send+Sync`). Structural sharing: `assoc`/`dissoc` return a NEW map sharing most nodes
with the old; the old is unchanged and the update is O(log n), not a full clone. Full op surface mirrors std
`HashMap`; the generic ops (`get`/`assoc`/`contains?`/`count`) dispatch on it (Layer-1 polymorphism — "a map
is a map"). EDN round-trips distinct from std `HashMap`.

## The ONE contract decision (pinned)

**EDN representation = a TAGGED literal** `#wat.core/PersistentMap {k1 v1 k2 v2 …}`. Writes as a tagged map;
reads the tag back to a `PersistentMap`. This keeps round-trip IDENTITY (a std-HashMap `{}` literal reads to
`wat__std__HashMap`; the tagged form reads to `PersistentMap`) — required for the state-blob (stone 4).
Mirror the existing tagged-value mechanism (records/holon already round-trip via `#`-tags through
`edn_shim`). If the tag reader/writer plumbing is more than mirroring an existing tagged type → STOP-2.

## The change (mirror `wat__std__HashMap` everywhere)

1. **Cargo.toml** (the `wat` crate): add `rpds = "1"` (uses `archery` Arc variants). STOP-1 if rpds's MSRV
   exceeds the toolchain → pin the newest rpds that builds + note it.
2. **`src/value/value.rs`** — new variant `Value::wat__core__PersistentMap(rpds::HashTrieMapSync<Value, Value>)`
   (no `Arc` wrapper — the rpds type is already cheap-clone/shared). Mirror `wat__std__HashMap` (:97) at its
   exhaustive-match impls: `PartialEq` (:610), `Hash` (:755 — order-independent, same strategy),
   `type_name` → `"wat::core::PersistentMap"` (:1030), `type_path` → `":wat::core::PersistentMap"` (:1129).
3. **The compiler IS the checklist (gate-the-whole-family).** A new `Value` variant makes every EXHAUSTIVE
   match non-exhaustive. `cargo build` → for each error, add a `PersistentMap` arm mirroring the
   `wat__std__HashMap` arm at that site. Known ripple (7 files): `value.rs`, `edn_shim.rs`, `collection/eval.rs`,
   `runtime.rs`, `value/observe.rs`, `ast.rs`, `closure_extract.rs`. Watch the fail-count waterfall to zero.
4. **Constructor** `:wat::core::PersistentMap` — `eval_persistentmap_ctor` mirroring `eval_hashmap_ctor`
   (`collection/eval.rs:984`) + dispatch arm in `runtime.rs` (mirror `:wat::core::HashMap` at :4330).
5. **Ops** (mirror the `hashmap_*` family in `collection/eval.rs`): `get` (:411), `assoc` (:669 — but rpds
   `.insert(k,v)` returns the new map, NO clone — this is the whole point), `dissoc` (:698), `contains-key?`
   (:258), `length` (:51), `keys`, `values`. Each: the `_inner` (add a `PersistentMap` arm) + the `eval_` +
   the `:wat::core::PersistentMap/<op>` dispatch in `runtime.rs` (mirror :4282–4330).
6. **Layer-1 generic polymorphism** — add `PersistentMap` arms so the GENERIC ops dispatch on it:
   `eval_get` (`runtime.rs:3839`), `eval_assoc` (:3843), `eval_contains` (:3829), and count/length. After this,
   `(:wat::core::get pm k)` / `(:wat::core::assoc pm k v)` / `(:wat::core::contains? pm k)` work on a
   PersistentMap exactly as on a std HashMap.
7. **Checker** — `src/collection/infer.rs`: add `TypeExpr::Parametric { head: "wat::core::PersistentMap" }`
   arms to `infer_get` (:214), `infer_assoc` (~308), `infer_contains` (:31) mirroring the `wat::core::HashMap`
   arms (key/value unification identical). The type is `:wat::core::PersistentMap<K,V>` parametric. + the ctor
   infer returns `PersistentMap<K,V>`.

## Proof (FM-2-bis probe — RED at HEAD)

`tests/probe_arc278_0a_persistent_map.rs` (committed RED; un-ignore on green). Exercises, via wat source:
- ctor `(:wat::core::PersistentMap :a 1 :b 2)` → `count` == 2;
- `assoc` returns a NEW map with the key, the ORIGINAL unchanged (structural sharing / immutability);
- `get` + `contains-key?` hit/miss; `dissoc` removes;
- the GENERIC ops work: `(:wat::core::get pm :a)` == 1, `(:wat::core::assoc pm :c 3)`, `(:wat::core::contains? pm :a)`;
- EDN round-trip: `value_to_edn_string` of a PersistentMap → parses back to a PersistentMap (tagged), equal.
RED at HEAD because `:wat::core::PersistentMap` is an unknown head (eval/check error).

## Out of scope (affirmative cuts — NOT deferrals dangling)

- **`PersistentVector`** — stone 0b (a mirror of this, for rpds `VectorSync`).
- **The transient↔persistent pair** — NOT in 0a; persistent O(log n) updates are non-wasteful on their own;
  the transient fast-path is added only if fire-loop profiling demands it (its own micro-stone). The
  CLARA-REFERENCE §5 transient is a JVM constant-factor optimization, not a correctness need.
- **The `:Map` protocol** (Layer 2) — arc 285; this stone delivers Layer 1 (shared op names) only.
- **atomizable/wire-crossing of PersistentMap** — add when a peer actually sends one (don't build the
  forcing function); std HashMap's atomizable arm (check.rs arc-216) is the model when needed.

## Four questions

- **Obvious?** YES — `PersistentMap` beside `HashMap`; the name says the semantic (structural-sharing immutable).
- **Simple?** YES — a mechanical mirror of an existing complete type; the compiler drives the ripple.
- **Honest?** YES — round-trips distinctly (tagged); the type names its perf character; no lossy aliasing to HashMap.
- **Good UX?** YES — generic ops just work on it (Layer 1); opt in by constructing it, std HashMap untouched.

## Blast radius

`Cargo.toml` (+1 dep) · `src/value/value.rs` (variant + 4 impls) · `src/collection/eval.rs` (ctor + ~7 ops) ·
`src/runtime.rs` (dispatch + generic-op arms) · `src/collection/infer.rs` (3 infer arms + ctor) ·
`src/edn_shim.rs` (tagged write + read) · mechanical arms in `observe.rs`/`ast.rs`/`closure_extract.rs` as the
compiler flags. + the probe. NO wat-side changes. NO behavior change to std HashMap. No git in the worker.
