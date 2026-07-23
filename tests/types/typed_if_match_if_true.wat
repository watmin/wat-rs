;; typed_if_match_if_true.wat — typed if returns then-branch on true.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::if true  11 22))
