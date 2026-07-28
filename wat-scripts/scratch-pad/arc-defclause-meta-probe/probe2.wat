;; probe: defclause with {:restricted-to [...]} called from an ALLOWED prefix — must check clean.
(:wat::core::defclause :probe::guarded
  {:restricted-to [:probe::]}
  ([a <- :wat::core::i64] -> :wat::core::i64 a))

(:wat::core::defn :probe::caller
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:probe::guarded x))
