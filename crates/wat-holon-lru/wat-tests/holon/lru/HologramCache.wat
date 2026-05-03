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
;;
;; Arc 135 slice 4 — complectēns rewrite. Named helpers in a single
;; make-deftest prelude; per-helper deftests; LRU scenario deftests
;; reduced to ≤10 outer bindings.
;;
;; ─── Helpers ─────────────────────────────────────────────────────────
;;
;;   :test::hc-make (cap)         — construct HologramCache with filter-coincident
;;   :test::hc-fill-two (s k1 k2 v) — put k1 and k2 into store (shared LRU setup)
;;   :test::hc-get-found? (s k)   — Some → true, None → false
;;   :test::hc-get-evicted? (s k) — Some → false, None → true

(:wat::test::make-deftest :deftest
  (
   ;; ─── hc-make ─────────────────────────────────────────────────────
   ;; Construct a HologramCache with filter-coincident and the given cap.
   ;; The filter is always filter-coincident for these tests (self-cosine 1.0
   ;; satisfies it; structurally-distinct keys do not cross slots).
   (:wat::core::define
     (:test::hc-make
       (cap :wat::core::i64)
       -> :wat::holon::lru::HologramCache)
     (:wat::holon::lru::HologramCache/make
       (:wat::holon::filter-coincident)
       cap))

   ;; ─── hc-fill-two ─────────────────────────────────────────────────
   ;; Put k1 then k2 (same value v) into the store.
   ;; Shared setup for LRU tests that need two pre-existing entries
   ;; before exercising bump/eviction.
   (:wat::core::define
     (:test::hc-fill-two
       (store :wat::holon::lru::HologramCache)
       (k1 :wat::holon::HolonAST)
       (k2 :wat::holon::HolonAST)
       (v  :wat::holon::HolonAST)
       -> :wat::core::unit)
     (:wat::core::let*
       (((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k1 v))
        ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k2 v)))
       ()))

   ;; ─── hc-get-found? ───────────────────────────────────────────────
   ;; Returns true if the key is present (Some), false if evicted (None).
   (:wat::core::define
     (:test::hc-get-found?
       (store :wat::holon::lru::HologramCache)
       (k :wat::holon::HolonAST)
       -> :wat::core::bool)
     (:wat::core::match
       (:wat::holon::lru::HologramCache/get store k) -> :wat::core::bool
       ((:wat::core::Some _) true)
       (:wat::core::None    false)))

   ;; ─── hc-get-evicted? ─────────────────────────────────────────────
   ;; Returns true if the key is gone (None), false if still present (Some).
   ;; Complement of hc-get-found? — used in eviction assertions for clarity.
   (:wat::core::define
     (:test::hc-get-evicted?
       (store :wat::holon::lru::HologramCache)
       (k :wat::holon::HolonAST)
       -> :wat::core::bool)
     (:wat::core::match
       (:wat::holon::lru::HologramCache/get store k) -> :wat::core::bool
       ((:wat::core::Some _) false)
       (:wat::core::None    true)))
  ))

;; ─── per-helper deftests ──────────────────────────────────────────

(:deftest :wat-tests::holon::HologramCache::test-hc-make
  ;; hc-make constructs a non-empty-capacity store; len is 0 initially.
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 4))
     ((n :wat::core::i64) (:wat::holon::lru::HologramCache/len store)))
    (:wat::test::assert-eq n 0)))

(:deftest :wat-tests::holon::HologramCache::test-hc-fill-two
  ;; hc-fill-two puts exactly two distinct entries; len becomes 2.
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 4))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((v  :wat::holon::HolonAST) (:wat::holon::leaf :val))
     ((_ :wat::core::unit) (:test::hc-fill-two store k1 k2 v))
     ((n :wat::core::i64) (:wat::holon::lru::HologramCache/len store)))
    (:wat::test::assert-eq n 2)))

(:deftest :wat-tests::holon::HologramCache::test-hc-get-found
  ;; hc-get-found? returns true for a key that was just put.
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 4))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k v)))
    (:wat::test::assert-eq (:test::hc-get-found? store k) true)))

(:deftest :wat-tests::holon::HologramCache::test-hc-get-evicted
  ;; hc-get-evicted? returns true for a key that was pushed out by eviction.
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 1))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((v  :wat::holon::HolonAST) (:wat::holon::leaf :val))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k1 v))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k2 v)))
    (:wat::test::assert-eq (:test::hc-get-evicted? store k1) true)))

;; ─── make + len + capacity: empty store ──────────────────────────

(:deftest :wat-tests::holon::HologramCache::test-make-empty
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 16))
     ((n :wat::core::i64) (:wat::holon::lru::HologramCache/len store))
     ((cap :wat::core::i64) (:wat::holon::lru::HologramCache/capacity store)))
    (:wat::test::assert-eq
      (:wat::core::if (:wat::core::= n 0) -> :wat::core::bool
        (:wat::core::= cap 100)
        false)
      true)))

;; ─── put + get round-trip: self-cosine = 1.0 ─────────────────────

(:deftest :wat-tests::holon::HologramCache::test-put-get-self-hit
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 16))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── len tracks puts ─────────────────────────────────────────────

(:deftest :wat-tests::holon::HologramCache::test-len-tracks-puts
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 16))
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

(:deftest :wat-tests::holon::HologramCache::test-lru-evicts-from-hologram
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 2))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
     ((v  :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :wat::core::unit) (:test::hc-fill-two store k1 k2 v))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k3 v))
     ((total :wat::core::i64) (:wat::holon::lru::HologramCache/len store))
     ((k1-evicted :wat::core::bool) (:test::hc-get-evicted? store k1))
     ((k2-present :wat::core::bool) (:test::hc-get-found?   store k2)))
    (:wat::test::assert-eq
      (:wat::core::if (:wat::core::= total 2) -> :wat::core::bool
        (:wat::core::if k1-evicted -> :wat::core::bool k2-present false)
        false)
      true)))

;; ─── LRU bump on get keeps hot entries warm ──────────────────────
;;
;; cap=2. put k1, put k2, GET k1 (bumps k1 to MRU), put k3. Eviction
;; should drop k2 (now LRU) instead of k1.

(:deftest :wat-tests::holon::HologramCache::test-get-bumps-lru
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 2))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
     ((v  :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :wat::core::unit) (:test::hc-fill-two store k1 k2 v))
     ;; Get k1 — bumps it to MRU; k2 becomes LRU.
     ((_ :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k1))
     ;; put k3 evicts k2 (LRU).
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k3 v))
     ((k1-present :wat::core::bool) (:test::hc-get-found?   store k1))
     ((k2-evicted :wat::core::bool) (:test::hc-get-evicted? store k2)))
    (:wat::test::assert-eq
      (:wat::core::if k1-present -> :wat::core::bool k2-evicted false)
      true)))

;; ─── Therm-form round-trip via HologramCache ──────────────────────
;;
;; Confirms that a therm-routed key passes through the LRU layer
;; without losing identity. Self-cosine 1.0 satisfies the
;; coincidence filter.

(:deftest :wat-tests::holon::HologramCache::test-therm-roundtrip
  (:wat::core::let*
    (((store :wat::holon::lru::HologramCache) (:test::hc-make 16))
     ((k :wat::holon::HolonAST)
      (:wat::holon::therm-form 0.0 100.0 70.0))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :rsi-70-answer))
     ((_ :wat::core::unit) (:wat::holon::lru::HologramCache/put store k v))
     ((got :wat::core::Option<wat::holon::HolonAST>)
      (:wat::holon::lru::HologramCache/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((:wat::core::Some h) h)
        (:wat::core::None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))
