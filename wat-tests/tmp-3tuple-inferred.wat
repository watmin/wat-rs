;; Generic-T 3-tuple — call site WITHOUT explicit <T> turbofish.
;; Tests whether check infers T at the call site.

(:wat::test::deftest :wat-tests::tmp::generic-3tuple-inferred
  ((:wat::core::define
     (:test::make-3tuple<T> (mid :T) -> :(wat::core::i64,T,wat::core::String))
     (:wat::core::Tuple 42 mid "hello")))
  (:wat::core::let*
    (((triple :(wat::core::i64,wat::core::bool,wat::core::String))
      (:test::make-3tuple true))
     ((a :wat::core::i64) (:wat::core::first triple))
     ((b :wat::core::bool) (:wat::core::second triple))
     ((c :wat::core::String) (:wat::core::third triple))
     ((_ :wat::core::unit) (:wat::test::assert-eq a 42))
     ((_ :wat::core::unit) (:wat::test::assert-eq b true)))
    (:wat::test::assert-eq c "hello")))
