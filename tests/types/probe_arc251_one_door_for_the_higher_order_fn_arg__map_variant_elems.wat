;; Vector of Op.Mark, mapped by a fn over Op. REFUSED at step 1, ACCEPTED at step 2.
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- :wat::core::i64]
  :Other [])
(:wat::core::defn :u::takes-op [o <- :u::Op] -> :wat::core::i64 1)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:wat::core::map :u::takes-op [(:u::Op.Mark {:n 1})])]
    (:wat::kernel::println "ok")))
