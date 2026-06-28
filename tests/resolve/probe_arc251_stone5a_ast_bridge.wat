(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::read-string "((:a 1) (:b 2))")))))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::List?
    (:wat::core::first
      (:wat::core::ast->children
        (:wat::core::first
          (:wat::core::ast->children
            (:wat::core::read-string "((:a 1) (:b 2))")))))))
