;; :user::wat::std::lru::LocalCache — wat-lru's LRU surface.
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
;;   (let* (((cache :user::wat::std::lru::LocalCache<String,i64>)
;;           (:user::wat::std::lru::LocalCache::new 16))
;;          ((_ :()) (:user::wat::std::lru::LocalCache::put cache "k" 42)))
;;     (:wat::core::match (:user::wat::std::lru::LocalCache::get cache "k") -> :i64
;;       ((Some v) v)
;;       (:None 0)))

(:wat::core::use! :rust::lru::LruCache)

;; Wat-native type name. The Rust backing is :rust::lru::LruCache<K,V>;
;; unify's alias expansion walks through at every use site, so
;; :user::wat::std::lru::LocalCache<K,V> and the backing are
;; interchangeable.
(:wat::core::typealias :user::wat::std::lru::LocalCache<K,V> :rust::lru::LruCache<K,V>)

(:wat::core::define
  (:user::wat::std::lru::LocalCache::new<K,V>
    (capacity :i64)
    -> :user::wat::std::lru::LocalCache<K,V>)
  (:rust::lru::LruCache::new capacity))

(:wat::core::define
  (:user::wat::std::lru::LocalCache::put<K,V>
    (cache :user::wat::std::lru::LocalCache<K,V>)
    (k :K)
    (v :V)
    -> :())
  (:rust::lru::LruCache::put cache k v))

(:wat::core::define
  (:user::wat::std::lru::LocalCache::get<K,V>
    (cache :user::wat::std::lru::LocalCache<K,V>)
    (k :K)
    -> :Option<V>)
  (:rust::lru::LruCache::get cache k))
