;; scout-eval-6: does eval-ast! of a DEFN-form install a callable by name?
(:wat::core::defn :user::uf
  [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [defn-form (:user::uf "(:wat::core::defn :probe::pred [n <- :wat::core::i64] -> :wat::core::bool (:wat::core::> n 3))")
     res       (:wat::eval-ast! defn-form)]
    (:wat::kernel::println (:wat::core::str res))))
