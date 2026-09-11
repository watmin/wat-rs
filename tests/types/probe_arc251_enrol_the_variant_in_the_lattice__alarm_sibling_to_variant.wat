;; Alarm<Op> -> Alarm<Op.Mark> REFUSED (narrowing)
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- :wat::core::i64]
  :Other [])
(:wat::core::defrecord :u::Alarm :- [O] [op <- :O])
(:wat::core::defn :u::takes-mark [a <- (:u::Alarm :- [:u::Op.Mark])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :u::relay [a <- (:u::Alarm :- [:u::Op])] -> :wat::core::nil
  (:u::takes-mark a))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
