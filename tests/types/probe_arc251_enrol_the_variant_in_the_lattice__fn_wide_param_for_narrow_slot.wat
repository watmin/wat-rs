;; Discriminator — fn(Alarm<Op>) stands in where fn(Alarm<Op.Mark>) is expected.
(:wat::core::defenum :u::Op :wat::enum::Pure
  :Mark [n <- :wat::core::i64]
  :Other [])
(:wat::core::defrecord :u::Alarm :- [O] [op <- :O])
(:wat::core::defn :u::apply
  [f <- [(:u::Alarm :- [:u::Op.Mark]) :-> :wat::core::nil]
   a <- (:u::Alarm :- [:u::Op.Mark])] -> :wat::core::nil
  (f a))
(:wat::core::defn :u::wide [a <- (:u::Alarm :- [:u::Op])] -> :wat::core::nil
  (:wat::kernel::println "ok"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:u::apply :u::wide (:u::Alarm :op (:u::Op.Mark {:n 1}))))
