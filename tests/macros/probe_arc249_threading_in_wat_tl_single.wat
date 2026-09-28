(:wat::core::defmacro :test::thread-last
  [acc <- wat.type/AST & steps <- (wat.type/Vector :- [wat.type/AST])]
  -> wat.type/AST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST `(~@step ~a))
    acc
    steps))
;; Arc 118.2a — `map` flipped LAZY; materialize via `mapv` for the equality check.
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::= (:test::thread-last [1 2 3]
                   (:wat::core::mapv (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::+ x 1))))
                 [2 3 4]))
