;; T18: match Result patterns — Ok-arm binds b, Err-arm has wildcard.
(:wat::core::defn :my::unwrap-or-false [r <- (:wat::core::Result :- [wat.type/bool wat.type/String])] -> wat.type/bool
  (:wat::core::match r 
              [:wat::core::Result.Ok {:value b}  b]
              [:wat::core::Result.Err {:error _} false]))
