;; SUBJECT — a variant name in annotation position. Refused today (UnknownNamedType).
(:wat::core::defenum :usr::Colour :wat::enum::Pure
  :Red  [shade <- wat.type/i64]
  :Blue [shade <- wat.type/i64])
(:wat::core::defn :user::f [c <- :usr::Colour.Red] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
