;; typed_if_match_else_branch_mismatch_bad.wat — else branch type mismatch. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::if true -> :wat::core::i64 1 "oops"))
