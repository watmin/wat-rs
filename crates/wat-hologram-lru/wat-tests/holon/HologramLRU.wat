;; wat-tests for arc 074 slice 2 — :wat::holon::HologramLRU.
;;
;; The bounded sibling of Hologram. Tests cover:
;;   - construction, len, dim
;;   - put + get round-trip (self-cosine = 1.0)
;;   - LRU bump on hit (recently-touched survives eviction)
;;   - LRU eviction at cap (oldest by LRU rank evicts; entry gone from Hologram)
;;   - Cell isolation under coincident-get (same as Hologram)
;;   - Coincident-get / present-get composition

;; ─── new + len: empty store ──────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-make-empty
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 16))
     ((n :i64) (:wat::holon::HologramLRU/len store))
     ((d :i64) (:wat::holon::HologramLRU/dim store)))
    (:wat::test::assert-eq
      (:wat::core::if (:wat::core::= n 0) -> :bool
        (:wat::core::= d 10000)
        false)
      true)))

;; ─── put + get round-trip: self-cosine = 1.0 ─────────────────────

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-put-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 16))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 5.0 k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── len tracks puts ─────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-len-tracks-puts
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 16))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
     ((_ :()) (:wat::holon::HologramLRU/put store  5.0 k1 v1))
     ((_ :()) (:wat::holon::HologramLRU/put store 80.0 k2 v2))
     ((n :i64) (:wat::holon::HologramLRU/len store)))
    (:wat::test::assert-eq n 2)))

;; ─── LRU eviction at capacity drops oldest from Hologram ────────
;;
;; cap=2; put 3 entries at positions all in the same cell-spread
;; range; the FIRST entry should be evicted from BOTH the LRU AND
;; the underlying Hologram cell. After 3 puts, len = 2 and the
;; first key's coincident-get returns None.

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-lru-evicts-from-hologram
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 2))
     ;; All three at pos=5.0 — same cell, so they fight for cap.
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k1 v))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k2 v))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k3 v))
     ;; Total entries = 2 (k1 evicted by k3's put).
     ((total :i64) (:wat::holon::HologramLRU/len store))
     ;; k1 specifically gone from Hologram.
     ((g1 :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 5.0 k1))
     ((k1-evicted :bool)
      (:wat::core::match g1 -> :bool
        ((Some _) false)
        (:None    true)))
     ;; k2 still there.
     ((g2 :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 5.0 k2))
     ((k2-present :bool)
      (:wat::core::match g2 -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq
      (:wat::core::if (:wat::core::= total 2) -> :bool
        (:wat::core::if k1-evicted -> :bool k2-present false)
        false)
      true)))

;; ─── LRU bump on get keeps hot entries warm ──────────────────────
;;
;; cap=2. put k1, put k2, GET k1 (bumps k1 to MRU), put k3. Eviction
;; should drop k2 (now LRU) instead of k1.

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-get-bumps-lru
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 2))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k1 v))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k2 v))
     ;; Get k1 — bumps it to MRU.
     ((_ :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 5.0 k1))
     ;; Now k2 is LRU; put k3 evicts k2.
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k3 v))
     ;; k1 should STILL be present (was MRU after the bump).
     ((g1 :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 5.0 k1))
     ((k1-present :bool)
      (:wat::core::match g1 -> :bool
        ((Some _) true)
        (:None    false)))
     ;; k2 should be evicted.
     ((g2 :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 5.0 k2))
     ((k2-evicted :bool)
      (:wat::core::match g2 -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq
      (:wat::core::if k1-present -> :bool k2-evicted false)
      true)))

;; ─── Cell isolation: distant cells stay isolated under coincident-get
;;
;; Mirrors Hologram's integ-cell-isolation. Coincident-get at a
;; distant pos returns None even when entries exist elsewhere.

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-cell-isolation
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 16))
     ((alpha :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((omega :wat::holon::HolonAST) (:wat::holon::leaf :omega))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))
     ((_ :()) (:wat::holon::HologramLRU/put store  5.0 alpha v))
     ((_ :()) (:wat::holon::HologramLRU/put store 80.0 omega v))
     ;; Coincident-get with alpha at pos=80 — omega is in cells
     ;; 80/81 with cosine far from alpha; coincident floor rejects.
     ((cross :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/coincident-get store 80.0 alpha))
     ((cross-none :bool)
      (:wat::core::match cross -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq cross-none true)))

;; ─── present-get accepts what coincident-get rejects ─────────────
;;
;; Only verifies that present-get is composable; the substrate
;; semantics are tested in the wat-rs core suite.

(:wat::test::deftest :wat-tests::holon::HologramLRU::test-present-get-composes
  ()
  (:wat::core::let*
    (((store :wat::holon::HologramLRU) (:wat::holon::HologramLRU/make 10000 16))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::HologramLRU/put store 5.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HologramLRU/present-get store 5.0 k))
     ((is-some :bool)
      (:wat::core::match got -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq is-some true)))
