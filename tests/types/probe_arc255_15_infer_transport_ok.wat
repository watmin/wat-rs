;; Stone 255.15 — infer a type variable from the implementor's extend-type binding.
;; A parametric surface `Loc :- [T]` (the locus, T = its transport) and two implementors that
;; bind DIFFERENT transports: `Th` binds `Shared`, `Pr` binds `Wire`.
(:wat::core::defrecord :probe::Shared [a <- :wat::core::i64])
(:wat::core::defrecord :probe::Wire [b <- :wat::core::String])
(:wat::core::defsurface :probe::Loc :- [T] :nature :wat::core::Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Th [x <- :wat::core::i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- :wat::core::i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
;; POSITIVE: ONE generic start; T binds from whichever implementor it is given.
(:wat::core::defn :probe::start :- [T] [loc <- (:probe::Loc :- [T])] -> :T (:probe::Loc/transport loc))
;; Two parameters sharing one T, both given implementors that bind the SAME transport.
(:wat::core::defn :probe::two :- [T] [a <- (:probe::Loc :- [T]) b <- (:probe::Loc :- [T])] -> :T
  (:probe::Loc/transport a))
(:wat::core::defn :probe::use-th [] -> :probe::Shared (:probe::start (:probe::Th :x 1)))
(:wat::core::defn :probe::use-pr [] -> :probe::Wire (:probe::start (:probe::Pr :y 2)))
(:wat::core::defn :probe::use-two [] -> :probe::Shared (:probe::two (:probe::Th :x 4) (:probe::Th :x 5)))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:wat::core::str (:probe::Shared/a (:probe::use-th))))
    (:wat::kernel::println (:probe::Wire/b (:probe::use-pr)))
    (:wat::kernel::println (:wat::core::str (:probe::Shared/a (:probe::use-two))))))
