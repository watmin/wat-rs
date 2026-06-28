;; probe_arc256_generic_defclause_c02.wat — (firstof 1 2) → T:=i64. RED at HEAD.

(:wat::core::defclause :user::firstof ([a <- :T b <- :T] -> :T a))
(:wat::core::defn :user::probe [] -> :wat::core::i64 (:user::firstof 1 2))
