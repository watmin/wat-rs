;; typed_if_match_untyped_match_bare.wat — bare match (arc 258.5) is VALID; result inferred = :i64.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::match (:wat::core::Some 1)
    ((:wat::core::Some v) v)
    (:wat::core::None 0)))
