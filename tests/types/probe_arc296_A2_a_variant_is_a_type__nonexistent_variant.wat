;; ⛔ OVER-REACH DETECTOR — :usr::Box::Nope is not a variant of Box. P-1'"'"'s wall must still fire.
(:wat::core::defenum :usr::Box :- [T] :wat::enum::Pure
  :Full  [inside <- :T]
  :Empty [])
(:wat::core::defn :user::f [b <- (:usr::Box::Nope :- [:wat::core::i64])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
