(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:wat::edn::write (:wat::deporder::verify-stdlib))))
