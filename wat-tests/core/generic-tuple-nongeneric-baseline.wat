;; Baseline — non-generic user define inside a deftest's prelude.

(:wat::core::defn :test::make-pair [a <- wat.type/i64 b <- wat.type/bool] -> (wat.type/Tuple :- [wat.type/i64 wat.type/bool]) (:wat::core::Tuple a b))

(:wat::test::deftest :wat-tests::core::generic-tuple-nongeneric-baseline
  
  (:wat::core::let
    [pair
      (:test::make-pair 42 true)
     a (:wat::core::first pair)
     b (:wat::core::second pair)
     _ (:wat::test::assert-eq a 42)]
    (:wat::test::assert-eq b true)))
