;; wat-tests/holon/Hologram.wat — tests for arc 074 slice 1.
;;
;; Hologram is a coordinate-cell store with cosine readout.
;; HolonAST → HolonAST. The user supplies pos: f64 in [0, 100];
;; substrate maps that to floor(sqrt(d)) cells. `get` walks the two
;; adjacent cells (or one, if pos is exactly on a boundary) and
;; cosine-matches each candidate against the probe; returns the val
;; of the highest-cosine entry whose cosine satisfies the user's
;; filter func.
;;
;;   new   :: i64 -> Hologram                   ; d as a positive int
;;   put   :: Hologram, f64, AST, AST -> ()    ; pos in [0, 100]
;;   get   :: Hologram, f64, AST, fn(f64)->bool -> Option<AST>
;;   len   :: Hologram -> i64

;; Filters are inline `:wat::core::lambda`s — defined functions can't
;; (yet) be referenced as values by their keyword path; the lambda
;; form is what binds to a `:fn(...)` type.

;; ─── new + len: empty store ───────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-new-empty
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 0)))

;; ─── put + len: count increments ──────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-put-increments-len
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── put idempotent at same key ───────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-put-idempotent-on-same-key
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k  :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k v1))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k v2))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── get hits self with permissive filter ─────────────────────────
;;
;; Self-cosine is 1.0; any reasonable filter accepts.

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((accept-near-one :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool)
        (:wat::core::> cos 0.95)))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 k accept-near-one))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── get returns None when filter rejects everything ──────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-rejects-via-filter
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((reject-all :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) false))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 k reject-all))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── get returns None on empty store ──────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-empty-returns-none
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 probe accept-any))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── get against a distant probe in the same cell ─────────────────
;;
;; Two unrelated atoms in the same cell — accept-any filter accepts
;; anything, so we get SOMETHING (the higher-cosine match wins).

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-distant-probe-still-returns
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :a-out))
     ((_  :()) (:wat::holon::Hologram/put store 5.0 k1 v1))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :unrelated))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 5.0 probe accept-any))
     ((is-some :bool)
      (:wat::core::match got -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq is-some true)))

;; ─── get does NOT find an entry in a distant cell ─────────────────
;;
;; pos=5 puts in cell 5; pos=80 looks in cells 80..81. Different
;; neighborhood, no candidates — None regardless of filter strictness.

(:wat::test::deftest :wat-tests::holon::Hologram::test-get-distant-cell-misses
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store 80.0 k accept-any))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── len counts entries across cells ──────────────────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-len-across-cells
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
     ((_ :()) (:wat::holon::Hologram/put store  5.0 k1 v1))
     ((_ :()) (:wat::holon::Hologram/put store 80.0 k2 v2))
     ((n :i64) (:wat::holon::Hologram/len store)))
    (:wat::test::assert-eq n 2)))

;; ─── presence-floor and coincident-floor expose substrate values ──

(:wat::test::deftest :wat-tests::holon::Hologram::test-presence-floor-positive
  ()
  (:wat::core::let*
    (((floor :f64) (:wat::holon::presence-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-floor-positive
  ()
  (:wat::core::let*
    (((floor :f64) (:wat::holon::coincident-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))

;; ─── dim accessor returns the d the store was built with ─────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-dim-returns-construction-d
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((d :i64) (:wat::holon::Hologram/dim store)))
    (:wat::test::assert-eq d 10000)))

;; ─── coincident-get composes filter-coincident at the store's d ──
;;
;; Self-cosine = 1.0 → coincident-filter accepts → Some(stored val).
;; The user passes no filter and no d; the convenience reads both
;; off the store.

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 5.0 k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── present-get composes filter-present at the store's d ────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-present-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store 5.0 k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/present-get store 5.0 k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── coincident-get on empty store returns None ──────────────────

(:wat::test::deftest :wat-tests::holon::Hologram::test-coincident-get-empty-none
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram) (:wat::holon::Hologram/new 10000))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/coincident-get store 5.0 probe))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))
