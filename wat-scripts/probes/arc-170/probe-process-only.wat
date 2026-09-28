(:wat::core::defn :my::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [nums (wat.type/Vector :- [wat.type/i64] 1 2 3 4 5)
     pr (:wat::bracket::map (:wat::spawn::process::runner-count 2) nums :my::double)]
    (:wat::kernel::println (:wat::edn::write pr))))
