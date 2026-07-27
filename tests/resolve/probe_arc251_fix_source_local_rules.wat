(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))))
(:wat::core::defn :user::c01 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "[x <- y]"))))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "[x <- :wat::core::i64]"))))
(:wat::core::defn :user::c03 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "[x <- :wat::core::Vector<wat::core::i64>]"))))
(:wat::core::defn :user::c04 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::map f xs)"))))
(:wat::core::defn :user::c05 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::fn [a <- :wat::core::i64] -> :wat::core::bool a)"))))
(:wat::core::defn :user::c06a [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::< a b)"))))
(:wat::core::defn :user::c06b [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::<= a b)"))))
(:wat::core::defn :user::c07 [] -> :wat::core::String
  (:wat::core::write-forms (:wat::fix::fix-source (:user::topform "(:wat::core::> a b)"))))
