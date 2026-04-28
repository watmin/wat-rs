;; :wat::lru::LocalCache — wat-lru's LRU surface.
;;
;; Repathed from wat-rs's former :wat::std::LocalCache when arc 013
;; externalized this crate (slice 4b). Single-thread-owned LRU; no
;; pipe, no thread, no queue. Fastest memoization possible.
;;
;; Built as three thin wrappers over the `lru` crate's LruCache,
;; exposed to wat via the :rust::lru::LruCache shim (this crate's
;; Rust side) with its thread-id scope guard. Zero Mutex — the
;; guard is structural, not contended.
;;
;; Usage:
;;   (let* (((cache :wat::lru::LocalCache<String,i64>)
;;           (:wat::lru::LocalCache::new 16))
;;          ((_ :()) (:wat::lru::LocalCache::put cache "k" 42)))
;;     (:wat::core::match (:wat::lru::LocalCache::get cache "k") -> :i64
;;       ((Some v) v)
;;       (:None 0)))

(:wat::core::use! :rust::lru::LruCache)

;; Wat-native type name. The Rust backing is :rust::lru::LruCache<K,V>;
;; unify's alias expansion walks through at every use site, so
;; :wat::lru::LocalCache<K,V> and the backing are
;; interchangeable.
(:wat::core::typealias :wat::lru::LocalCache<K,V> :rust::lru::LruCache<K,V>)

(:wat::core::define
  (:wat::lru::LocalCache::new<K,V>
    (capacity :i64)
    -> :wat::lru::LocalCache<K,V>)
  (:rust::lru::LruCache::new capacity))

(:wat::core::define
  (:wat::lru::LocalCache::put<K,V>
    (cache :wat::lru::LocalCache<K,V>)
    (k :K)
    (v :V)
    -> :Option<(K,V)>)
  (:rust::lru::LruCache::put cache k v))

(:wat::core::define
  (:wat::lru::LocalCache::get<K,V>
    (cache :wat::lru::LocalCache<K,V>)
    (k :K)
    -> :Option<V>)
  (:rust::lru::LruCache::get cache k))

;; `:wat::lru::LocalCache::len cache` — current entry count. Read-only;
;; does not affect LRU order. Lab cache services (umbrella 059
;; slice 1) emit this through rundb telemetry on a rate gate.
(:wat::core::define
  (:wat::lru::LocalCache::len<K,V>
    (cache :wat::lru::LocalCache<K,V>)
    -> :i64)
  (:rust::lru::LruCache::len cache))
