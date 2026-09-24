;; Stone 255.17 — the last type argument of a parametric annotation is not forgotten (arc 255).
;; ACCEPTED — the correct twins: every field returned at its OWN type variable, then run.
(:wat::core::defrecord :probe::R :- [X Y Z] [x <- :X y <- :Y z <- :Z])
(:wat::core::defenum :probe::E :- [X Y] :wat::enum::Pure :A [v <- :X] :B [w <- :Y])
(:wat::core::defn :probe::rz :- [A B C] [r <- (:probe::R :- [A B C])] -> :C (:probe::R/z r))
(:wat::core::defn :probe::eb :- [T S] [e <- (:probe::E :- [T S]) d <- :S] -> :S
  (:wat::core::match e [:probe::E.B {:w v} v] [_ d]))
(:wat::core::defn :probe::re :- [T E] [o <- (:wat::core::Result :- [T E]) d <- :E] -> :E
  (:wat::core::match o [:wat::core::Result.Err {:error v} v] [_ d]))
(:wat::core::defn :probe::os :- [T] [o <- (:wat::core::Option :- [T]) d <- :T] -> :T
  (:wat::core::match o [:wat::core::Option.Some {:value v} v] [_ d]))
(:wat::core::defn :probe::same :- [T] [r <- (:probe::R :- [T T T])] -> (:probe::R :- [T T T]) r)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:probe::rz (:probe::R :x 1 :y "y" :z "z")))
    (:wat::kernel::println (:probe::eb (:probe::E.B {:w "b"}) "d"))
    (:wat::kernel::println (:probe::re (:wat::core::Result.Err {:error "e"}) "d"))
    (:wat::kernel::println (:probe::os (:wat::core::Option.Some {:value "o"}) "d"))
    (:wat::kernel::println (:probe::R/z (:probe::same (:probe::R :x "1" :y "2" :z "3"))))))
