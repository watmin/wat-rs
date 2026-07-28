;; probe: malformed metadata-map (non-keyword key) on a defclause — must produce a LOCATED
;; error AT THIS DEFINITION, never a downstream "unresolved reference" at a call site.
(:wat::core::defclause :probe::guarded
  {"not-a-keyword" [:wat::kernel::]}
  ([a <- :wat::core::i64] -> :wat::core::i64 a))

(:wat::core::defn :user::caller
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:probe::guarded x))
