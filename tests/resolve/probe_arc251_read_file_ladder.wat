(:wat::core::defn :user::run [path <- :wat::core::String c <- :wat::core::String] -> :wat::core::String
  (:wat::core::do
    (:wat::io::write-file path c)
    (:wat::io::read-file path)))
(:wat::core::defn :user::run2 [path <- :wat::core::String] -> :wat::core::String
  (:wat::io::IOReader/read-all-string (:wat::io::IOReader/open-file path)))
