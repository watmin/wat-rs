;; Alarm<Op> -> Alarm<Op.Mark> REFUSED (narrowing)
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- wat.type/i64]
  :Other [])
(:wat::core::defrecord :u::Alarm :- [O] [op <- :O])
(:wat::core::defn :u::takes-mark [a <- (:u::Alarm :- [:u::Op.Mark])] -> wat.type/nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :u::relay [a <- (:u::Alarm :- [:u::Op])] -> wat.type/nil
  (:u::takes-mark a))
(:wat::core::defn :user::main [] -> wat.type/nil (:wat::kernel::println "ok"))
