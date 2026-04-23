;; wat-tests/holon/Reject.wat — tests for wat/holon/Reject.wat + Project.wat.
;;
;; The Gram-Schmidt duo (058-005). Reject(x,y) carries x's component
;; ORTHOGONAL to y; Project(x,y) carries x's component ALONG y. Load-
;; bearing for the DDoS sidecar's anomaly detection (Challenge 010,
;; F1=1.000). Geometry is exact:
;;   presence(y, Reject(x,y))  → false  (by construction)
;;   presence(y, Project(x,y)) → true   (projection preserves direction)

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::test::deftest :wat-tests::holon::Reject::test-reject-strips-y-direction 1024 :error
  (:wat::core::let*
    (((x :wat::holon::HolonAST) (:wat::holon::Atom "x"))
     ((y :wat::holon::HolonAST) (:wat::holon::Atom "y"))
     ((residual :wat::holon::HolonAST) (:wat::holon::Reject x y)))
    (:wat::test::assert-eq (:wat::holon::presence? y residual) false)))

(:wat::test::deftest :wat-tests::holon::Reject::test-project-preserves-y-direction 1024 :error
  (:wat::core::let*
    (((x :wat::holon::HolonAST) (:wat::holon::Atom "x"))
     ((y :wat::holon::HolonAST) (:wat::holon::Atom "y"))
     ((shadow :wat::holon::HolonAST) (:wat::holon::Project x y)))
    (:wat::test::assert-eq (:wat::holon::presence? y shadow) true)))
