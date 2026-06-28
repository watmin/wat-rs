(:wat::core::defn :user::probe [] -> :wat::core::String
  (:wat::fix::fix-text ";; this doc comment must survive byte-identical\n(:wat::core::if true -> :wat::core::i64 1 2)"))

(:wat::core::defn :user::once [] -> :wat::core::String
  (:wat::fix::fix-text ";; alpha comment\n;; beta comment\n\n(:wat::core::if true -> :wat::core::i64 1 2)\n;; gamma trailing"))

(:wat::core::defn :user::twice [] -> :wat::core::String
  (:wat::fix::fix-text (:user::once)))

