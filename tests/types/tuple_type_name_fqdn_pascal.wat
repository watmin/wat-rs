;; tuple_type_name_fqdn_pascal.wat — type_name returns FQDN PascalCase at runtime.
(:wat::core::defn :my::compute [] -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [t (:wat::core::Tuple 10 20)]
    t))
