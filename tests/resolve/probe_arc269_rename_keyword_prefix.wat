(:wat::core::defn :user::go [] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":my::old::Bound" ":my::new::Bound"
    "(:wat::core::let
   ;; KEEP THIS COMMENT byte-identical
   [b (:my::old::Bound/listener x)
    s (:my::old::Bound/address b)]
   b)"))
