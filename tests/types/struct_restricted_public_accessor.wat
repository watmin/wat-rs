;; struct_restricted_public_accessor.wat — unrestricted field readable from any namespace.
(:wat::core::defstruct :my::Token
  {:restricted-to  [:my::issuer::]
   :field-metadata {:private-field {:restricted-to [:my::issuer::]}}}
  [private-field <- :wat::core::i64
   public-field  <- :wat::core::i64])
(:wat::core::defn :my::issuer::mint [] -> :my::Token
  (:my::Token :private-field 1 :public-field 2))
(:wat::core::defn :totally::different::ns::read-pub
  [tok <- :my::Token] -> :wat::core::i64
  (:my::Token/public-field tok))
