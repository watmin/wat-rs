;; wat-tests/std/Reject.wat — tests for wat/std/Reject.wat + Project.wat.
;;
;; The Gram-Schmidt duo (058-005). Reject(x,y) carries x's component
;; ORTHOGONAL to y; Project(x,y) carries x's component ALONG y. Load-
;; bearing for the DDoS sidecar's anomaly detection (Challenge 010,
;; F1=1.000). Geometry is exact:
;;   presence(y, Reject(x,y))  → false  (by construction)
;;   presence(y, Project(x,y)) → true   (projection preserves direction)

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::test::deftest :wat-tests::std::Reject::test-reject-strips-y-direction 1024 :error
  (:wat::core::let*
    (((x :holon::HolonAST) (:wat::algebra::Atom "x"))
     ((y :holon::HolonAST) (:wat::algebra::Atom "y"))
     ((residual :holon::HolonAST) (:wat::std::Reject x y)))
    (:wat::test::assert-eq (:wat::algebra::presence? y residual) false)))

(:wat::test::deftest :wat-tests::std::Reject::test-project-preserves-y-direction 1024 :error
  (:wat::core::let*
    (((x :holon::HolonAST) (:wat::algebra::Atom "x"))
     ((y :holon::HolonAST) (:wat::algebra::Atom "y"))
     ((shadow :holon::HolonAST) (:wat::std::Project x y)))
    (:wat::test::assert-eq (:wat::algebra::presence? y shadow) true)))
