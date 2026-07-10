;; Negative fixture for wat_dispatch_193a.rs — startup must fail (type error).
;; :my::probe passes i64 literal 42 where :wat::core::u8 is expected → type check fires.

(:wat::core::use! :rust::test::MathUtils)


(:wat::core::defn :my::probe [] -> :wat::core::i64
  (:rust::test::MathUtils::add "not-an-int" 2))
