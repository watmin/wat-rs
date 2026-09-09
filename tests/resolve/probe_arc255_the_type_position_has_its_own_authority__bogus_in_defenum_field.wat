;; RED — a bogus type name in a defenum VARIANT FIELD annotation.
(:wat::core::defenum :user::E :wat::enum::Pure
  :V [f <- (wat.type/Vector :- [wat.type/Bogus])])
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "x"))
