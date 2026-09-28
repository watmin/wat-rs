(:wat::core::defn :user::c01 [] -> wat.type/String
  (:wat::string::subs "hello world" 0 5))
(:wat::core::defn :user::c02 [] -> wat.type/String
  (:wat::string::subs "hello world" 6 11))
(:wat::core::defn :user::c03 [] -> wat.type/String
  (:wat::string::subs "abc" 1 1))
(:wat::core::defn :user::c04 [] -> (wat.type/Vector :- [wat.type/String])
  (:wat::io::list-dir "wat"))
