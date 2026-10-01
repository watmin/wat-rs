;; Stone 255.25a — a variant is a type the moment its enum is (arc 255).
;; CONTROL — the same edge with the marker an empty defstruct: checks, runs, prints (both binaries).
(:wat::core::defstruct :probe::Wi [])
(:wat::core::defsurface :probe::Greets :- [T] :nature wat.type/Struct
  :features [(greet [self <- (:probe::Greets :- [T])] -> wat.type/String)])
(:wat::core::defstruct :probe::Box [])
(:wat::core::extend-type :probe::Box (:probe::Greets :- [:probe::Wi])
  (greet [self] -> wat.type/String "hello"))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::kernel::println (:probe::Greets/greet (:probe::Box))))
