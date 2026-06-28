;; typed_if_match_non_bool_cond_bad.wat — if with non-bool condition. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::if 42 -> :wat::core::i64 1 2))
