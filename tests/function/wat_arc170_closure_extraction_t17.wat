;; T17: match wildcard _ does not surface as free symbol.
(:wat::core::defn :my::is-some? [opt <- (:wat::core::Option :- [wat.type/i64])] -> wat.type/bool
  (:wat::core::match opt 
              [:wat::core::Option.Some {:value _} true]
              [:wat::core::Option.None {}     false]))
