(:wat::core::defmacro :test::thread-last
  [acc <- :wat::WatAST & steps <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST `(~@step ~a))
    acc
    steps))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:test::thread-last [1 2 3]
                   (:wat::core::map (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ x 1)))
                   (:wat::core::filter (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 2))))
                 [3 4]))
