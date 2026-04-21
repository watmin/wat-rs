;; wat-tests/std/Sequential.wat — tests for wat/std/Sequential.wat.
;;
;; Sequential encoding (058-009) is STRICT identity: two lists with
;; the same items in different order produce vectors that are
;; orthogonal at the noise-floor level. This is the load-bearing
;; property of the bind-chain expansion (reframed 2026-04-18) — any
;; positional encoding that depends on order (trigrams, indicators
;; rhythms, the trading lab's rhythm.rs module) rests on this.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::test::deftest :wat-tests::std::Sequential::test-self-identity 1024 :error
  (:wat::core::let*
    (((a :holon::HolonAST) (:wat::algebra::Atom "a"))
     ((b :holon::HolonAST) (:wat::algebra::Atom "b"))
     ((c :holon::HolonAST) (:wat::algebra::Atom "c"))
     ((abc :holon::HolonAST)
      (:wat::std::Sequential (:wat::core::list :holon::HolonAST a b c))))
    (:wat::test::assert-eq (:wat::algebra::presence? abc abc) true)))

(:wat::test::deftest :wat-tests::std::Sequential::test-order-sensitivity 1024 :error
  (:wat::core::let*
    (((a :holon::HolonAST) (:wat::algebra::Atom "a"))
     ((b :holon::HolonAST) (:wat::algebra::Atom "b"))
     ((c :holon::HolonAST) (:wat::algebra::Atom "c"))
     ((abc :holon::HolonAST)
      (:wat::std::Sequential (:wat::core::list :holon::HolonAST a b c)))
     ((acb :holon::HolonAST)
      (:wat::std::Sequential (:wat::core::list :holon::HolonAST a c b))))
    (:wat::test::assert-eq (:wat::algebra::presence? abc acb) false)))
