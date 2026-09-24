;; Stone 255.22 — an extend-type declares its type parameters (arc 255).
;; The surface spells its parameter T, the edge spells it E: the method's result is instantiated by matching the EDGE's child, so it is String.
(:wat::core::defsurface :hello::Greets :- [T] :nature :wat::core::Struct
  :features [(greet [self <- (:hello::Greets :- [T])] -> :T)])
(:wat::core::defrecord :hello::Box :- [Elem] [v <- :Elem])
(:wat::core::extend-type :- [E] (:hello::Box :- [E]) (:hello::Greets :- [E])
  (greet [self] -> :E (:hello::Box/v self)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:hello::Greets/greet (:hello::Box :v "hello, world!"))))
