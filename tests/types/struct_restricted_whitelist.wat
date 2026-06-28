;; struct_restricted_whitelist.wat — defstruct with :restricted-to + :field-metadata compiles cleanly.
(:wat::core::defstruct :my::Token
  {:restricted-to  [:my::issuer::]
   :field-metadata {:secret {:restricted-to [:my::issuer::]}}}
  [secret <- :wat::core::i64
   id     <- :wat::core::i64])
(:wat::core::defn :my::issuer::mint [] -> :my::Token
  (:my::Token/new 42 99))
(:wat::core::defn :my::issuer::get-secret
  [tok <- :my::Token] -> :wat::core::i64
  (:my::Token/secret tok))
(:wat::core::defn :any::caller::read-id
  [tok <- :my::Token] -> :wat::core::i64
  (:my::Token/id tok))
