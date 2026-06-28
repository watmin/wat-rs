;; T1: top-level defn, no deps, no captures.
(:wat::core::defn :my::add-one [n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ n 1))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
