;; typed_if_match_untyped_match_bare.wat — bare match (arc 258.5) is VALID; result inferred = :i64.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Option.Some {:value 1})
    [:wat::core::Option.Some {:value v} v]
    [:wat::core::Option.None {} 0]))
