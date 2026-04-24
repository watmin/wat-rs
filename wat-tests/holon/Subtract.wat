;; wat-tests/holon/Subtract.wat — tests for wat/holon/Subtract.wat.
;;
;; :wat::holon::Subtract (058-019) expands to (Blend x y 1 -1). The
;; canonical use case: build a vector that "anchors on x while
;; inverting y" so measuring presence of x against the result lands
;; above the noise floor, and measuring presence of an unrelated atom
;; against the result lands below. That's the "discriminant" identity
;; for role-filler binding and residual encoding.


(:wat::test::deftest :wat-tests::holon::Subtract::test-self-presence-above-floor
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "alice"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "bob"))
     ((diff :wat::holon::HolonAST) (:wat::holon::Subtract a b)))
    (:wat::test::assert-eq (:wat::holon::presence? a diff) true)))

(:wat::test::deftest :wat-tests::holon::Subtract::test-unrelated-presence-below-floor
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "alice"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "bob"))
     ((c :wat::holon::HolonAST) (:wat::holon::Atom "charlie"))
     ((diff :wat::holon::HolonAST) (:wat::holon::Subtract a b)))
    (:wat::test::assert-eq (:wat::holon::presence? c diff) false)))
