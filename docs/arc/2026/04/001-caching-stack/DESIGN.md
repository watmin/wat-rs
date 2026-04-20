# Caching — design notes (2026-04-19)

Durable record of decisions reached this session. Resume state if compacted.

## What 058 specifies

Three pieces in the proposal (FOUNDATION lines 1527-1565):

1. **`:wat::std::LocalCache<K,V>`** — `wat/std/LocalCache.wat`. In-program
   cache, single-threaded, owned as a binding. No pipe, no thread, no queue.
   Fastest memoization.
2. **`:wat::std::program::Cache<K,V>`** — `wat/std/program/Cache.wat`.
   Spawnable program whose driver thread owns an LRU; other programs talk
   to it via queues. The program IS the synchronization point. A program
   wrapping its own LocalCache behind a select loop.
3. **`:wat::std::cached-encode`** — `wat/std/cached-encode.wat`. Thin
   function over `encode` + a cache handle (local or remote).

## Lab prior art (`holon-lab-trading/src/`)

- L1 lives in `encoding/encode.rs` as `EncodeState { l1_cache: LruCache<u64, Vector> }`.
  In `market_observer_program.rs` line 188: `let mut encode_state = EncodeState::new(DEFAULT_L1_CAPACITY);`
  — a plain Rust stack local. Cannot escape scope because Rust owns it.
- L2 lives in `programs/stdlib/cache.rs`. The driver thread owns
  `let mut cache = LruCache::new(NonZeroUsize::new(capacity).unwrap());`
  — another plain stack local. CacheHandle (QueueSender+QueueReceiver) is what
  crosses threads, never the LRU itself.
- **Neither cache is wrapped in Arc, Rc, RefCell, Mutex.** Ownership is the
  discipline. Scope is the enforcement.

## User directive

> "zero mutex - we do not need them. they are easy - not simple.
>  i think C is the answer - the caches /must/ never cross scopes.
>  L1 live in a user's program and L2 live in another program...
>  the L2 just protects its L1 through a select loop."

- Zero Mutex. Firm.
- Caches must never cross scopes.
- L2 is a program that wraps its own L1 behind a select loop.

## wat-rs translation (settled)

**LocalCache Value variant:**

```rust
pub struct LocalCacheCell {
    owner: std::thread::ThreadId,
    cache: std::cell::UnsafeCell<lru::LruCache<Value, Value>>,
}

// Justified by the owner-thread check on every op.
unsafe impl Send for LocalCacheCell {}
unsafe impl Sync for LocalCacheCell {}

// In Value:
Value::wat__std__LocalCache(Arc<LocalCacheCell>)
```

Every `get` / `put` asserts `thread::current().id() == self.owner` before
touching the UnsafeCell. Cross-thread use panics with "LocalCache crossed
scope", not silently. No Mutex, no RwLock, no registry, no Rc.

Value stays `Send + Sync` — channels and spawn keep working.

**Why this is C-in-spirit:** the original option C said "remove Send+Sync
from Value." That would break every crossbeam channel in the runtime.
The real mechanism is: Value stays Send+Sync, LocalCacheCell is
Send+Sync by unsafe+runtime-check, and the runtime panics if a
LocalCache crosses scope. Same semantic ("never crosses scope"), feasible
implementation.

## Construction shape (settled)

Matches `make-bounded-queue` — explicit `:T` arg since there's no content
to infer from:

```
(let* (((cache :wat::std::LocalCache<i64, Holon>)
         (:wat::std::LocalCache::new :(i64,Holon) 16384)))
  (:wat::std::LocalCache::put cache 1 some-holon)
  (:wat::std::LocalCache::get cache 1))
```

- `::new :T capacity` — creates.
- `::put cache key value` — returns `:()`, mutates in place.
- `::get cache key` — returns `:Option<V>`.

## Pre-requisite retrofit — typed constructors for all data structs

User caught a defect during the LocalCache naming discussion: `vec`, `list`,
`HashMap`, `HashSet` rely on content inference / let-annotation fallback,
while `make-bounded-queue` requires explicit `:T`. Inconsistent. Poison.

Fix before LocalCache lands:

- `(:wat::core::vec :T x1 x2 ...)` — explicit T.
- `(:wat::core::list :T x1 x2 ...)` — explicit T.
- `(:wat::std::HashMap :(K,V) k1 v1 k2 v2 ...)` — explicit K,V.
- `(:wat::std::HashSet :T x1 x2 ...)` — explicit T.

**Blast radius (verified 2026-04-19):**
- Zero stdlib wat files use these constructors (grep confirmed).
- Zero macro emissions of these constructors (grep on wat/ returned empty).
- All call sites are Rust: runtime.rs eval_*_ctor (3 fns), check.rs schemes
  (3 fns), and ~70 embedded test strings across runtime.rs, wat_vm_cli.rs,
  mvp_end_to_end.rs.

## Ordering

1. Retrofit typed constructors (vec, list, HashMap, HashSet) — PRE-REQ.
2. LocalCache — Rust primitive + wat stdlib file + tests.
3. Program Cache — redo on LocalCache substrate (select loop + its own L1).
4. cached-encode — pending concrete caller demand.
5. Backfill 058 FOUNDATION.md to match the typed-constructor conventions
   and the LocalCache implementation shape once shipped.

## Uncommitted work to revert

Before starting retrofit: `src/runtime.rs` has ~254 lines of Cache
primitive scaffold (eval_std_program_cache, run_cache_driver,
service_cache_request) built on `LruCache<String, Value>` — wrong
backing, wrong order. Revert. Keep Cargo.toml's `lru = "0.12"` (we'll
need it for LocalCache).

## Sources of truth

- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION.md`
  (lines 1527-1565 on caching, 2127-2128 on stdlib file paths).
- `holon-lab-trading/src/encoding/encode.rs` (L1 shape).
- `holon-lab-trading/src/programs/stdlib/cache.rs` (L2 shape).
- `wat-rs/wat/std/program/Console.wat` (constructor-convention reference).

Until a contradiction surfaces, proposal 058 is correct. When we find
a mismatch in impl, we update the proposal — we don't deviate silently.
