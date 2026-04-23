;; wat-tests/holon/Sequential.wat — tests for wat/holon/Sequential.wat.
;;
;; Sequential encoding (058-009) is STRICT identity: two lists with
;; the same items in different order produce vectors that are
;; orthogonal at the noise-floor level. This is the load-bearing
;; property of the bind-chain expansion (reframed 2026-04-18) — any
;; positional encoding that depends on order (trigrams, indicators
;; rhythms, the trading lab's rhythm.rs module) rests on this.

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::deftest :wat-tests::holon::Sequential::test-self-identity
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "a"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "b"))
     ((c :wat::holon::HolonAST) (:wat::holon::Atom "c"))
     ((abc :wat::holon::HolonAST)
      (:wat::holon::Sequential (:wat::core::list :wat::holon::HolonAST a b c))))
    (:wat::test::assert-eq (:wat::holon::presence? abc abc) true)))

(:wat::test::deftest :wat-tests::holon::Sequential::test-order-sensitivity
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "a"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "b"))
     ((c :wat::holon::HolonAST) (:wat::holon::Atom "c"))
     ((abc :wat::holon::HolonAST)
      (:wat::holon::Sequential (:wat::core::list :wat::holon::HolonAST a b c)))
     ((acb :wat::holon::HolonAST)
      (:wat::holon::Sequential (:wat::core::list :wat::holon::HolonAST a c b))))
    (:wat::test::assert-eq (:wat::holon::presence? abc acb) false)))
