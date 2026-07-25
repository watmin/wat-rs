# BRIEF — Cache Stone 1: the LRU primitive → core (Rust shim + baked surface, the sqlite pattern)

> Part of `DESIGN-cache-tooling-to-core.md`. Stone 1 of 5. **Direct build at final `:wat::cache::` names — NO
> prime** (fresh namespace, grep-verified 0 refs). The crate is the ORACLE; **build fresh, never `cp`.**
> **Name RULED (intueri): `:wat::cache::Lru<K,V>`** (`Lru` — the structure-noun; the namespace supplies "cache";
> rejected `LocalCache`/`Local`/`Bounded`). Ops: `new`/`put`(→`Option<Entry>`)/`get`/`len`/`capacity`; pair alias
> `:wat::cache::Entry<K,V>`. This stone does not touch the service (Stone 2) or the `Lru|Hologram` get/put defclause.

## Objective

Bring the bounded LRU cache primitive into core as a fresh Rust shim + a baked `.wat` surface — the load-bearing
piece (`examples/with-lru` uses it; `HologramCache` composes it). This is the sqlite pattern: a `src/rust_deps/`
primitive + a `:wat::cache::` surface baked into the stdlib. No behavior invented — the crate defines the
semantics; we re-author them clean in the core idiom.

## The ORACLE (study, NEVER copy)

- `crates/wat-lru/src/shim.rs` — a `#[wat_dispatch]` `impl` over the `lru` crate's `LruCache<Value,Value>`:
  `new(capacity: i64) -> Self` (capacity must be > 0), `put(&mut self, k: Value, v: Value) -> Option<(Value,Value)>`
  (the evicted pair, `None` until over capacity), `get(&mut self, k: Value) -> Option<Value>`, `len(&self) -> i64`,
  `is_empty`. Keys must be hashable `Value`s. `register(builder: &mut RustDepsBuilder)` via the macro.
- `crates/wat-lru/src/lib.rs` — `register()` (rust deps) + `wat_sources()` (the `.wat` surface files).
- `crates/wat-lru/wat/lru/LocalCache.wat` — the typed surface (`LocalCache<K,V>` over the rust LRU; `new`/`put`/
  `get`/`len` + whatever the surface adds — read it for the exact wat contract, incl. the `Entry`/evicted-pair shape).

## Rooms (read in order)

1. `src/rust_deps/sqlite.rs` (esp. `pub fn register` ~:350) — the EXEMPLAR: a fresh core Rust primitive as a
   `#[wat_dispatch]` impl + `register(&mut RustDepsBuilder)`. Mirror this shape for the cache.
2. Where core wires its rust-deps `register`s (the `RustDepsBuilder` assembly — grep `RustDepsBuilder` / the core
   `register` call chain that `sqlite::register` joins). The new `cache::register` joins the same list.
3. `src/io.rs` ~:1660 (`the baked stdlib load order as a vector of [path, source] pairs`) — add the `:wat::cache::`
   surface source here (the bake manifest), in the right load order (primitive before any consumer).
4. Core `Cargo.toml` — add the `lru` crate dep (currently a `wat-lru`-only dep). Confirm version from
   `crates/wat-lru/Cargo.toml`.
5. `stdlib.rs` ~:339/:383 — the precedent notes that "a baked core source may declare under `:wat::`" (the
   `:wat::query::` / reclaimed-sqlite path). `:wat::cache::` is net-new + unprimed — same clean case.

## Disconfirming probe (WRITE + RUN before briefing the rider — proves the composition on exactly the gap)

A minimal round-trip against the fresh core primitive at cap 2, proving eviction + generic-over-EDN keys:
```
new(2)
put(:a 1)            → None            ; under cap
put(:b 2)            → None            ; at cap
put(:c 3)            → Some (:a, 1)    ; LRU evicts the oldest
get(:b)              → Some 2          ; still present
get(:a)              → None            ; evicted
len                  → 2
```
It should fail on exactly the missing piece (the primitive/surface not yet wired), everything around it clean.
If the probe can't isolate the gap (e.g. `RustDepsBuilder` wiring differs from sqlite's), STOP — the foundation
isn't ready; surface it.

## Blast radius (bounded)

- **NEW** `src/rust_deps/cache.rs` (the `#[wat_dispatch]` LRU shim) + its `register` wired into the core rust-deps list.
- **NEW** `wat/cache/<Primitive>.wat` (the baked surface) + its line in the `src/io.rs` bake manifest.
- **`Cargo.toml`** — add the `lru` dep.
- **NO deletion** this stone — the `wat-lru` crate stays intact until the migration/annihilate stone (Stone 5).
- **DO NOT** touch the `check.rs` `make-channel`/`send`/`recv` pair-tracking or deadlock walker — that is the raw-channel
  retirement (a separate crusade slice); the cache primitive uses none of it.

## STOP triggers

1. If core rust-deps registration does NOT go through the same `RustDepsBuilder` path `sqlite::register` uses — STOP, surface the real wiring (do not invent one).
2. If the `lru` crate cannot be a core dep (feature/edition conflict) — STOP.
3. If generic `<K,V>` over `Value` needs anything the sqlite `Connection` opaque didn't (e.g. a Hash bound the wat side must declare) — STOP, surface it (don't silently narrow to a concrete K,V).

## Gate

A `deftest` round-trip in core (the probe, promoted to a real test): `new`/`put`→evict/`get`/`len`/`capacity`,
generic keys. Weigh by my OWN `cargo nextest run --release` re-run (Summary line, never a piped exit); floor
must stay at the known green. Commit on green (green = DR it). => the primitive is home; Stone 2 (the `defservice`
over it) follows.
