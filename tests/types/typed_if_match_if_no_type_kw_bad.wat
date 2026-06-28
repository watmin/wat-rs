;; typed_if_match_if_no_type_kw_bad.wat — if without type keyword after arrow. Must FAIL.
(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::if true -> 1 2 3))
