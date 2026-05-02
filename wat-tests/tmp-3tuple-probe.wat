;; Minimal reproduction — does generic-T 3-tuple return work?

(:wat::core::define
  (:test::make-3tuple<T> (mid :T) -> :(wat::core::i64,T,wat::core::String))
  (:wat::core::Tuple 42 mid "hello"))

(:wat::test::deftest :wat-tests::tmp::generic-3tuple-roundtrip
  ()
  (:wat::core::let*
    (((triple :(wat::core::i64,wat::core::bool,wat::core::String))
      (:test::make-3tuple<wat::core::bool> true))
     ((a :wat::core::i64) (:wat::core::first triple))
     ((b :wat::core::bool) (:wat::core::second triple))
     ((c :wat::core::String) (:wat::core::third triple))
     ((_ :wat::core::unit) (:wat::test::assert-eq a 42))
     ((_ :wat::core::unit) (:wat::test::assert-eq b true)))
    (:wat::test::assert-eq c "hello")))
