# BRIEF — Cache Stone 1: the LRU primitive → core (Rust shim + baked surface, the sqlite pattern)

> Part of `DESIGN-cache-tooling-to-core.md`. Stone 1 of 5. **Direct build at final `:wat::cache::` names — NO
> prime** (fresh namespace, grep-verified 0 refs). The crate is the ORACLE; **build fresh, never `cp`.**
> **Name RULED (intueri): `:wat::cache::Lru<K,V>`.** This stone does not touch the service (Stone 2) or the
> `Lru|HolographicLru` get/put defclause.
>
> **Rooms re-grounded 2026-07-25 by the orchestrator** (the prior revision named `src/io.rs:~1660` for the bake
> manifest — that was WRONG; it is `src/stdlib.rs`). Every file:line below was read this session.

## Objective

Bring the bounded LRU cache primitive into core as a fresh Rust shim + a baked `.wat` surface — the load-bearing
piece (`HologramCache` composes it). This is the sqlite pattern: a `src/rust_deps/` primitive + a `:wat::cache::`
surface baked into `STDLIB_FILES`. The crate defines the SEMANTICS; we re-author them clean in the core idiom.

## The ORACLE — study, NEVER copy (`crates/wat-lru/`, all verified)

- **`crates/wat-lru/src/shim.rs:44`** — `pub struct WatLruCache { inner: LruCache<Value, Value> }`. Storage is
  `Value,Value`; the wat-level `K,V` are phantom.
- **`:48-52`** — the attribute, note the THIRD arg sqlite does not have:
  ```rust
  #[wat_dispatch(path = ":rust::lru::LruCache", scope = "thread_owned", type_params = "K,V")]
  ```
- **The methods:** `new(capacity: i64) -> Self` (`:58`) · `put(&mut self, k: Value, v: Value) -> Option<(Value, Value)>`
  (`:90`, returns the EVICTED pair, `None` until over capacity) · `get(&mut self, k: Value) -> Option<Value>` (`:107`) ·
  `len(&self) -> i64` (`:122`) · `is_empty(&self) -> bool` (`:127`).
- **`:135`** — `pub fn register(builder: &mut RustDepsBuilder)` → `__wat_dispatch_WatLruCache::register(builder)`.
- **`crates/wat-lru/Cargo.toml:11`** — `lru = "0.12"`.
- **`crates/wat-lru/wat/lru/LocalCache.wat`** — the wat surface: a `typealias :wat::lru::LocalCache<K,V>` onto
  `:rust::lru::LruCache<K,V>` plus four thin `defn` wrappers. **Note `put` returns a bare tuple
  `:wat::core::Option<(K,V)>`** — see § Entry below; we improve on this.
- **Two guard behaviours to PRESERVE:** `new` panics when `capacity <= 0`; `put` panics when the key is not
  hashable (`value_is_hashable`, guarding opaque handles before `push`). Both are documented at the oracle.

## Rooms (read in order — all verified this session)

1. **`src/rust_deps/sqlite.rs:249`** — `#[wat_dispatch(path = ":rust::sqlite::Connection", scope = "thread_owned")]`
   on an `impl` block. **`:350`** — `pub fn register(builder: &mut RustDepsBuilder)` calling
   `__wat_dispatch_WatSqliteConnection::register(builder)`. This is the EXEMPLAR shape to mirror.
2. **`src/rust_deps/mod.rs:60`** — `mod sqlite;` (private module declaration). **`:178`** —
   `sqlite::register(&mut builder);` inside `with_wat_rs_defaults`. `cache` joins in BOTH places.
3. **`src/stdlib.rs:34`** — `const STDLIB_FILES: &[WatSource]`, the bake manifest. Entry shape (see `:48-51`, the
   sqlite exemplar):
   ```rust
   WatSource { path: "wat/sqlite.wat", source: include_str!("../wat/sqlite.wat") },
   ```
   Add `wat/cache.wat` the same way, with a comment naming the arc + why its load position is legal.
   **`:30-33`** — load order is foundational→derived and is **enforced by `:wat::deporder::verify-stdlib`; a
   violation is a RED BUILD.** The cache surface's only eval-deps are core.wat builtins
   (`typealias`/`defn`/`defrecord`/`Option`), so it may load immediately after `wat/core.wat`, alongside
   `wat/sqlite.wat`.
