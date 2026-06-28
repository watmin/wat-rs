(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::fix::fix-macro-param-types ";; keep me byte-identical\n(:wat::core::defmacro :user::m [a <- :wat::holon::HolonAST & rest <- :AST<wat::holon::Holons>] -> :wat::holon::HolonAST a)\n(:wat::core::defn :user::f [x <- :wat::core::i64] -> :wat::core::i64 x)"))

