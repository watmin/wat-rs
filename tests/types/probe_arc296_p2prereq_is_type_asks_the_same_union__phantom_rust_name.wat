;; CONTROL — a :rust:: name nothing knows. FALSE now, must stay FALSE.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :rust::totally::MadeUp)))
