(:wat::core::defn :probe::plain [n <- :wat::core::i64] -> :wat::core::i64 n)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [k     (:wat::keyword::from-string "probe::plain")   ;; runtime-computed keyword
     forms (:wat::kernel::fn-forms k :x)]                     ;; must resolve k → the plain fn, reify
    (:wat::kernel::println "fn-forms-keyword: ok")))
