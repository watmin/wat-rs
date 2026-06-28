;; struct_restricted_ctor_only.wat — ctor restricted, no per-field restrictions (all fields public).
(:wat::core::defstruct :my::PublicToken
  {:restricted-to [:my::issuer::]}
  [payload <- :wat::core::i64])
(:wat::core::defn :my::issuer::mint [] -> :my::PublicToken
  (:my::PublicToken/new 42))
(:wat::core::defn :anyone::read
  [tok <- :my::PublicToken] -> :wat::core::i64
  (:my::PublicToken/payload tok))
