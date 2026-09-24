;; Stone 255.16 — a type binds a parametric surface once (arc 255).
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Both [q <- :wat::core::i64])
;; LEGAL — DIFFERENT types bind the one surface differently (Th → Shared, Pr → Wire): the whole
;; point of the surface. `Both` is unused here.
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
(:wat::core::defn :probe::ss [loc <- (:probe::Loc :- [:probe::Shared])] -> :probe::Shared (:probe::Loc/transport loc))
(:wat::core::defn :probe::sc [loc <- (:probe::Loc :- [:probe::Wire])] -> :probe::Wire (:probe::Loc/transport loc))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::str (:probe::Shared/a (:probe::ss (:probe::Th :x 1)))))
    (:wat::kernel::println (:probe::Wire/b (:probe::sc (:probe::Pr :y 1))))))
