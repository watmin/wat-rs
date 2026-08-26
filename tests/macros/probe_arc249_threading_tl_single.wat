;; Arc 118.2a — `map` flipped LAZY; materialize via `mapv` for the equality check.
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:wat::core::->> [1 2 3] (:wat::core::mapv (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ x 1)))) [2 3 4]))
