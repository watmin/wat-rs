;; struct_restricted_ctor_denied_bad.wat — caller outside whitelist tries ctor. Must FAIL.
(:wat::core::defstruct :my::Token
  {:restricted-to [:my::issuer::]}
  [id <- :wat::core::i64])
(:wat::core::defn :user::bad-mint [] -> :my::Token
  (:my::Token/new 7))
