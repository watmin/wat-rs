# Arc 078 — Service contract: Reporter + MetricsCadence as substrate idiom

**Status:** SHIPPED 2026-04-29 across slices 0–5. See INSCRIPTION.md.

**Predecessors:**
- Arc 074 — `Hologram` + `HologramLRU` shipped. The bounded sibling lived in its own sibling crate `wat-hologram-lru`.
- Arc 076 — therm-routed Hologram; filter at construction.
- Arc 077 — kill the dim router; one program-d, capacity = `floor(sqrt(d))`.
- Lab 059-001 slice 1 — built `:trading::cache::Service` with Reporter + MetricsCadence pattern, mirroring the archive's `programs/stdlib/cache.rs::cache(can_emit, emit)` injection-point shape.

**Surfaced by:** Building the lab's L2 cache. After landing the Reporter / MetricsCadence pair on `:trading::cache::Service`, the user's recognition (2026-04-29):

> "did we just make something that should live in wat-rs/crate/ ? i think we did... we have HologramLRU and HologramLRUService ?... this could now live in the :wat::holon::* namespace?..."

> "the Reporter + MetricsCadence + null-* + Report-as-enum pattern is actually a SUBSTRATE-LEVEL service idiom — all service patterns (excluding Console) should do this..."

The lab is the wrong home for the cache service. Nothing in `:trading::cache::*` is trader-specific; the Request enum, the Reporter contract, the cadence-gated metrics — all of it is generic substrate machinery built atop `HologramLRU`. The trader merely *uses* it.

This arc lifts the substrate machinery into wat-rs and codifies the Reporter / MetricsCadence pattern as the canonical service-contract idiom. Future services follow it; existing services (`wat::lru::CacheService`) retrofit to it; Console stays as the documented exception (its tagged-stdout-write IS the report — no nested layer needed).

---

## What this arc is, and is not

**Is:**
- A crate-directory rename — `crates/wat-hologram-lru/` → `crates/wat-holon-lru/` to match the wat-side namespace path.
- A substrate-API rename — `:wat::holon::HologramLRU` → `:wat::holon::lru::HologramCache`. The "LRU" qualifier moves from the type name to the namespace; the type name says what the thing IS (a hologram-backed cache).
- A new substrate type — `:wat::holon::lru::HologramCacheService` — the queue-addressed wrapper, ported from lab's `:trading::cache::Service` with Reporter + MetricsCadence already wired.
- A retrofit of `:wat::lru::CacheService<K,V>` — pick up Reporter + MetricsCadence so all queue-addressed cache services share the same shape.
- A documented contract — `CONVENTIONS.md` grows a "Service contract" section codifying Reporter + MetricsCadence + null-helpers + typed Report enum as the canonical service idiom.
- A lab call-site sweep — `:trading::cache::*` consumers repoint to substrate; lab's `wat/cache/Service.wat` deletes; lab's `L1.wat` + `L2-spawn.wat` survive as **thin composition** (the trading app declares how it uses caches; the substrate provides primitives).

**Is not:**
- A retrofit of Console. Console's tagged-stdout-write IS the report layer; nesting another Reporter under it would be Reporter-of-Reporter ceremony with no information gain. Documented exception.
- A change to HologramCache's slot-routing or filter behavior. Arc 076's therm-routed semantics stay.
- A new arc for the trader's actual `Reporter` implementations (sqlite-backed, CloudWatch-backed). Those are downstream consumer work — when the trader hot path needs them, they ship.
- A trait-based polymorphism move. Per 058-030 the substrate has no traits; both `CacheService` and `HologramCacheService` keep their own concrete types and reach the same shape by convention, not by abstraction.

---

## Naming alignment

| L1 storage primitive | L2 queue-addressed wrapper |
|---|---|
| `:wat::lru::LocalCache<K,V>` | `:wat::lru::CacheService<K,V>` |
| `:wat::holon::lru::HologramCache` | `:wat::holon::lru::HologramCacheService` |

The pattern: `<Storage>` + `<Storage>Service`, both under their family namespace. The "LRU" qualifier rides at the namespace level (where the eviction policy belongs); type names describe what each thing IS.

Crate directories match the namespace path:
- `crates/wat-lru/` ships under `:wat::lru::*`.
- `crates/wat-holon-lru/` ships under `:wat::holon::lru::*` (renamed from `wat-hologram-lru`).

