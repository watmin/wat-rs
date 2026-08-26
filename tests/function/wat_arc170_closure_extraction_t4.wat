;; T4: inline lambda, no captures (factory pattern returning fn).
(:wat::core::defn :my::factory [] -> :wat::core::Fn(wat::core::i64)->wat::core::i64
  (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64
              (:wat::i64::+ n 7)))
