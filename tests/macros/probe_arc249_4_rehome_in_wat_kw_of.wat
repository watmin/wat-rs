(:wat::core::defmacro :test::kw-of
  [head <- :wat::holon::HolonAST & args <- (:AST :- [:wat::holon::Holons])]
  -> (:AST :- [:wat::holon::HolonAST])
  (:wat::core::let [head-text (:wat::keyword::name head)
                    arg-texts (:wat::core::map
                                (:wat::core::fn [a <- :wat::holon::HolonAST] -> :wat::core::String
                                   (:wat::keyword::name a))
                                args)
                    joined (:wat::string::join "," arg-texts)
                    full (:wat::string::concat head-text
                           (:wat::string::concat "<"
                             (:wat::string::concat joined ">")))]
    `~(:wat::keyword::from-name full)))
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::keyword::name (:test::kw-of :foo :bar :baz)))