---

## The service contract

Every queue-addressed cache service ships:

1. **A typed Request enum** — what clients can ask. Variants ARE the RPC methods. Mirror of archive's `TreasuryRequest`.
2. **A typed Report enum** — what the service emits outbound. Producer-defined; consumer dispatches via match. Slice-1 ships only `(Metrics stats)`; future variants (Error, Evicted, Lifecycle) extend additively.
3. **A `Reporter` typealias** — `:fn(Type::Report) -> :()`. The user's match-dispatching consumer.
4. **A `MetricsCadence<G>` struct** — `{gate :G, tick :fn(G,Stats) -> :(G,bool)}`. The stateful rate gate. User picks `G`; substrate threads it through the loop, rebuilding the struct each iteration with the advanced gate.
5. **A `Stats` struct** — counter type emitted via `Report::Metrics`. Counter set is service-defined.
6. **`Type/null-reporter` + `Type/null-metrics-cadence`** — the explicit no-reporting choice. Caller must pick BOTH; opting out is a deliberate choice, not a default.
7. **`Type/spawn count cap reporter metrics-cadence`** — the constructor. Order encodes the contract: "here's your reporter, then here's how often you use it for metrics." Both arguments are non-negotiable.
8. **`Type/handle req state -> state'`** — the per-variant request dispatcher. Pure values-up.
9. **`Type/tick-window state reporter metrics-cadence -> Step<G>`** — gate-fire logic; ALWAYS advances the cadence; conditionally emits + resets stats. Named for what it always does, not the conditional branch.
10. **`Type/loop`** — driver, threads State + Reporter + MetricsCadence; selects + dispatches + ticks the window.
11. **`Type/run`** — worker entry; wraps the loop with the storage construction and dropping (per the thread-owned-cache discipline).

**Console exception.** The Console service writes to stdout/stderr through tagged messages; that IS its report layer. There's no inner Reporter to inject — the channel writes ARE the reports. This is the one stdlib service that opts out of the contract by virtue of being the report layer itself.

### Concrete shapes the user expresses

```scheme
;; Null path — both required to be passed deliberately
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :wat::holon::lru::HologramCacheService/null-reporter
  (:wat::holon::lru::HologramCacheService/null-metrics-cadence))

;; Time-based metrics gate — wall-clock tick-gate
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :my::reporter
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    (:wat::time::now)
    (:wat::core::lambda
      ((g :wat::time::Instant) (_s :Stats) -> :(wat::time::Instant,bool))
      (:trading::log::tick-gate g 5000))))

;; Counter-based — every 100 lookups
(:wat::holon::lru::HologramCacheService/spawn 2 16
  :my::reporter
  (:wat::holon::lru::HologramCacheService::MetricsCadence/new
    0
    (:wat::core::lambda ((n :i64) (_s :Stats) -> :(i64,bool))
      (:wat::core::if (:wat::core::i64::>= n 99) -> :(i64,bool)
        (:wat::core::tuple 0 true)
        (:wat::core::tuple (:wat::core::i64::+ n 1) false)))))
```

The user's `:my::reporter` is `:fn(Report) -> :()` — a closure that captures whatever stateful sink they want (sqlite handle, CloudWatch tx, stdout writer).

---

## Slice plan

Each slice is locally green at its boundary; no slice ships a broken workspace.

### Slice 0 — Crate directory rename

`crates/wat-hologram-lru/` → `crates/wat-holon-lru/`. Mechanical:

- Rename the directory.
- Update `Cargo.toml`'s `name = "wat-hologram-lru"` → `"wat-holon-lru"`.
- Update workspace root `Cargo.toml` `[workspace.members]` and `default-members`.
- Update lab repo's `Cargo.toml` dependency line.
- Update lab `tests/test.rs` `wat::test! { deps: [..., wat_hologram_lru, ...] }` → `wat_holon_lru`.
- Update any `mod wat_hologram_lru` imports in shim wiring.
- USER-GUIDE / docs sweep for the crate name.

The wat-side API surface stays the SAME this slice — `:wat::holon::HologramLRU` still works. Only the shipping vehicle's name changed. Tests pass with no wat changes.

### Slice 1 — Substrate API rename `HologramLRU` → `lru::HologramCache`

