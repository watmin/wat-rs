;; Stone 255.25a — a variant is a type the moment its enum is (arc 255).
;; CONTROL — the same edge with the marker an empty defstruct: checks, runs, prints (both binaries).
(:wat::core::defstruct :probe::Wi [])
(:wat::core::defsurface :probe::Greets :- [T] :nature :wat::core::Struct
  :features [(greet [self <- (:probe::Greets :- [T])] -> :wat::core::String)])
(:wat::core::defstruct :probe::Box [])
(:wat::core::extend-type :probe::Box (:probe::Greets :- [:probe::Wi])
  (greet [self] -> :wat::core::String "hello"))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::Greets/greet (:probe::Box))))
