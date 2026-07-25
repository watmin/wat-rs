;; Co-located fixture for probe_arc278_cache_lru.rs — arc 278 Cache Stone 1 acceptance gate.
;;
;; Proves the baked `:wat::cache::Lru` primitive (src/rust_deps/cache.rs + wat/cache.wat — a
;; fresh, thread-owned bounded LRU over the `lru` crate) round-trips new/put/get/len AND that
;; the bound is real: a cap-2 cache holding {:a :b} evicts `:a` when `:c` arrives, handing the
;; evicted pair back as a NAMED `:wat::cache::Entry` (key/value), not a positional tuple.
;;
;; Keys are KEYWORDS and values are i64 — the generic `<K,V>` carries arbitrary hashable EDN,
;; it is not narrowed to String/i64.

(:wat::core::use! :rust::cache::Lru)

(:wat::test::deftest :user::cache_lru
  (:wat::core::let
    [cache (:wat::cache::Lru::new 2)
     e1    (:wat::cache::Lru::put cache :a 1)   ;; under cap  -> None
     e2    (:wat::cache::Lru::put cache :b 2)   ;; at cap     -> None
     e3    (:wat::cache::Lru::put cache :c 3)   ;; over cap   -> Some Entry{:a 1}
     got-b (:wat::cache::Lru::get cache :b)     ;; still present
     got-a (:wat::cache::Lru::get cache :a)     ;; evicted
     n     (:wat::cache::Lru::len cache)]

    (:wat::test::assert-eq e1 :wat::core::None)
    (:wat::test::assert-eq e2 :wat::core::None)
    (:wat::test::assert-eq e3 (:wat::core::Some (:wat::cache::Entry :key :a :value 1)))
    (:wat::test::assert-eq got-b (:wat::core::Some 2))
    (:wat::test::assert-eq got-a :wat::core::None)
    (:wat::test::assert-eq n 2)))
