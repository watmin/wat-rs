;; Fixture: primed two-param generic head — must pass the lexer (CommaInKeywordBody must NOT fire).
(:wat::core::defn :user::take [m <- :wat::kernel::Thread'<wat::core::i64,wat::core::i64>] -> :wat::core::nil nil)
