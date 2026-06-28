;; typed_if_match_match_no_type_kw_bad.wat — match without type keyword after arrow. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::match (:wat::core::Some 1) -> oops ((:wat::core::Some v) v) (:wat::core::None 0)))