4. **Core `Cargo.toml`** — add `lru = "0.12"` (currently a `wat-lru`-only dep; confirm the version resolves in the
   workspace).

## The `Entry` improvement (RULED — this is NOT a copy of the oracle)

The oracle's `put` returns a raw positional tuple `Option<(K,V)>`. Modern wat prefers a NAMED record over a
positional pair. Introduce **`:wat::cache::Entry<K,V>`** — a `defrecord` with named `key`/`value` fields — so the
wat surface's `put` returns `:wat::core::Option<wat::cache::Entry<K,V>>`.

The Rust shim MAY keep returning `Option<(Value, Value)>` (proven, mirrors the oracle) with the **wat surface**
doing the wrap. **STOP-4** covers the case where that mapping is not clean.

## Disconfirming probe (WRITE + RUN before the implementation — must fail on exactly the gap)

A cap-2 round-trip proving eviction + generic-over-EDN keys:

```
new(2)
put(:a 1)   → None                       ; under cap
put(:b 2)   → None                       ; at cap
put(:c 3)   → Some Entry{key :a value 1} ; LRU evicts the oldest
get(:b)     → Some 2                     ; still present
get(:a)     → None                       ; evicted
len         → 2
```

Run it FIRST; it should fail on exactly the missing piece (primitive/surface not yet wired), everything around it
clean. If it cannot isolate the gap, STOP — the foundation is not ready; surface it.

## Blast radius (bounded — STOP + report if exceeded)

- **NEW** `src/rust_deps/cache.rs` (the `#[wat_dispatch]` LRU shim) + `mod cache;` and `cache::register(&mut builder);`
  in `src/rust_deps/mod.rs`.
- **NEW** `wat/cache.wat` (the baked surface) + its `WatSource` entry in `src/stdlib.rs`.
- **`Cargo.toml`** — the `lru` dep.
- **NO deletion.** The `wat-lru` crate stays fully intact — it is the oracle, and it is retired in Stone 5, not here.
- **DO NOT** touch `check.rs` channel/pair-tracking or the deadlock walker; the cache primitive uses none of it.

## STOP triggers (halt and report — do NOT improvise)

1. If core rust-deps registration does NOT go through the `RustDepsBuilder` path `sqlite::register` uses — STOP,
   surface the real wiring; do not invent one.
2. If `lru = "0.12"` cannot be a core dep (feature/edition/version conflict) — STOP.
3. If the generic `<K,V>` needs anything the sqlite `Connection` opaque did not (the `type_params` attr, a Hash
   bound the wat side must declare) — STOP and surface; do NOT silently narrow to a concrete `K,V`.
4. If mapping the shim's `Option<(Value,Value)>` into `Option<Entry<K,V>>` at the wat surface is not clean — STOP
   and surface it. Do not force it, and do not silently fall back to the bare tuple.
5. If `:wat::deporder::verify-stdlib` goes red on the new file's load position — STOP and report the ordering
   constraint it names.

## A question to SURFACE in your report (do not decide it yourself)

The oracle **panics** on `capacity <= 0` and on a non-hashable key, with a comment noting the dispatch macro cannot
yet marshal method-internal errors back to wat as a `RuntimeError`. Preserve that behaviour for this stone
(behaviour-parity with the oracle). But **report** whether those panics look wrong to you under the
no-hidden-failures LAW (a failure should be a matchable VALUE, not a raise) — the orchestrator will rule on whether
a later stone converts them. Do not convert them now.

## Gate

The probe promoted to a real `deftest` in core: `new` / `put`→evict / `get` / `len`, generic keys, cap-2 eviction.
Then **`cargo build --release` clean** and **`cargo nextest run --release`**. Run everything FOREGROUND; do not
background a command and return. Report the nextest **Summary line verbatim** — the orchestrator weighs the floor
by their OWN re-run (current known floor: **4163 passed, 314 skipped**). Do NOT commit; the orchestrator commits
on their own weigh.
