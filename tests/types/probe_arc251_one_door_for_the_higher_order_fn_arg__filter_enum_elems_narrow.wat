;; NEGATIVE — pred over Op.Mark refused for a Vector of Op (narrowing stays one-way).
;; filter's expected return is bool (not a fresh U), so the TypeMismatch has no :?N to drift.
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- :wat::core::i64]
  :Other [])
(:wat::core::defn :u::is-mark [o <- :u::Op.Mark] -> :wat::core::bool true)
(:wat::core::defn :u::go [xs <- (:wat::core::Vector :- [:u::Op])] -> (:wat::stream::Stream :- [:u::Op])
  (:wat::core::filter :u::is-mark xs))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
