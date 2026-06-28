;; struct_restricted_field_denied_c_bad.wat — outsider cannot read data field. Must FAIL.
(:wat::core::defstruct :my::Secret
  {:restricted-to  [:my::internal::]
   :field-metadata {:data {:restricted-to [:my::internal::]}}}
  [data <- :wat::core::i64])
(:wat::core::defn :user::outsider::get-data
  [s <- :my::Secret] -> :wat::core::i64
  (:my::Secret/data s))
