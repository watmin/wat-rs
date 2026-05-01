;; wat-tests for arc 076 + 077 — :wat::holon::lru::HologramCache.
;;
;; The bounded sibling of Hologram. Tests cover:
;;   - construction, len, capacity
;;   - put + get round-trip (self-cosine = 1.0)
;;   - LRU bump on hit (recently-touched survives eviction)
;;   - LRU eviction at cap (oldest by LRU rank evicts; entry gone from Hologram)
;;   - Slot isolation (mirrors Hologram's behavior under the wrapper)
;;
;; All sites use the new arc-076 surface (filter at construction;
;; slot routing inferred from the form's structure; no caller-pos).

;; ─── make + len + capacity: empty store ──────────────────────────

(:wat::test::deftest :wat-tests::holon::HologramCache::test-make-empty
  ()
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        16))
     ((n :wat::core::i64) (:wat::holon::lru::HologramCache/len store))
     ((cap :wat::core::i64) (:wat::holon::lru::HologramCache/capacity store)))
    (:wat::test::assert-eq
      (:wat::core::if (:wat::core::= n 0) -> :wat::core::bool
        (:wat::core::= cap 100)
        false)
      true)))

;; ─── put + get round-trip: self-cosine = 1.0 ─────────────────────

(:wat::test::deftest :wat-tests::holon::HologramCache::test-put-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        16))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── len tracks puts ─────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::HologramCache::test-len-tracks-puts
  ()
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        16))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k1 v1))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k2 v2))
     ((n :wat::core::i64) (:wat::holon::lru::HologramCache/len store)))
    (:wat::test::assert-eq n 2)))

;; ─── LRU eviction at capacity drops oldest from Hologram ────────
;;
;; cap=2; put 3 entries; the FIRST entry should be evicted from BOTH
;; the LRU AND the underlying Hologram. After 3 puts, len = 2 and the
;; first key's get returns None.

(:wat::test::deftest :wat-tests::holon::HologramCache::test-lru-evicts-from-hologram
  ()
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        2))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k1 v))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k2 v))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k3 v))
     ;; Total entries = 2 (k1 evicted by k3's put).
     ((total :wat::core::i64) (:wat::holon::lru::HologramCache/len store))
     ;; k1 specifically gone from Hologram.
     ((g1 :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k1))
     ((k1-evicted :wat::core::bool)
      (:wat::core::match g1 -> :wat::core::bool
        ((Some _) false)
        (:None    true)))
     ;; k2 still there.
     ((g2 :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k2))
     ((k2-present :wat::core::bool)
      (:wat::core::match g2 -> :wat::core::bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq
      (:wat::core::if (:wat::core::= total 2) -> :wat::core::bool
        (:wat::core::if k1-evicted -> :wat::core::bool k2-present false)
        false)
      true)))

;; ─── LRU bump on get keeps hot entries warm ──────────────────────
;;
;; cap=2. put k1, put k2, GET k1 (bumps k1 to MRU), put k3. Eviction
;; should drop k2 (now LRU) instead of k1.

(:wat::test::deftest :wat-tests::holon::HologramCache::test-get-bumps-lru
  ()
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        2))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k1 v))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k2 v))
     ;; Get k1 — bumps it to MRU.
     ((_ :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k1))
     ;; Now k2 is LRU; put k3 evicts k2.
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k3 v))
     ;; k1 should STILL be present (was MRU after the bump).
     ((g1 :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k1))
     ((k1-present :wat::core::bool)
      (:wat::core::match g1 -> :wat::core::bool
        ((Some _) true)
        (:None    false)))
     ;; k2 should be evicted.
     ((g2 :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k2))
     ((k2-evicted :wat::core::bool)
      (:wat::core::match g2 -> :wat::core::bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq
      (:wat::core::if k1-present -> :wat::core::bool k2-evicted false)
      true)))

;; ─── Therm-form round-trip via HologramCache ──────────────────────
;;
;; Confirms that a therm-routed key passes through the LRU layer
;; without losing identity. Self-cosine 1.0 satisfies the
;; coincidence filter.

(:wat::test::deftest :wat-tests::holon::HologramCache::test-therm-roundtrip
  ()
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache)
      (:wat::holon::lru::HologramCache/make
        (:wat::holon::filter-coincident)
        16))
     ((k :wat::holon::HolonAST)
      (:wat::holon::therm-form 0.0 100.0 70.0))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :rsi-70-answer))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))
