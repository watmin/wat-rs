;; Baseline — non-generic user define inside a deftest's prelude.

(:wat::test::deftest :wat-tests::tmp::baseline-nongeneric
  ((:wat::core::define
     (:test::make-pair (a :wat::core::i64) (b :wat::core::bool) -> :(wat::core::i64,wat::core::bool))
     (:wat::core::Tuple a b)))
  (:wat::core::let*
    (((pair :(wat::core::i64,wat::core::bool))
      (:test::make-pair 42 true))
     ((a :wat::core::i64) (:wat::core::first pair))
     ((b :wat::core::bool) (:wat::core::second pair))
     ((_ :wat::core::unit) (:wat::test::assert-eq a 42)))
    (:wat::test::assert-eq b true)))
