(:wat::core::defmacro :test::kw-of
  [head <- :wat::holon::HolonAST & args <- (:AST :- [:wat::holon::Holons])]
  -> (:AST :- [:wat::holon::HolonAST])
  (:wat::core::let [head-text (:wat::core::keyword/to-string head)
                    arg-texts (:wat::core::map
                                (:wat::core::fn [a <- :wat::holon::HolonAST] -> :wat::core::String
                                   (:wat::core::keyword/to-string a))
                                args)
                    joined (:wat::string::join "," arg-texts)
                    full (:wat::string::concat head-text
                           (:wat::string::concat "<"
                             (:wat::string::concat joined ">")))]
    `~(:wat::core::keyword/from-string full)))
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::keyword/to-string (:test::kw-of :foo :bar :baz)))
