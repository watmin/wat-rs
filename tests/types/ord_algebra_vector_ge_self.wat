;; ord_algebra_vector_ge_self.wat — v >= v is true
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [v (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::>= v v)))
