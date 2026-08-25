(:wat::core::defmacro :user::mk [base <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let
    [base-str (:wat::core::ast-name base)
     full     (:wat::string::interpolate "{b}::built" :b base-str)]
    (:wat::core::first (:wat::core::ast->children
      (:wat::core::match (:wat::core::read-string (:wat::string::concat "\"" (:wat::string::concat full "\""))) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::core::macro-error (:wat::string::concat "expand-time read-string failed: " (:wat::core::Error/message __cause)))))))))
(:wat::core::defn :user::probe [] -> :wat::core::String (:user::mk hello))
(:wat::core::defn :user::runtime-interp [] -> :wat::core::String
  (:wat::string::interpolate "{a}::{b} {{lit}}" :a "x" :b 5))
