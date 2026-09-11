;; Vector of Op.Mark, folded by a fn over Op. Already ACCEPTED (foldl uses assignable).
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- :wat::core::i64]
  :Other [])
(:wat::core::defn :u::acc [a <- :wat::core::i64 o <- :u::Op] -> :wat::core::i64 a)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:wat::core::foldl :u::acc 0 [(:u::Op.Mark {:n 1})])]
    (:wat::kernel::println "ok")))
