(:wat::core::defn :user::end-col [] -> :wat::core::i64
  (:wat::core::let
    [tree (:wat::core::read-string "(a b c)")
     form (:wat::core::first (:wat::core::ast->children tree))
     espan (:wat::core::ast-end-span form)]
    (:wat::core::Option/expect
      (:wat::core::HashMap/get espan :col)
      "end :col")))
