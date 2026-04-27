;; wat-tests/holon/Reject.wat — tests for wat/holon/Reject.wat + Project.wat.
;;
;; The Gram-Schmidt duo (058-005). Reject(x,y) carries x's component
;; ORTHOGONAL to y; Project(x,y) carries x's component ALONG y. Load-
;; bearing for the DDoS sidecar's anomaly detection (Challenge 010,
;; F1=1.000). Geometry is exact when x ACTUALLY HAS y as a component:
;;
;;   x = Bundle(y, noise)            ; x explicitly contains y
;;   presence(y, Reject(x,y))  → false   ; y removed by orthogonalization
;;   presence(y, Project(x,y)) → true    ; y preserved by projection
;;
;; Pre-arc-067 these tests used `x = Atom("x")`, `y = Atom("y")` —
;; two random independent atoms. At d=256's noise floor (1/sqrt(256)
;; ≈ 0.0625), random vectors had enough crosstalk that Project's
;; tiny y-component (dot-product ratio ~0.06) survived ternary
;; thresholding and the test passed — for the wrong reason. At
;; d=10000 (post-arc-067 default) the cleaner geometry zeroes the
;; sub-threshold component; the random-atoms framing demonstrated
;; noise, not the primitive's actual behavior.
;;
;; Honest reference: x must ACTUALLY contain y as part of its
;; structure. The bundle gives x a real y-component; Reject removes
;; it; Project preserves it. The geometric claim now holds at the
;; default d, and the tests are proper reference material for
;; callers building on the primitives.


(:wat::test::deftest :wat-tests::holon::Reject::test-reject-strips-y-direction
  ;; Helper visible to the test body — bundle-or-fail wraps the
  ;; BundleResult match into a single-shot HolonAST producer for
  ;; small-arity bundles where capacity is never exceeded.
  ((:wat::core::define
     (:wat-tests::holon::Reject::bundle-or-fail
       (a :wat::holon::HolonAST)
       (b :wat::holon::HolonAST)
       -> :wat::holon::HolonAST)
     (:wat::core::match
       (:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST a b))
       -> :wat::holon::HolonAST
       ((Ok h) h)
       ((Err _) (:wat::holon::leaf 0)))))
  (:wat::core::let*
    (((y :wat::holon::HolonAST) (:wat::holon::Atom "y"))
     ((noise :wat::holon::HolonAST) (:wat::holon::Atom "noise"))
     ;; x contains y plus a noise atom — x has a real y-component.
     ((x :wat::holon::HolonAST)
      (:wat-tests::holon::Reject::bundle-or-fail y noise))
     ((residual :wat::holon::HolonAST) (:wat::holon::Reject x y)))
    (:wat::test::assert-eq (:wat::holon::presence? y residual) false)))


(:wat::test::deftest :wat-tests::holon::Reject::test-project-preserves-y-direction
  ((:wat::core::define
     (:wat-tests::holon::Reject::bundle-or-fail
       (a :wat::holon::HolonAST)
       (b :wat::holon::HolonAST)
       -> :wat::holon::HolonAST)
     (:wat::core::match
       (:wat::holon::Bundle (:wat::core::vec :wat::holon::HolonAST a b))
       -> :wat::holon::HolonAST
       ((Ok h) h)
       ((Err _) (:wat::holon::leaf 0)))))
  (:wat::core::let*
    (((y :wat::holon::HolonAST) (:wat::holon::Atom "y"))
     ((noise :wat::holon::HolonAST) (:wat::holon::Atom "noise"))
     ;; x contains y plus a noise atom — x has a real y-component
     ;; for Project to preserve.
     ((x :wat::holon::HolonAST)
      (:wat-tests::holon::Reject::bundle-or-fail y noise))
     ((shadow :wat::holon::HolonAST) (:wat::holon::Project x y)))
    (:wat::test::assert-eq (:wat::holon::presence? y shadow) true)))
