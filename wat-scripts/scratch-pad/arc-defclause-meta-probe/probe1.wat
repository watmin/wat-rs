;; probe: defclause with a metadata-map today (pre-Part-1) — does it vanish silently?
(:wat::core::defclause :probe::guarded
  {:restricted-to [:wat::kernel::]}
  ([a <- :wat::core::i64] -> :wat::core::i64 a))

(:wat::core::defn :user::caller
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:probe::guarded x))
