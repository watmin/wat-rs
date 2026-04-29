# Arc 090 — Cache Batch Primitives — DESIGN (skeleton)

**Status:** deferred. Skeleton only — written 2026-04-29 to make the future
shape obvious on disk. Pick up when a consumer needs it.

The current `:wat::lru::CacheService` (in `crates/wat-lru/`) ships per-key
`get key` and `put key value`. Each call is one round-trip through the
driver. The archive's pre-wat-native cache (`archived/pre-wat-native/src/programs/stdlib/cache.rs`)
proved a different shape: `batch_get(Vec<K>) -> Vec<(K, Option<V>)>` and
`batch_set(Vec<(K,V)>)`. One round-trip for N keys. Driver drains all
clients, services writes-first-reads-second, responds to all at once
(cache.rs:178-246).

## Why deferred

No current wat-side consumer of `CacheService` does multi-key work in a
single dispatch. The lab uses sqlite, not LRU. The holon-rs ports use the
substrate cache through holon's own caching path, not `:wat::lru::*`. Adding
batch primitives now would be substrate work without a caller — the
hammock-driven version of "design for a real consumer, not a hypothetical."

When a consumer arrives that wants multi-key dispatch (likely: the
indicator-bank port doing N feature lookups per candle), arc 090 ships.

## Sketch (when picked up)

Two new primitives at the wat-lru layer:

```scheme
(:wat::core::define
  (:wat::lru::CacheService/batch-get<K,V>
    (req-tx :wat::lru::CacheService::ReqTx<K,V>)
    (reply-tx :wat::lru::CacheService::ReplyTx<V>)
    (reply-rx :rust::crossbeam_channel::Receiver<Option<V>>)
    (keys :Vec<K>)
    -> :Vec<(K,Option<V>)>)
  ...)

(:wat::core::define
  (:wat::lru::CacheService/batch-set<K,V>
    (req-tx ...)
    (reply-tx ...)
    (reply-rx ...)
    (entries :Vec<(K,V)>)
    -> :())
  ...)
```

Per-key wrappers stay one-line:

```scheme
(:wat::core::define
  (:wat::lru::CacheService/get<K,V> ... (key :K) -> :Option<V>)
  (:wat::core::let*
    (((results :Vec<(K,Option<V>)>)
      (:wat::lru::CacheService/batch-get req-tx reply-tx reply-rx
        (:wat::core::vec :K key))))
    (:wat::core::match (:wat::core::first results) -> :Option<V>
      ((Some pair) (:wat::core::second pair))
      (:None :None))))
```

The Body protocol grows two new tags (BatchGet=2, BatchSet=3) — additive,
no breaking change. The driver's loop-step grows two new arms.

Cross-client drain (cache.rs:178-196) is the second half of this arc: the
loop should drain all clients before processing, services writes-first-then-reads,
respond to all in one pass. **Decision deferred:** does this matter at our
scale, or do we ship batch primitives without cross-client drain and revisit
when fanout matters?

## What's NOT in this arc

- The substrate cache (`:wat::std::cache::*` in wat-rs proper) is a separate
  surface from wat-lru's CacheService. It's not currently service-shaped
  (it's holon-rs-backed, in-process). If batch lookups become a substrate
  concern, that's a third arc.

## Open until

Picked up when a real consumer wants batched cache calls. Don't pre-empt —
the archive's batch_get worked because the broker had a Vec of paper_ids per
candle. Without that real shape on the wat side, the design's premise is
hypothetical.
