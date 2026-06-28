;; probe_arc256_generic_defclause_c03_bad.wat — T:=i64 then String mismatches. Must FAIL.

(:wat::core::defclause :user::firstof ([a <- :T b <- :T] -> :T a))
(:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::firstof 1 "two"))
