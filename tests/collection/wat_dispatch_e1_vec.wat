;; tests/collection/wat_dispatch_e1_vec.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Distinct :my::compute-* defns for each test.

(:wat::core::use! :rust::test::VecUtils)

(:wat::core::defn :my::compute-sum [] -> wat.type/i64
  (:rust::test::VecUtils/sum (wat.type/Vector :- [wat.type/i64] 10 20 30)))

(:wat::core::defn :my::compute-reverse [] -> wat.type/i64
  (:wat::core::first
    (:rust::test::VecUtils/reverse (wat.type/Vector :- [wat.type/i64] 1 2 3))))

(:wat::core::defn :my::compute-sort [] -> wat.type/i64
  (:wat::core::first
    (:rust::test::VecUtils/sort (wat.type/Vector :- [wat.type/i64] 5 2 8 1))))

(:wat::core::defn :my::compute-empty [] -> wat.type/i64
  (:rust::test::VecUtils/sum (wat.type/Vector :- [wat.type/i64])))

