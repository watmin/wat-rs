(:wat::core::defn :user::c01 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-name
      (:wat::core::first
        (:wat::core::ast->children (:wat::core::read-string "<-"))))
    "<-"))
(:wat::core::defn :user::c02 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-kind
      (:wat::core::first
        (:wat::core::ast->children (:wat::core::read-string ":-"))))
    "keyword"))
(:wat::core::defn :user::c03 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-name (:wat::core::symbol-node "wat.core/map"))
    "wat.core/map"))
(:wat::core::defn :user::c04 [] -> :wat::core::bool
  (:wat::core::=
    (:wat::core::ast-name (:wat::core::keyword-node ":-"))
    ":-"))
