;; Negative fixture probe 13: typeunion-typed arg rejects non-member value — must be a CHECK ERROR.
(:wat::core::typeunion :my::IorF [:wat::core::i64 :wat::core::f64])
(:wat::core::defn :my::identity [x <- :my::IorF] -> :my::IorF x)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:my::identity "hello")
    :wat::core::nil))
