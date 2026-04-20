;; :wat::std::LocalCache — the L1 tier of 058 FOUNDATION's caching
;; stack (lines 1527-1543). Single-thread-owned LRU; no pipe, no
;; thread, no queue. Fastest memoization possible.
;;
;; Built as three thin wrappers over the `lru` crate's LruCache,
;; exposed to wat via the :rust::lru::LruCache shim with its
;; thread-id scope guard. Zero Mutex — the guard is structural,
;; not contended.
;;
;; Usage:
;;   (let* (((cache :rust::lru::LruCache<String,i64>)
;;           (:wat::std::LocalCache::new 16))
;;          ((_ :()) (:wat::std::LocalCache::put cache "k" 42)))
;;     (:wat::core::match (:wat::std::LocalCache::get cache "k")
;;       ((Some v) v)
;;       (:None 0)))
;;
;; The cache's wat-level type IS `:rust::lru::LruCache<K,V>` — a
;; later typealias can hide that under `:wat::std::LocalCache<K,V>`
;; when wat's type checker supports alias expansion at unification.
;; The functional surface (new/put/get) is stable regardless.

(:wat::core::use! :rust::lru::LruCache)

(:wat::core::define
  (:wat::std::LocalCache::new<K,V>
    (capacity :i64)
    -> :rust::lru::LruCache<K,V>)
  (:rust::lru::LruCache::new capacity))

(:wat::core::define
  (:wat::std::LocalCache::put<K,V>
    (cache :rust::lru::LruCache<K,V>)
    (k :K)
    (v :V)
    -> :())
  (:rust::lru::LruCache::put cache k v))

(:wat::core::define
  (:wat::std::LocalCache::get<K,V>
    (cache :rust::lru::LruCache<K,V>)
    (k :K)
    -> :Option<V>)
  (:rust::lru::LruCache::get cache k))
