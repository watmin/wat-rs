;; struct_restricted_all_restricted.wat — ctor + all fields restricted to same namespace.
(:wat::core::defstruct :my::Secret
  {:restricted-to  [:my::internal::]
   :field-metadata {:data {:restricted-to [:my::internal::]}}}
  [data <- :wat::core::i64])
(:wat::core::defn :my::internal::make [] -> :my::Secret
  (:my::Secret :data 0))
(:wat::core::defn :my::internal::get-data
  [s <- :my::Secret] -> :wat::core::i64
  (:my::Secret/data s))
