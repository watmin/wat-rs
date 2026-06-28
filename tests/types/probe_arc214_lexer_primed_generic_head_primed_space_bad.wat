;; Negative fixture: primed generic head with space inside <...> — must lex-error (unclosed bracket, not CommaInKeywordBody).
(:wat::core::defn :user::take [m <- :wat::kernel::Thread'<wat::core::i64, wat::core::i64>] -> :wat::core::nil nil)
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
