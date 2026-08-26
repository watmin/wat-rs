(:wat::core::defmacro :test::thread-first
  [acc <- :wat::WatAST & steps <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) 
         `(~(:wat::core::first step) ~a ~@(:wat::core::rest step))
         `(~step ~a)))
    acc
    steps))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::= (:test::thread-first 5 (:wat::i64::- 3)) 2))
