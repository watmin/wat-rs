(:wat::core::defmacro :probe::splice-watast
  [xs <- :wat::WatAST]
  -> :wat::WatAST
  (:wat::core::quasiquote
    (:wat::core::Vector :wat::core::i64
      (:wat::core::unquote-splicing
        (:wat::core::let
          [forms (:wat::core::map
                   (:wat::core::fn [x <- :wat::core::i64] -> :wat::WatAST
                     (:wat::core::quasiquote
                       (:wat::core::unquote (:wat::core::i64::* x 10))))
                   xs)]
          forms)))))

(:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> (:probe::splice-watast [1 2 3]))
