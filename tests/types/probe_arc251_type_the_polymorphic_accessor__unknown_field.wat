;; (:nosuchfield r) on a known receiver  REFUSE, located
(:wat::core::defrecord :u::Plain [x <- :wat::core::i64])
(:wat::core::defn :u::c [c <- :u::Plain] -> :wat::core::i64 (:nonexistent c))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
