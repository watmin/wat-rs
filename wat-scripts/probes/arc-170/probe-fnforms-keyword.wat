(:wat::core::defn :probe::plain [n <- wat.type/i64] -> wat.type/i64 n)
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [k     (:wat::keyword::from-string "probe::plain")   ;; runtime-computed keyword
     forms (:wat::kernel::fn-forms k :x)]                     ;; must resolve k → the plain fn, reify
    (:wat::kernel::println "fn-forms-keyword: ok")))
