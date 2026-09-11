;; Vector of Op.Mark, filtered by a pred over Op. REFUSED at step 1, ACCEPTED at step 2.
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- :wat::core::i64]
  :Other [])
(:wat::core::defn :u::is-op [o <- :u::Op] -> :wat::core::bool true)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [_ (:wat::core::filter :u::is-op [(:u::Op.Mark {:n 1})])]
    (:wat::kernel::println "ok")))
