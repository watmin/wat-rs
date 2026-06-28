(:wat::core::defn :user::p2k [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::pascal->kebab s))
(:wat::core::defn :user::k2p [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::kebab->pascal s))
(:wat::core::defn :user::up [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::to-uppercase s))
(:wat::core::defn :user::roundtrip [s <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::kebab->pascal (:wat::core::string::pascal->kebab s)))