In the renamed crate:
- `wat/holon/HologramLRU.wat` → `wat/holon/lru/HologramCache.wat`.
- Substrate dispatch: `:wat::holon::HologramLRU/<method>` → `:wat::holon::lru::HologramCache/<method>` for `make`, `put`, `get`, `find`, `remove`, `len`, `capacity`.
- Type schemes in `check.rs`.
- The Rust struct `crate::hologram::Hologram` stays as-is (it's the SUBSTRATE primitive backing both `Hologram` AND `HologramCache`); the wat-side typealias `:wat::holon::HologramLRU` retires.
- Test sweep: `wat-tests/holon/lru/HologramCache.wat`.
- Lab consumers (`wat/cache/L1.wat`, `walker.wat`, `Service.wat`) update.
- USER-GUIDE / docs.

### Slice 2 — Add `:wat::holon::lru::HologramCacheService`

In the same crate:
- New file `wat/holon/lru/HologramCacheService.wat`. Ports the lab's `:trading::cache::Service` machinery verbatim under the substrate namespace:
  - `HologramCacheService::Stats` struct.
  - `HologramCacheService::Report` enum (slice-1 ships only `(Metrics stats)`).
  - `HologramCacheService::Reporter` typealias.
  - `HologramCacheService::MetricsCadence<G>` struct.
  - `HologramCacheService::State`, `HologramCacheService::Step<G>`.
  - `HologramCacheService::Request` enum, `ReqTx`, `ReqRx`, `ReqTxPool`, `Spawn`.
  - `HologramCacheService::GetReplyTx`, `GetReplyRx`.
  - `HologramCacheService/null-reporter`, `HologramCacheService/null-metrics-cadence`, `HologramCacheService/Stats/zero`.
  - `HologramCacheService/handle`, `HologramCacheService/tick-window`, `HologramCacheService/loop`, `HologramCacheService/run`, `HologramCacheService/spawn`.
- Tests: `wat-tests/holon/lru/HologramCacheService.wat` — ports the lab's six Service tests (step1-spawn-join through step6-lru-eviction-via-service) plus L2-spawn shape.

### Slice 3 — Lab call-site sweep

Lab repo:
- `wat/cache/Service.wat` deletes — substrate ships it now.
- `wat/cache/L1.wat` collapses to a thin wrapper: still owns the `:trading::cache::L1` struct (per-thinker dual cache), but its body is just `:wat::holon::lru::HologramCache/make` calls and accessor pass-throughs. The lab DECLARES how it uses caches; the substrate provides the primitives.
- `wat/cache/L2-spawn.wat` similar — keeps the `:trading::cache::L2` struct (cache-next + cache-terminal pair), but each spawn delegates to `:wat::holon::lru::HologramCacheService/spawn`. The lab still owns the L2-as-paired-services policy; the service implementation is substrate.
- `wat/cache/walker.wat` — uses `:wat::holon::lru::HologramCache` directly through the L1 wrapper.
- `wat-tests/cache/{L1,walker,L2-spawn,Service}.wat` — Service.wat deletes (substrate's tests cover it); the others adapt.

### Slice 4 — Retrofit `:wat::lru::CacheService<K,V>` (separate-shipped-API)

In `crates/wat-lru/`:
- Add `Stats`, `Report` (with `Metrics` variant), `Reporter`, `MetricsCadence<G>`, `null-reporter`, `null-metrics-cadence`.
- Adapt `CacheService/spawn` signature to take Reporter + MetricsCadence.
- Existing tests sweep.
- This is the only slice that breaks a SHIPPED substrate API (post-arc-074-slice-2). Worth doing in same arc since the contract IS the point — a half-retrofitted contract isn't a contract.

### Slice 5 — `CONVENTIONS.md` codification

`wat-rs/docs/CONVENTIONS.md` grows a section: "Service contract — Reporter + MetricsCadence". Documents:
- The 11 elements above (Request/Report enum, Reporter, MetricsCadence, Stats, null-helpers, /spawn / handle / tick-window / loop / run).
- The Console exception.
- When a service should adopt vs. opt out (any service that owns a queue + state benefits; trivial pure-fn services don't earn the contract).
- A working example showing the three cadence shapes (null, time, counter).

---

## What this arc deliberately does NOT do

- **Doesn't generalize over a "QueuedCache" trait or shared loop body.** Per 058-030 the substrate has no traits; `CacheService` and `HologramCacheService` are concrete types reaching the same shape by convention. If duplication of the loop body becomes painful, a future arc may extract a `wat/std/service/loop-template.wat` macro — but the duplication is the price of trait-free polymorphism, and slice 1 of this arc accepts it.
- **Doesn't change `Hologram`'s API.** Only the LRU sibling and its service surface move. The unbounded `Hologram` stays at `:wat::holon::Hologram` — it's the shared primitive backing the bounded sibling.
- **Doesn't ship Reporter implementations.** No sqlite-backed, no CloudWatch-backed. The substrate ships the contract; consumers ship their backends.
- **Doesn't move `RunDb` or `:trading::log::*`.** The lab's logging service stays in the lab — it's downstream of the cache contract, not part of it.
- **Doesn't add an L0 (in-process, lock-free) tier.** L1 and L2 are the two tiers we're naming; if a future workload demands a process-shared in-memory L0 (zero-Mutex, value-up), that's a separate arc.

---

## Open questions

### Q1 — Slice 4 ordering: with this arc, or split off?

`CacheService<K,V>` retrofit (slice 4) breaks a shipped substrate API. Two choices:
- (a) **Same arc.** The contract IS the point. Both consumers in lockstep avoids a half-retrofitted contract.
- (b) **Split to arc 079.** Keeps arc 078 strictly additive on `wat-holon-lru`; CacheService retrofit ships separately when consumers are quiet.

Default position: **(a)**. The user surfaced the contract recognition; doing it incompletely would document an idiom the substrate doesn't actually follow yet.

### Q2 — Lab L1/L2 keep their wrapper structs, or collapse to direct substrate use?

The user direction: **keep them thin.** The trading app declares how IT uses caches (one-cache-per-thinker for L1; paired-cache for L2). The substrate provides the primitives. The lab's `:trading::cache::L1` and `:trading::cache::L2` survive as policy — composing substrate types under domain-meaningful names.

### Q3 — Should `:wat::lru::*` move to `:wat::lru::*` keep its position, or also subnamespace?

Currently `wat::lru::LocalCache<K,V>` and `wat::lru::CacheService<K,V>` sit at top-level under `wat::lru`. The hologram-backed pair goes under `wat::holon::lru::*` (one level deeper because `holon` is its own family).

No rename for `wat::lru::*` proposed in this arc. The asymmetry is honest: `wat::lru::*` is the generic K/V family (only one storage backing); `wat::holon::lru::*` is one of potentially many holon-backed cache flavors (e.g., a future `wat::holon::ttl::*` for time-eviction).

### Q4 — Should `MetricsCadence<G>` be a substrate-shared type instead of per-service?

Each service currently ships its own `Type::MetricsCadence<G>`. Could lift to a shared `:wat::std::service::MetricsCadence<G,Stats>` taking the Stats type as a parameter.

Default position: **don't lift in this arc.** Per-service MetricsCadence keeps the type system honest — the cadence's `tick` knows about the service's specific Stats. Shared cadence would need to be `MetricsCadence<G, S>` where S is the service-specific stats — adds a generic without a clear payoff yet. Revisit when a third service surfaces.

---

## Test strategy

- Slice 0: workspace `cargo test` green after rename. No wat changes; just shipping-vehicle.
- Slice 1: existing `HologramLRU` tests rewritten as `HologramCache` tests; same coverage, renamed surface.
- Slice 2: port lab's six Service tests + two L2-spawn tests under substrate naming. Confirm the Reporter / MetricsCadence wiring works in a substrate harness.
- Slice 3: lab tests stay green at every change point; the deletion of `wat/cache/Service.wat` happens last (after lab consumers point at substrate).
- Slice 4: CacheService tests get Reporter + MetricsCadence parameter additions; existing semantics intact.
- Slice 5: doc-only, no test impact.

---

## What this unblocks

- **Anyone writing a queue-addressed substrate service** — the contract is a one-page recipe.
- **Trader telemetry** — the lab's Reporter implementations land downstream against a stable substrate target.
- **Future caches** — TTL-eviction, Redis-backed, etc. — all follow the same shape.
- **MTG / truth-engine / cross-domain consumers** — share the contract; their Reporter implementations vary.

PERSEVERARE.
