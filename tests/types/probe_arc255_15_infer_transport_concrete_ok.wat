;; Stone 255.15 — infer a type variable from the implementor's extend-type binding.
;; A parametric surface `Loc :- [T]` (the locus, T = its transport) and two implementors that
;; bind DIFFERENT transports: `Th` binds `Shared`, `Pr` binds `Wire`.
(:wat::core::defrecord :probe::Shared [a <- wat.type/i64])
(:wat::core::defrecord :probe::Wire [b <- wat.type/String])
(:wat::core::defsurface :probe::Loc :- [T] :nature wat.type/Struct
  :features [(transport [self <- (:probe::Loc :- [T])] -> :T)])
(:wat::core::defrecord :probe::Th [x <- wat.type/i64])
(:wat::core::extend-type :probe::Th (:probe::Loc :- [:probe::Shared]) (transport [self] (:probe::Shared :a 1)))
(:wat::core::defrecord :probe::Pr [y <- wat.type/i64])
(:wat::core::extend-type :probe::Pr (:probe::Loc :- [:probe::Wire]) (transport [self] (:probe::Wire :b "w")))
;; CONTROL (unchanged): the CONCRETE parameter accepts the implementor that binds it.
(:wat::core::defn :probe::start-concrete [loc <- (:probe::Loc :- [:probe::Shared])] -> :probe::Shared (:probe::Loc/transport loc))
(:wat::core::defn :probe::use [] -> :probe::Shared (:probe::start-concrete (:probe::Th :x 1)))
