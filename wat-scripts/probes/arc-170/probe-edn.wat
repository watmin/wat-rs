(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:wat::edn::write (wat.type/Vector :- [wat.type/i64] 2 4 6))))
