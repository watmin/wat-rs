(:wat::core::defmacro :my::sum [& nums <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::holon::HolonAST n <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST `(:wat::core::i64::+ ~acc ~n))
    `0
    nums))
(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::= (:my::sum 1 2 3) 6))
