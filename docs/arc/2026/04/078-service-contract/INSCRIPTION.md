# wat-rs arc 078 — Service contract: Reporter + MetricsCadence — INSCRIPTION

**Status:** SHIPPED 2026-04-29 across slices 0–5.

The lab built `:trading::cache::Service` with a Reporter +
MetricsCadence telemetry contract, then recognized it as substrate
machinery. This arc lifted the lab's machinery into wat-rs as
`:wat::holon::lru::HologramCacheService`, retrofitted
`:wat::lru::CacheService<K,V>` to the same shape, and codified the
canonical service-contract idiom in CONVENTIONS.md.

---

## What shipped

| Slice | Surface | Commit |
|-------|---------|--------|
| 0 | Crate dir rename `wat-hologram-lru/` → `wat-holon-lru/` to match the namespace path. | wat-rs `bb95f27`, lab `eaaffbd` |
| 1 | API rename `:wat::holon::HologramLRU` → `:wat::holon::lru::HologramCache`. The "LRU" qualifier moves from type name to namespace; the type name describes what the thing IS. | wat-rs `bf62426`, lab `a57ae55` |
| 2 | Add `:wat::holon::lru::HologramCacheService`. Lifts the lab's `:trading::cache::Service` verbatim under substrate naming. Six-step test progression (spawn+join, counted recv, Put-only, Put+Get round-trip, multi-client constructor, LRU eviction visible through service). | wat-rs `c0f6f2f` |
| 3 | Lab call-site sweep — `wat/cache/Service.wat` deleted, `L2-spawn.wat` repointed to substrate. Net −706 lines from the lab. | lab `e67861f` |
| 4 | Retrofit `:wat::lru::CacheService<K,V>` with Reporter + MetricsCadence. Both substrate cache services now follow one contract; the only slice that broke a shipped substrate API. | wat-rs `a02d647` |
| 5 | CONVENTIONS.md "Service contract — Reporter + MetricsCadence" section codified — the 11 contract elements, three cadence shapes, when to adopt vs opt out, the Console exception. | wat-rs `1fee6d3` |

---

## The contract as it landed

Eleven elements:

1. Typed Request enum (RPC methods as variants)
2. Typed Report enum (slice-1: only `Metrics`; future variants extend additively)
3. `Reporter` typealias = `:fn(Type::Report) -> :()`
4. `MetricsCadence<G>` struct = `{gate :G, tick :fn(G,Stats) -> :(G,bool)}`
5. `Stats` struct (counter set is service-defined)
6. `Type/null-reporter` + `Type/null-metrics-cadence` — opt-out is deliberate
7. `Type/spawn ... reporter metrics-cadence` — both injection points required
8. `Type/handle req state -> state'` — pure values-up dispatcher
9. `Type/tick-window state reporter cadence -> Step<G>` — always advances cadence
10. `Type/loop` — driver, threads State + Reporter + MetricsCadence
11. `Type/run` — worker entry; wraps the loop with storage construction

Both substrate cache services (`CacheService`,
`HologramCacheService`) ship all eleven. Console is the documented
exception — its tagged-stdout-write IS the report layer.

---

## Naming alignment as it landed

| L1 storage primitive | L2 queue-addressed wrapper |
|---|---|
| `:wat::lru::LocalCache<K,V>` | `:wat::lru::CacheService<K,V>` |
| `:wat::holon::lru::HologramCache` | `:wat::holon::lru::HologramCacheService` |

Crate directories match the namespace path:
- `crates/wat-lru/` → `:wat::lru::*`
- `crates/wat-holon-lru/` → `:wat::holon::lru::*`

---

## What this arc unblocked

- **Trader telemetry.** The lab's downstream Reporter
  implementations (sqlite-backed, CloudWatch-backed) land against a
  stable substrate target.
- **Future caches.** TTL-eviction, Redis-backed, etc. — all follow
  the same shape.
- **Future stdlib services.** The contract is a one-page recipe in
  CONVENTIONS.md.
- **MTG / truth-engine consumers.** Cross-domain consumers share
  the contract; their Reporter implementations vary.

---

## What this arc deliberately did NOT do

- No traits or shared `QueuedCache` abstraction. Per 058-030, the
  substrate has no traits; both cache services reach the same shape
  by convention. The duplication is the price of trait-free
  polymorphism; if it ever becomes painful, a future arc can extract
  a `wat/std/service/loop-template.wat` macro.
- No Reporter implementations. Substrate ships the contract;
  consumers ship their backends.
- No L0 (lock-free in-process) tier. If a future workload demands
  it, that's a separate arc.
- No retrofit of `MetricsCadence<G>` to a shared
  `:wat::std::service::MetricsCadence<G,Stats>`. Per-service keeps
  the Stats type honest at the cadence boundary; revisit when a
  third service surfaces.
