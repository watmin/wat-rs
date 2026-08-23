(:wat::core::defn :user::c01 [] -> :wat::core::String
  (:wat::core::string::subs "hello world" 0 5))
(:wat::core::defn :user::c02 [] -> :wat::core::String
  (:wat::core::string::subs "hello world" 6 11))
(:wat::core::defn :user::c03 [] -> :wat::core::String
  (:wat::core::string::subs "abc" 1 1))
(:wat::core::defn :user::c04 [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::io::list-dir "wat"))
