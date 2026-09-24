;; Stone 255.25a — a variant is a type the moment its enum is (arc 255).
;; ROW — a Pure enum's variant in an extend-type TARGET: checks, runs, prints. Pre-stone: EdgeFreeTypeName.
(:wat::core::defenum :probe::Tr :wat::enum::Pure
  :Sh []
  :Wi [])
(:wat::core::defsurface :probe::Greets :- [T] :nature :wat::core::Struct
  :features [(greet [self <- (:probe::Greets :- [T])] -> :wat::core::String)])
(:wat::core::defstruct :probe::Box [])
(:wat::core::extend-type :probe::Box (:probe::Greets :- [:probe::Tr.Wi])
  (greet [self] -> :wat::core::String "hello"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::Greets/greet (:probe::Box))))
