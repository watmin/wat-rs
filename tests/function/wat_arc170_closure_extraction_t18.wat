;; T18: match Result patterns — Ok-arm binds b, Err-arm has wildcard.
(:wat::core::defn :my::unwrap-or-false [r <- (:wat::core::Result :- [:wat::core::bool :wat::core::String])] -> :wat::core::bool
  (:wat::core::match r 
              ((:wat::core::Ok b)  b)
              ((:wat::core::Err _) false)))
