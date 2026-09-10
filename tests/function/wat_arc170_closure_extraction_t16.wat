;; T16: match arm with (:wat::core::Some n) pattern — n must not surface as free symbol.
(:wat::core::defn :my::option-or-zero [opt <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match opt 
              [:wat::core::Option.Some {:value n} n]
              [:wat::core::Option.None {}    0]))
