;; typed_if_match_if_wrong_arity_bad.wat — if with 6 args (one too many). Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::if true -> :wat::core::i64 1 2 99))
