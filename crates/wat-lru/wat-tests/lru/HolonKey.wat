;; crates/wat-lru/wat-tests/lru/HolonKey.wat — deftests for the
;; HolonAST-as-cache-key surface (arc 057 slice 3).
;;
;; Background: arc 057 closed the algebra (typed leaves; HolonAST has
;; structural Hash + Eq derive). hashmap_key now accepts
;; Value::holon__HolonAST and the wat-lru shim no longer panics on
;; non-primitive keys. This unblocks lab arc 030 slice 2 (encoding
;; cache: HolonAST → Vector memoization for the predictor's hot path).
;;
;; Three properties under test, the minimum any cache layer needs to
;; trust holon keys:
;;   1. round-trip — put a holon key + value, get back the value
;;   2. distinguishes — structurally distinct holons land in distinct
;;      cache slots (no false positives)
;;   3. structural equal — two holons built independently but
;;      structurally equal hit the same cache slot (no false negatives,
;;      i.e. memoization actually works)
;;
;; All three are in-process; no driver threads, no channels — the
;; deftest sandbox suffices.


;; ─── round-trip ─────────────────────────────────────────────────────

(:wat::test::deftest :wat-lru::test-local-cache-holon-key-roundtrip
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<wat::holon::HolonAST,i64>)
      (:wat::lru::LocalCache::new 16))
     ((k :wat::holon::HolonAST)
      (:wat::holon::Atom (:wat::core::quote :the-form)))
     ((_ :()) (:wat::lru::LocalCache::put cache k 42))
     ((got :Option<i64>) (:wat::lru::LocalCache::get cache k))
     ((result :i64)
      (:wat::core::match got -> :i64
        ((Some v) v)
        (:None -1))))
    (:wat::test::assert-eq result 42)))


;; ─── distinguishes ──────────────────────────────────────────────────
;; Two structurally distinct holons must produce distinct cache slots:
;; storing under one and querying the other returns :None.

(:wat::test::deftest :wat-lru::test-local-cache-holon-key-distinguishes
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<wat::holon::HolonAST,i64>)
      (:wat::lru::LocalCache::new 16))
     ((k1 :wat::holon::HolonAST) (:wat::holon::Atom (:wat::core::quote :a)))
     ((k2 :wat::holon::HolonAST) (:wat::holon::Atom (:wat::core::quote :b)))
     ((_ :()) (:wat::lru::LocalCache::put cache k1 1))
     ((got :Option<i64>) (:wat::lru::LocalCache::get cache k2))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _v) false)
        (:None true))))
    (:wat::test::assert-eq is-none true)))


;; ─── structural equal ──────────────────────────────────────────────
;; Two holons built independently but structurally equal MUST collide
;; in the cache — this is the load-bearing property for memoization.
;; If the substrate's hash key included identity (Arc address) instead
;; of structure, this would miss; the put under k1 would never be
;; visible under k2. The arc 057 derived Hash gives the right shape.

(:wat::test::deftest :wat-lru::test-local-cache-holon-key-structural-equal
  ()
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<wat::holon::HolonAST,i64>)
      (:wat::lru::LocalCache::new 16))
     ((k1 :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::Atom (:wat::core::quote :role))
        (:wat::holon::Atom (:wat::core::quote :filler))))
     ((k2 :wat::holon::HolonAST)
      (:wat::holon::Bind
        (:wat::holon::Atom (:wat::core::quote :role))
        (:wat::holon::Atom (:wat::core::quote :filler))))
     ((_ :()) (:wat::lru::LocalCache::put cache k1 99))
     ((got :Option<i64>) (:wat::lru::LocalCache::get cache k2))
     ((result :i64)
      (:wat::core::match got -> :i64
        ((Some v) v)
        (:None -1))))
    (:wat::test::assert-eq result 99)))
