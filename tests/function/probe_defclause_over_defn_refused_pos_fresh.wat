;; THE ORDINARY CASE the wall must not touch — 71 of these in the corpus.
(:wat::core::defclause :user::twice ([n <- :wat::core::i64] -> :wat::core::i64 (:wat::core::* n 2)))
(:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::twice 21))
