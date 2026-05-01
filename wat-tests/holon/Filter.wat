;; wat-tests/holon/Filter.wat — tests for the substrate-default
;; filter factories that ride on Hologram/get.
;;
;; Each factory takes the encoder dim and returns a closure that
;; gates on the corresponding floor. Verified by comparing the
;; closure's behavior against threshold values just inside / outside
;; the floor.

;; ─── filter-coincident gates on coincident floor ──────────────────

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-coincident-rejects-far
  ()
  (:wat::core::let*
    (((f :fn(wat::core::f64)->wat::core::bool) (:wat::holon::filter-coincident)))
    ;; cosine 0.0 means orthogonal — far from coincident.
    (:wat::test::assert-eq (f 0.0) false)))

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-coincident-accepts-near-one
  ()
  (:wat::core::let*
    (((f :fn(wat::core::f64)->wat::core::bool) (:wat::holon::filter-coincident)))
    ;; cosine 0.9999 — very close to 1.0; (1 - cos) = 0.0001;
    ;; coincident floor at d=10000 with sigma=1 is 1/sqrt(10000) = 0.01.
    ;; 0.0001 < 0.01 → true.
    (:wat::test::assert-eq (f 0.9999) true)))

;; ─── filter-present gates on presence floor ───────────────────────

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-present-rejects-zero
  ()
  (:wat::core::let*
    (((f :fn(wat::core::f64)->wat::core::bool) (:wat::holon::filter-present)))
    ;; cosine 0.0 — no signal at all; below the noise floor.
    (:wat::test::assert-eq (f 0.0) false)))

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-present-accepts-strong
  ()
  (:wat::core::let*
    (((f :fn(wat::core::f64)->wat::core::bool) (:wat::holon::filter-present)))
    ;; cosine 0.9 — strong signal; well above presence floor.
    (:wat::test::assert-eq (f 0.9) true)))

;; ─── filter-accept-any always returns true ───────────────────────

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-accept-any-on-zero
  ()
  (:wat::core::let*
    (((f :fn(wat::core::f64)->wat::core::bool) (:wat::holon::filter-accept-any)))
    (:wat::test::assert-eq (f 0.0) true)))

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-accept-any-on-negative
  ()
  (:wat::core::let*
    (((f :fn(wat::core::f64)->wat::core::bool) (:wat::holon::filter-accept-any)))
    ;; even pathological inputs: anti-correlated cosine still passes.
    (:wat::test::assert-eq (f -1.0) true)))

;; ─── End-to-end: filter-coincident bound at construction ────────
;;
;; Arc 076: the filter is bound at Hologram/make time, not per-call.
;; Build a Hologram with the substrate-default coincidence filter,
;; put a (k, v) pair, get with no filter arg. Self-cosine is 1.0 →
;; filter accepts → Some.

(:wat::test::deftest :wat-tests::holon::Filter::test-filter-coincident-composes-with-get
  ()
  (:wat::core::let*
    (((store :wat::holon::Hologram)
      (:wat::holon::Hologram/make
        (:wat::holon::filter-coincident)))
     ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
     ((v :wat::holon::HolonAST) (:wat::holon::leaf :beta))
     ((_ :()) (:wat::holon::Hologram/put store k v))
     ((got :Option<wat::holon::HolonAST>)
      (:wat::holon::Hologram/get store k))
     ((found :wat::holon::HolonAST)
      (:wat::core::match got -> :wat::holon::HolonAST
        ((Some h) h)
        (:None    (:wat::holon::leaf :unreachable)))))
    (:wat::test::assert-eq found v)))
