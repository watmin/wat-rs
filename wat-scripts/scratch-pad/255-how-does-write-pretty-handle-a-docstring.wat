(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "-- write-pretty on a map with a MULTI-LINE string --")
    (:wat::kernel::println
      (:wat::edn::write-pretty
        (:wat::edn::read "{:doc \"line one\nline two\nline three\" :added \"1.0.0\"}")))
    (:wat::kernel::println "")))
