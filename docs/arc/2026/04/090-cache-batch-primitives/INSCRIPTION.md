# Arc 090 — Cache Batch Primitives — INSCRIPTION

**Status:** **superseded by arc 119** (work shipped 2026-05-01).
**Closure:** 2026-05-03.

---

## What happened

Arc 090 (drafted 2026-04-29) sketched batch primitives for `:wat::lru::CacheService` — `batch_get(Vec<K>) -> Vec<(K, Option<V>)>` and `batch_set(Vec<(K,V)>)` — and explicitly deferred implementation pending a consumer:

> When a consumer arrives that wants multi-key dispatch (likely: the indicator-bank port doing N feature lookups per candle), arc 090 ships.

Between draft and consumer arrival, **arc 119's substrate work generalized the pattern across every wat-rs-shipped service**. The promotion got codified in `docs/CONVENTIONS.md` § "Batch convention":

> Every wat-rs-shipped service exposes only batch-oriented `get` / `put` interfaces. Console is the single exception.

Arc 119 shipped the LRU + HolonLRU batch surfaces uniformly:
- **Get**: `(get probes :Vec<K>) -> Vec<Option<V>>` (Pattern B back-edge)
- **Put**: `(put entries :Vec<Entry<K,V>>) -> unit` (Pattern A back-edge)

Live in `crates/wat-lru/wat/lru/CacheService.wat` and `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`.

The verbs arc 090 sketched are live; the namespacing differs (`get`/`put` rather than `batch_get`/`batch_set`) because every service is batch-only by convention — the qualifier is implicit.

## Why this closes as superseded

The hammock-driven principle held: arc 090 waited for a real consumer, the broader doctrine arrived first, the wider work shipped under arc 119's number. The original deferral was correct. The eventual work was right-sized — substrate-wide, not 090-specific.

The predicted consumer (indicator-bank port at `holon-lab-trading/wat/encoding/indicator-bank`) inherits the verbs without arc 090 needing further substrate work.

## What 090 contributed

The DESIGN named the shape early. The `cache.rs:178-246` archived precedent + the `(get/put)` data-bearing/unit-ack pair shaped arc 119's eventual surface. The deferral discipline ("design for a real consumer, not a hypothetical") prevented a one-arc fix that arc 119 would have had to redo.

## References

- `docs/CONVENTIONS.md` § "Batch convention — substrate-shipped services (arc 119)"
- `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md`
- `docs/arc/2026/04/089-batch-as-protocol/INSCRIPTION.md` (Telemetry's prior batch path; the convention's first instance)
- `crates/wat-lru/wat/lru/CacheService.wat` (the live verbs)

---

**Arc 090 — closed as superseded by arc 119.**
