;; Stone 255.22 — an extend-type declares its type parameters (arc 255).
;; The builder's hello-world, parameter spelled T: checks, runs, prints.
(:wat::core::defsurface :hello::Greets :- [T] :nature :wat::core::Struct
  :features [(greet [self <- (:hello::Greets :- [T])] -> :T)])
(:wat::core::defrecord :hello::Box :- [T] [v <- :T])
(:wat::core::extend-type :- [T] (:hello::Box :- [T]) (:hello::Greets :- [T])
  (greet [self] -> :T (:hello::Box/v self)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:hello::Greets/greet (:hello::Box :v "hello, world!"))))
