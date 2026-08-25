;; wat_grep__with_tilde.wat — a TARGET file containing a reader macro (unquote via `~`), for G5:
;; Written count < Named count (the phantom class must actually be present).
(:wat::core::defmacro :user::wrap
  [x <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::do ~x))
