;; T17: match wildcard _ does not surface as free symbol.
(:wat::core::defn :my::is-some? [opt <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::core::match opt 
              ((:wat::core::Some _) true)
              (:wat::core::None     false)))
