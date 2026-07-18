(:wat::core::defmacro :user::mk [base <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let
    [base-str (:wat::core::ast-name base)
     full     (:wat::core::string::interpolate "{b}::built" :b base-str)]
    (:wat::core::first (:wat::core::ast->children
      (:wat::core::read-string (:wat::core::string::concat "\"" (:wat::core::string::concat full "\"")))))))
(:wat::core::defn :user::probe [] -> :wat::core::String (:user::mk hello))
(:wat::core::defn :user::runtime-interp [] -> :wat::core::String
  (:wat::core::string::interpolate "{a}::{b} {{lit}}" :a "x" :b 5))
