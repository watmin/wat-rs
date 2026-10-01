;; CLAIM: bracket::map over a process pool of 2 returns [2 4 6 8 10].
(:wat::core::defn :my::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [nums (wat.type/Vector :- [wat.type/i64] 1 2 3 4 5)
     pr (:wat::bracket::map (:wat::spawn::process::runner-count 2) nums :my::double)]
    (:wat::core::do
      (:wat::test::assert-eq pr (wat.type/Vector :- [wat.type/i64] 2 4 6 8 10))
      (:wat::kernel::println (:wat::edn::write pr)))))
