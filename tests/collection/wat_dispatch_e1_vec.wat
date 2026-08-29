;; tests/collection/wat_dispatch_e1_vec.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Distinct :my::compute-* defns for each test.

(:wat::core::use! :rust::test::VecUtils)

(:wat::core::defn :my::compute-sum [] -> :wat::core::i64
  (:rust::test::VecUtils::sum (:wat::core::Vector :- [:wat::core::i64] 10 20 30)))

(:wat::core::defn :my::compute-reverse [] -> :wat::core::i64
  (:wat::core::first
    (:rust::test::VecUtils::reverse (:wat::core::Vector :- [:wat::core::i64] 1 2 3))))

(:wat::core::defn :my::compute-sort [] -> :wat::core::i64
  (:wat::core::first
    (:rust::test::VecUtils::sort (:wat::core::Vector :- [:wat::core::i64] 5 2 8 1))))

(:wat::core::defn :my::compute-empty [] -> :wat::core::i64
  (:rust::test::VecUtils::sum (:wat::core::Vector :- [:wat::core::i64])))

