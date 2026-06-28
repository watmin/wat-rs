(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))
(:wat::core::defn :user::c01 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::map f xs)"))))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::if true -> :wat::core::i64 1 2)"))))
(:wat::core::defn :user::c03 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::do (:wat::core::first xs))"))))
(:wat::core::defn :user::c04 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:else 1)"))))
