;; typed_if_match_if_false.wat — typed if returns else-branch on false.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::if false -> :wat::core::i64 11 22))
