(:wat::core::defn :user::compute [] -> wat.type/i64
  (:wat::core::let
      [holon-rep (:wat::holon::to-holon "not-a-callable-string")]
      (:wat::core::let
        [v (:wat::holon::from-holon holon-rep)]
        (v 1 2))))
