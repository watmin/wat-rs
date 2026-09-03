(:wat::core::defn :my::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [nums (:wat::core::Vector :- [:wat::core::i64] 1 2 3 4 5)
     pr (:wat::bracket::map (:wat::spawn::process/runner-count 2) nums :my::double)]
    (:wat::kernel::println (:wat::edn::write pr))))
