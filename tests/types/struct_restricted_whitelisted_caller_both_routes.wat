;; struct_restricted_whitelisted_caller_both_routes.wat — Arc 198 strike 2 (A1 + B2). A caller
;; INSIDE the ctor whitelist constructs via both the kwargs form and the positional prime.
;; Without this, A1's whitelist-inheritance on `:my::Token'` would be indistinguishable from a
;; total construction ban.
(:wat::core::defstruct :my::Token
  {:restricted-to [:my::issuer::]}
  [id <- :wat::core::i64])
(:wat::core::defn :my::issuer::mint-kwargs [] -> :my::Token
  (:my::Token :id 7))
(:wat::core::defn :my::issuer::mint-prime [] -> :my::Token
  (:my::Token' 9))
