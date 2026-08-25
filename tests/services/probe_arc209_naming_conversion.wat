(:wat::core::defn :user::p2k [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::pascal->kebab s))
(:wat::core::defn :user::k2p [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::kebab->pascal s))
(:wat::core::defn :user::up [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::to-uppercase s))
(:wat::core::defn :user::roundtrip [s <- :wat::core::String] -> :wat::core::String
  (:wat::string::kebab->pascal (:wat::string::pascal->kebab s)))

;; zero-arg wrappers over fixed literals (no inline-wat driver calls in the .rs).
(:wat::core::defn :user::p2k-get-object [] -> :wat::core::String (:user::p2k "GetObject"))
(:wat::core::defn :user::p2k-get [] -> :wat::core::String (:user::p2k "Get"))
(:wat::core::defn :user::up-abc [] -> :wat::core::String (:user::up "abc"))
(:wat::core::defn :user::k2p-get-object [] -> :wat::core::String (:user::k2p "get-object"))
(:wat::core::defn :user::roundtrip-get-object [] -> :wat::core::String (:user::roundtrip "GetObject"))
