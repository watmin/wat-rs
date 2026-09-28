(:wat::core::defn :user::p2k [s <- wat.type/String] -> wat.type/String
  (:wat::string::pascal->kebab s))
(:wat::core::defn :user::k2p [s <- wat.type/String] -> wat.type/String
  (:wat::string::kebab->pascal s))
(:wat::core::defn :user::up [s <- wat.type/String] -> wat.type/String
  (:wat::string::to-uppercase s))
(:wat::core::defn :user::roundtrip [s <- wat.type/String] -> wat.type/String
  (:wat::string::kebab->pascal (:wat::string::pascal->kebab s)))

;; zero-arg wrappers over fixed literals (no inline-wat driver calls in the .rs).
(:wat::core::defn :user::p2k-get-object [] -> wat.type/String (:user::p2k "GetObject"))
(:wat::core::defn :user::p2k-get [] -> wat.type/String (:user::p2k "Get"))
(:wat::core::defn :user::up-abc [] -> wat.type/String (:user::up "abc"))
(:wat::core::defn :user::k2p-get-object [] -> wat.type/String (:user::k2p "get-object"))
(:wat::core::defn :user::roundtrip-get-object [] -> wat.type/String (:user::roundtrip "GetObject"))
