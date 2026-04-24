;; crates/wat-lru/wat-tests/lru.wat — deftests for wat-lru's
;; LocalCache surface (arc 015 slice 3).
;;
;; Exercises the single-thread-owned LRU: new / put / get /
;; overwrite-same-key / evict-at-capacity / miss-returns-None.
;; All tests are in-process — LocalCache has no driver threads, no
;; channels, so the `deftest`'s implicit sandbox suffices.
;;
;; Run via: `cargo test -p wat-lru`. The `tests/test.rs` file
;; invokes `wat::test! { path: "wat-tests", deps: [wat_lru] }`,
;; which routes through `wat::test_runner` with wat-lru composed in —
;; the same pipeline a downstream consumer uses when it declares
;; `deps: [wat_lru]`.


;; ─── put-then-get round-trip ────────────────────────────────────────

(:wat::test::deftest :wat-lru::test-local-cache-put-then-get
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<String,i64>)
      (:wat::lru::LocalCache::new 16))
     ((_ :()) (:wat::lru::LocalCache::put cache "answer" 42))
     ((got :Option<i64>)
      (:wat::lru::LocalCache::get cache "answer"))
     ((result :i64)
      (:wat::core::match got -> :i64
        ((Some v) v)
        (:None -1))))
    (:wat::test::assert-eq result 42)))

;; ─── miss returns :None ─────────────────────────────────────────────

(:wat::test::deftest :wat-lru::test-local-cache-miss-returns-none
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<String,i64>)
      (:wat::lru::LocalCache::new 16))
     ((got :Option<i64>)
      (:wat::lru::LocalCache::get cache "missing"))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _v) false)
        (:None true))))
    (:wat::test::assert-eq is-none true)))

;; ─── put overwrites existing key ────────────────────────────────────

(:wat::test::deftest :wat-lru::test-local-cache-put-overwrites
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<String,i64>)
      (:wat::lru::LocalCache::new 16))
     ((_ :()) (:wat::lru::LocalCache::put cache "k" 1))
     ((_ :()) (:wat::lru::LocalCache::put cache "k" 99))
     ((got :Option<i64>)
      (:wat::lru::LocalCache::get cache "k"))
     ((result :i64)
      (:wat::core::match got -> :i64
        ((Some v) v)
        (:None -1))))
    (:wat::test::assert-eq result 99)))

;; ─── evict at capacity ──────────────────────────────────────────────
;; Capacity 2: after putting 3 keys, the oldest (1) is evicted.

(:wat::test::deftest :wat-lru::test-local-cache-evict-at-capacity
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<i64,i64>)
      (:wat::lru::LocalCache::new 2))
     ((_ :()) (:wat::lru::LocalCache::put cache 1 10))
     ((_ :()) (:wat::lru::LocalCache::put cache 2 20))
     ((_ :()) (:wat::lru::LocalCache::put cache 3 30))
     ((got :Option<i64>)
      (:wat::lru::LocalCache::get cache 1))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _v) false)
        (:None true))))
    (:wat::test::assert-eq is-none true)))
