;; SUBJECT — a variant name in annotation position. Refused today (UnknownNamedType).
(:wat::core::defenum :usr::Colour :wat::enum::Pure
  :Red  [shade <- :wat::core::i64]
  :Blue [shade <- :wat::core::i64])
(:wat::core::defn :user::f [c <- :usr::Colour.Red] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
