;; T16: match arm with (:wat::core::Some n) pattern — n must not surface as free symbol.
(:wat::core::defn :my::option-or-zero [opt <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::match opt 
              ((:wat::core::Some n) n)
              (:wat::core::None    0)))
