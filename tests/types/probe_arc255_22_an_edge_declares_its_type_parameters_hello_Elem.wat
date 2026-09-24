;; Stone 255.22 — an extend-type declares its type parameters (arc 255).
;; ⭐ The SAME program, parameter spelled Elem: checks, runs, prints. Before 255.22 it was refused (the edge was found by its parameter's spelling).
(:wat::core::defsurface :hello::Greets :- [Elem] :nature :wat::core::Struct
  :features [(greet [self <- (:hello::Greets :- [Elem])] -> :Elem)])
(:wat::core::defrecord :hello::Box :- [Elem] [v <- :Elem])
(:wat::core::extend-type :- [Elem] (:hello::Box :- [Elem]) (:hello::Greets :- [Elem])
  (greet [self] -> :Elem (:hello::Box/v self)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:hello::Greets/greet (:hello::Box :v "hello, world!"))))
