# DESIGN — the cache tooling comes home to core (correct modern-wat, not a carbon copy)

> **Status: DRAWN (2026-07-25), builder-ruled.** *"all the cache tooling moves — we need it — we built it as
> a second crate to prove we could have distributions of wat … wat needs it, the cache tooling becomes core
> because wat needs it — not having these in the core distribution of wat is unacceptable … it is not a carbon
> copy — it is a correct impl of what they need to be in modern wat."* The `wat-lru` / `wat-holon-lru` crates
> were the **distributions-of-wat proof**; that experiment succeeded and is over. The cache tooling now joins
> `sqlite` and `telemetry` in core, rebuilt in the modern idiom — the **crate is the ORACLE, never `cp`** (R48
> `ABOLENDO RENASCIMVR` / the sqlite+telemetry precedent: *"crates are HINTS, not trusted — build fresh"*).

## Why (grounded)

- **wat needs it in the core distribution.** A cache is table-stakes substrate. Same call as sqlite (S-series)
  and telemetry (T-series): *wat needs it → it is core.*
- **It is not a carbon copy.** wat is far more mature than the ~2-month-old crate: `defservice`, `:satisfies`
  surfaces, the `Peer'` model, `connect'`, the outcome walls, records-are-EDN. The two SERVICES were
  hand-rolled actors (`make-channel` + `spawn-thread` + `select` + arc-130 pair-by-index) — that whole scaffold
  **dissolves** into a `defservice` (`COMPONENDO DELEO`, R33). The pure primitives move largely as-is, clean.
- **It is a real slice of the IPC de-prime.** The `CacheService` actor is 2 of the ~14 remaining `make-channel`
  callers; doing this *right* (→ `defservice`) retires their raw-channel/`spawn-thread` use as a byproduct.

## Grounded facts (studied 2026-07-25 — the lair)

- **`LocalCache` is Rust-backed** — `crates/wat-lru/src/shim.rs` (`#[wat_dispatch]`-generated register of the LRU
  data structure, a Rust opaque) + `crates/wat-lru/wat/lru/LocalCache.wat` (the typed surface). Core-move = the
  **sqlite pattern**: a `src/` primitive + a baked `.wat` surface. NOT a pure-wat lift.
- **`HologramCache` is a genuine composite**, not a trivial `LocalCache<HolonAST>` — a `defstruct` over
  `:wat::holon::Hologram` (VSA slot-routed similarity store, **already core**: `wat/holon.wat` + `src/hologram.rs`)
  **+** `LocalCache<HolonAST,nil>` (LRU freshness): `put` inserts into the Hologram AND tracks/evicts via the LRU
  (drops the evicted key from BOTH); `get` does a hologram similarity `find`, LRU-bumps on hit.
