(:wat::core::defmacro :probe::splice-i64
  [xs <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::quasiquote
    (:wat::core::Vector :wat::core::i64
      (:wat::core::unquote-splicing
        (:wat::core::let
          [doubled (:wat::core::map
                     (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                       (:wat::core::i64::* x 2))
                     xs)]
          doubled)))))

(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> (:probe::splice-i64 [1 2 3]))
