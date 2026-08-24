;; Generic-T 3-tuple — call site WITHOUT explicit turbofish.
;; Tests whether check infers T at the call site.

(:wat::core::defn :test::make-3tuple :- [T] [mid <- :T] -> (:wat::core::Tuple :- [:wat::core::i64 T :wat::core::String]) (:wat::core::Tuple 42 mid "hello"))

(:wat::test::deftest :wat-tests::core::generic-tuple-infer
  
  (:wat::core::let
    [triple
      (:test::make-3tuple true)
     a (:wat::core::first triple)
     b (:wat::core::second triple)
     c (:wat::core::third triple)
     _ (:wat::test::assert-eq a 42)
     _ (:wat::test::assert-eq b true)]
    (:wat::test::assert-eq c "hello")))
