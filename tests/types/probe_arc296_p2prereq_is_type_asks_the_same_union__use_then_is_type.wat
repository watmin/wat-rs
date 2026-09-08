;; THE SUBJECT — a use!'d foreign type IS a type. is-type? must say so.
(:wat::core::use! :rust::sqlite::Connection)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :rust::sqlite::Connection)))
