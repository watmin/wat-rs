;; ord_algebra_vector_lt_self_false.wat — v < v is false (assert!(!run_bool))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [v (:wat::holon::encode (:wat::holon::to-holon "x"))]
    (:wat::core::< v v)))