- **`Hologram` is already core** → no hidden crate dependency. Only the cache tooling itself is stranded.
- **Sharding = ONE cache, N client channels** (arc-130 pair-by-index): `spawn capacity count` builds `count`
  request+reply channel pairs, ONE driver thread, one `LocalCache`, a `select` fan-in routing replies by index.
  This is exactly "N clients → one server" hand-rolled — **`defservice` + `connect'` does it natively** (each
  client dials its own peer; replies route by the peer). The `count`/`select`/DriverPair/pair-by-index machinery
  **dissolves**. `capacity` (the single cache's size) stays.
- **Consumers** (whole-tree scout): `LocalCache` is **load-bearing** (used by `examples/with-lru` and composed by
  `wat-holon-lru`'s `HologramCache`); the two **`*Service` actors are exercised ONLY by their own tests** (no
  external caller) → the defservice rebuild re-points tests, not live callers. `wat-holon-lru` is standalone
  (CLI-registered + own tests only).
- **No self-scheduling / telemetry dependency.** The metrics cadence (`tick-window`/`MetricsCadence`/`Reporter`/
  `Report`/`Stats`) is per-request-gated, not a wall-clock timer — and **deferred from v1 entirely** (builder:
  *"the LRUs don't need metrics to function; thread telemetry into them once it's ready"*). So the port does NOT
  wait on the un-green self-scheduling stone (item-c) NOR on telemetry proper.

## The four pieces, and their correct modern form

| piece | today (crate) | correct in core |
|---|---|---|
| `LocalCache<K,V>` | Rust shim + `.wat` surface | `src/` LRU primitive + baked surface (sqlite pattern); rebuilt clean |
| `CacheService` | hand-rolled actor (make-channel×N + spawn-thread + select + pair-by-index) | **`defservice`** — `get`/`put`; state = a `LocalCache` in `:ephemeral`, born in `:init` from `capacity`; N clients via `connect'`; scaffold dissolves |
| `HologramCache` | `defstruct` (Hologram + LocalCache) | composite defstruct + fns in core over core `Hologram` + core `LocalCache` |
| `HologramCacheService` | hand-rolled actor | **`defservice`** over `HologramCache` |

## The surfaces (v1 — metrics DEFERRED)

`CacheService` (generic `<K,V>`):
- `Get [probes <- Vector<K>] -> GetResult [results <- Vector<Option<V>>]`
- `Put [entries <- Vector<Entry<K,V>>] -> PutAck []`  (`Entry<K,V>` = `(K,V)`)
- client convenience: `get [svc probes] -> Vector<Option<V>>`, `put [svc entries] -> nil`

`HologramCacheService` (concrete `HolonAST`):
- `Get [probes <- Vector<HolonAST>] -> GetResult [results <- Vector<Option<HolonAST>>]`
- `Put [entries <- Vector<Entry>] -> PutAck`

**Deferred to a telemetry follow-on** (NOT in v1): `Report`/`Metrics`/`Stats`/`MetricsCadence`/`Reporter`/
`tick-window`/`null-reporter`/`null-metrics-cadence`. When telemetry proper is ready, the service is *given* a
telemetry sink capability and *emits* (R51 `TYPO TANGO` — thread telemetry into anything that wishes to log;
the sink is a granted capability, not baked-in).

## Build order (mirrors sqlite S0→S2 / telemetry T0→T1; each stone: DESIGN/RED-probe/BRIEF → rider → weigh by own `--release` re-run → `deftest` gate)

1. **`LocalCache` → core** — the Rust LRU primitive (fresh `src/` shim, crate = oracle) + the baked surface. deftest round-trip gate (new/put→evict/get/len). *(heaviest — fresh Rust.)*
2. **`CacheService` defservice → core** — `:satisfies` a `CacheService` surface; state = `LocalCache` in `:ephemeral` from `capacity`; `:impls` = get/put; N clients via `connect'`. deftest multi-client gate (proves the arc-130 N-client case natively, no pair-by-index).
3. **`HologramCache` → core** — the composite (core `Hologram` + core `LocalCache`); put/get/len/capacity with the dual-eviction (drop evicted key from both). deftest gate.
4. **`HologramCacheService` defservice → core** — over `HologramCache`. deftest gate.
5. **Migrate + annihilate + reclaim** — re-point the crates' tests to the core forms; delete `crates/wat-lru` + `crates/wat-holon-lru` (drop the CLI registrations `wat_lru::register`/`wat_holon_lru::register` in `wat-cli` + `cargo-wat`; the `examples/with-lru` LocalCache use re-points to core); reclaim the FQDNs.

## Naming — RULED: the `:wat::cache::` namespace, NO PRIME (direct build at final names)

**Builder-ruled 2026-07-25:** the core home is **`:wat::cache::`**. arc-109 (kill-std) rules out `:wat::std::cache::`;
`:wat::cache::` is a **fresh, unoccupied namespace** (grep-verified: 0 refs in `.wat`/`.rs`). Because the crates
occupy *different* names (`:wat::lru::` / `:wat::holon::lru::`), there is **no in-place collision → no `'` prime,
no reclaim step**: we build directly at the final `:wat::cache::` names, migrate the crate tests to them, delete
the crates. (The prime dance exists only to replace a name whose old impl still occupies it — not the case here.)

**Leaf vocabulary — intueri-cast + weighed (2026-07-25):**

| piece | name | note |
|---|---|---|
| LRU primitive | `:wat::cache::Lru<K,V>` | `Lru` speaks (structure-noun; backing type is `LruCache`; namespace supplies "cache"). Rejected: `LocalCache`/`Cache*` (stutter), `Local` (dead "networked" contrast), `Bounded` (mumbles the discipline). |
| KV pair alias | `:wat::cache::Entry<K,V>` | kept — standard cache/map word, no stutter. |
| LRU service | `defservice :wat::cache::lru-svc` | kebab `-svc`, matching the stdio precedent (`stdout-svc`); arc-109 K.lru **retired the `CacheService` grouping noun** — no PascalCase service type. Rejected: `CacheService` (stutter), `Service` (mumbles), `Server` (**lies** — implies a network listener; a defservice is not one). |
| holographic-LRU composite | **`:wat::cache::HolographicLru`** (concrete over `HolonAST`) | **RULED (builder 2026-07-25).** It IS an LRU cache — same eviction discipline as `Lru`; the ONLY difference is key-matching (base = exact-key; this = hologram *similarity*). So it carries the shared `Lru` label (bare `Holographic` mumbled it) + the `Holographic` variant qualifier. Placement `:wat::cache::` (role = a cache; the `query/sqlite-store` composite-in-its-role-namespace precedent). `Holographic` (adjective) not bare `Hologram` → sidesteps the **three-way `Hologram` tangle** (the `:wat::holon::Hologram` store it composes, the future renamed `Hologram` value per 294.e, and this). Concrete over `HolonAST` (the `Hologram` store is HolonAST-keyed, not generic). |
| hologram service | `defservice :wat::cache::hologram-svc` | mirrors `lru-svc`. |
| protocol + client fns | `Request::Get`/`Put` · `Reply::GetResult`/`PutAck` · verbs `:wat::cache::get`/`put` | all keep their promise. |

**Architectural note surfaced by the naming (fold into Stones 2/4):** both services expose client `get`/`put` in
one namespace → they'd collide (wat forbids two `defn`s). Resolve with the **sqlite `select` precedent** — ONE
`get`/`put` defclause dispatching over the cache-flavor types (`Lru | HolographicLru`). This is also *why* the
holographic-LRU composite can stay in `:wat::cache::`.

## Relationship to arc 294 (DECOUPLED)

The cache tooling keys on `HolonAST` and composes `:wat::holon::Hologram`. Arc **294.e** (`HolonAST → Hologram`
rename + `src/holon/`) is the pending, PHASE-1-gated keystone. This campaign does **not** wait on it: the cache
tooling uses the **current live `HolonAST`** type, and `Holographic` was chosen precisely so nothing here collides
with the future `Hologram` value name. When 294.e fires, its corpus-wide keyword codemod sweeps the cache's
`HolonAST` refs → `Hologram` along with everyone else — for free, no special handling; `Holographic` stays put.
The two arcs are independent.

## Open items (rule/cast before / during the build)
2. **`LocalCache` generic vs concrete Rust backing** — confirm the `src/` primitive stays generic `<K,V>` over
   EDN values (it must, for `HologramCache`'s `<HolonAST,nil>` use + the `<String,i64>` example).
3. **CLI registration** — `wat-cli`/`cargo-wat` currently `register` the crates; core-baked stdlib needs no
   per-crate registration (it's in `wat_sources()`/the freeze). Confirm the deletion path.

## Do-nots (carried from the sqlite/telemetry precedent + this session's grounding)

- **Build FRESH; the crate is the ORACLE, never `cp`** (R48; the "not a carbon copy" ruling).
- **The services are `defservice`s** — do NOT re-create the hand-rolled `make-channel`/`spawn-thread`/`select`/
  pair-by-index scaffold; N clients `connect'`; the actor's serialization is the mutex (ZERO-MUTEX, R28/R30).
- **Metrics DEFERRED** — v1 is get/put only; telemetry threaded in later as a granted sink.
- **Weigh by your OWN `--release` re-run** (Summary line, never a piped exit); cast wards, never narrate;
  four-questions inform every decision; commit + push per green stone (green = DR it).
