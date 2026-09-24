;; Stone 255.16 — a type binds a parametric surface once (arc 255).
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Both [q <- :wat::core::i64])
;; LEGAL — the IDENTICAL binding re-registered (bodied, then bodiless) is not a second binding.
(:wat::core::extend-type :probe::Both (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
(:wat::core::extend-type :probe::Both (:probe::Loc :- [:probe::Wire]))
(:wat::core::defn :probe::sc [loc <- (:probe::Loc :- [:probe::Wire])] -> :probe::Wire (:probe::Loc/transport loc))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:probe::Wire/b (:probe::sc (:probe::Both :q 1)))))
