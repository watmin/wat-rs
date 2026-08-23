;; Does a generic fn substitute its type param I into a `forms` quote when called concretely?
(:wat::core::defn :probe::mk :- [I]
  [x <- :I] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::forms
    (:wat::core::defn :probe::inner [a <- :I] -> :I a)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::edn::write (:probe::mk 5))))
