;; typed_if_match_match_too_few_bad.wat — match with no arms. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::match (:wat::core::Some 1) -> :wat::core::i64))
