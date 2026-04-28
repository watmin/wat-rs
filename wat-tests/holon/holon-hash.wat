;; wat-tests/holon/holon-hash.wat — tests for arc 074 slice 1.
;;
;; HolonHash is a coordinate-cell store with cosine readout.
;; HolonAST → HolonAST. The user supplies pos: f64 in [0, 100];
;; substrate maps that to floor(sqrt(d)) cells. `get` walks the two
;; adjacent cells (or one, if pos is exactly on a boundary) and
;; cosine-matches each candidate against the probe; returns the val
;; of the highest-cosine entry whose cosine satisfies the user's
;; filter func.
;;
;;   new   :: i64 -> HolonHash                   ; d as a positive int
;;   put   :: HolonHash, f64, AST, AST -> ()    ; pos in [0, 100]
;;   get   :: HolonHash, f64, AST, fn(f64)->bool -> Option<AST>
;;   len   :: HolonHash -> i64

;; Filters are inline `:wat::core::lambda`s — defined functions can't
;; (yet) be referenced as values by their keyword path; the lambda
;; form is what binds to a `:fn(...)` type.

;; ─── new + len: empty store ───────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::holon-hash::test-new-empty
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((n :i64) (:wat::holon::HolonHash/len store)))
    (:wat::test::assert-eq n 0)))

;; ─── put + len: count increments ──────────────────────────────────

(:wat::test::deftest :wat-tests::holon::holon-hash::test-put-increments-len
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_  :()) (:wat::holon::HolonHash/put store 5.0 k v))
     ((n :i64) (:wat::holon::HolonHash/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── put idempotent at same key ───────────────────────────────────

(:wat::test::deftest :wat-tests::holon::holon-hash::test-put-idempotent-on-same-key
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k  :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
     ((_  :()) (:wat::holon::HolonHash/put store 5.0 k v1))
     ((_  :()) (:wat::holon::HolonHash/put store 5.0 k v2))
     ((n :i64) (:wat::holon::HolonHash/len store)))
    (:wat::test::assert-eq n 1)))

;; ─── get hits self with permissive filter ─────────────────────────
;;
;; Self-cosine is 1.0; any reasonable filter accepts.

(:wat::test::deftest :wat-tests::holon::holon-hash::test-get-self-hit
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::HolonHash/put store 5.0 k v))
     ((accept-near-one :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool)
        (:wat::core::> cos 0.95)))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HolonHash/get store 5.0 k accept-near-one))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))

;; ─── get returns None when filter rejects everything ──────────────

(:wat::test::deftest :wat-tests::holon::holon-hash::test-get-rejects-via-filter
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::HolonHash/put store 5.0 k v))
     ((reject-all :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) false))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HolonHash/get store 5.0 k reject-all))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── get returns None on empty store ──────────────────────────────

(:wat::test::deftest :wat-tests::holon::holon-hash::test-get-empty-returns-none
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HolonHash/get store 5.0 probe accept-any))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── get against a distant probe in the same cell ─────────────────
;;
;; Two unrelated atoms in the same cell — accept-any filter accepts
;; anything, so we get SOMETHING (the higher-cosine match wins).

(:wat::test::deftest :wat-tests::holon::holon-hash::test-get-distant-probe-still-returns
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :a-out))
     ((_  :()) (:wat::holon::HolonHash/put store 5.0 k1 v1))
     ((probe :wat::holon::HolonAST) (:wat::holon::leaf :unrelated))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HolonHash/get store 5.0 probe accept-any))
     ((is-some :bool)
      (:wat::core::match got -> :bool
        ((Some _) true)
        (:None    false))))
    (:wat::test::assert-eq is-some true)))

;; ─── get does NOT find an entry in a distant cell ─────────────────
;;
;; pos=5 puts in cell 5; pos=80 looks in cells 80..81. Different
;; neighborhood, no candidates — None regardless of filter strictness.

(:wat::test::deftest :wat-tests::holon::holon-hash::test-get-distant-cell-misses
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::HolonHash/put store 5.0 k v))
     ((accept-any :fn(f64)->bool)
      (:wat::core::lambda ((cos :f64) -> :bool) true))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::HolonHash/get store 80.0 k accept-any))
     ((is-none :bool)
      (:wat::core::match got -> :bool
        ((Some _) false)
        (:None    true))))
    (:wat::test::assert-eq is-none true)))

;; ─── len counts entries across cells ──────────────────────────────

(:wat::test::deftest :wat-tests::holon::holon-hash::test-len-across-cells
  ()
  (:wat::core::let*
    (((store :wat::holon::HolonHash) (:wat::holon::HolonHash/new 10000))
     ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
     ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
     ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
     ((_ :()) (:wat::holon::HolonHash/put store  5.0 k1 v1))
     ((_ :()) (:wat::holon::HolonHash/put store 80.0 k2 v2))
     ((n :i64) (:wat::holon::HolonHash/len store)))
    (:wat::test::assert-eq n 2)))

;; ─── presence-floor and coincident-floor expose substrate values ──

(:wat::test::deftest :wat-tests::holon::holon-hash::test-presence-floor-positive
  ()
  (:wat::core::let*
    (((floor :f64) (:wat::holon::presence-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))

(:wat::test::deftest :wat-tests::holon::holon-hash::test-coincident-floor-positive
  ()
  (:wat::core::let*
    (((floor :f64) (:wat::holon::coincident-floor 10000)))
    (:wat::test::assert-eq (:wat::core::> floor 0.0) true)))
