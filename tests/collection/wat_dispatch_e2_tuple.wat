;; tests/collection/wat_dispatch_e2_tuple.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Distinct :my::compute-* defns for each test.

(:wat::core::use! :rust::test::TupleUtils)

(:wat::core::defn :my::compute-sum2 [] -> :wat::core::i64
  (:rust::test::TupleUtils::sum2 (:wat::core::Tuple 20 22)))

(:wat::core::defn :my::compute-pair-first [] -> :wat::core::i64
  (:wat::core::first (:rust::test::TupleUtils::pair_of 7 13)))

(:wat::core::defn :my::compute-describe [] -> :wat::core::String
  (:rust::test::TupleUtils::describe
    (:wat::core::Tuple 1 "row" true)))

