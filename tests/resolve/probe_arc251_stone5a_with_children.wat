(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::not
    (:wat::core::List?
      (:wat::core::with-children
        (:wat::core::first
          (:wat::core::ast->children
            (:wat::core::read-string "[a b]")))
        (:wat::core::ast->children
          (:wat::core::first
            (:wat::core::ast->children
              (:wat::core::read-string "[a b]"))))))))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::with-children
      (:wat::core::first
        (:wat::core::ast->children
          (:wat::core::read-string "(a b)")))
      (:wat::core::ast->children
        (:wat::core::first
          (:wat::core::ast->children
            (:wat::core::read-string "(a b)")))))))
