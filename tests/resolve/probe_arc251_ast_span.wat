(:wat::core::defn :user::c01 [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::core::HashMap/get
      (:wat::core::ast-span
        (:wat::core::first (:wat::core::ast->children
          (:wat::core::first (:wat::core::ast->children
            (:wat::core::read-string "(:wat::core::map x)"))))))
      :line)
    "field"))
(:wat::core::defn :user::c02 [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::core::HashMap/get
      (:wat::core::ast-span
        (:wat::core::first (:wat::core::ast->children
          (:wat::core::first (:wat::core::ast->children
            (:wat::core::read-string "(:wat::core::map x)"))))))
      :col)
    "field"))
(:wat::core::defn :user::c03 [] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::core::HashMap/get
      (:wat::core::ast-span
        (:wat::core::first (:wat::core::rest (:wat::core::ast->children
          (:wat::core::first (:wat::core::ast->children
            (:wat::core::read-string "(:wat::core::map x)")))))))
      :col)
    "field"))
