;; wat-tests/holon/Sequential.wat — tests for wat/holon/Sequential.wat.
;;
;; Sequential encoding (058-009) is STRICT identity: two lists with
;; the same items in different order produce vectors that are
;; orthogonal at the noise-floor level. This is the load-bearing
;; property of the bind-chain expansion (reframed 2026-04-18) — any
;; positional encoding that depends on order (trigrams, indicators
;; rhythms, the trading lab's rhythm.rs module) rests on this.


(:wat::test::deftest :wat-tests::holon::Sequential::test-self-identity
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     abc
      (:wat::holon::Sequential (:wat::core::Vector :- [:wat::holon::HolonAST] a b c))]
    (:wat::test::assert-eq (:wat::holon::presence? abc abc) true)))

(:wat::test::deftest :wat-tests::holon::Sequential::test-order-sensitivity
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     abc
      (:wat::holon::Sequential (:wat::core::Vector :- [:wat::holon::HolonAST] a b c))
     acb
      (:wat::holon::Sequential (:wat::core::Vector :- [:wat::holon::HolonAST] a c b))]
    (:wat::test::assert-eq (:wat::holon::presence? abc acb) false)))
