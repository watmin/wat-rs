;; probe: an unexpected extra form (neither a metadata-map, `-> :T` sugar, nor a valid
;; clause) in the position right after the name — must ALSO produce a LOCATED error at
;; the definition site, never a downstream unresolved-reference at a call site.
(:wat::core::defclause :probe::guarded
  :not-a-clause-or-metadata
  ([a <- :wat::core::i64] -> :wat::core::i64 a))

(:wat::core::defn :user::caller
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:probe::guarded x))
