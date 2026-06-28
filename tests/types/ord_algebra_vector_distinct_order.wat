;; ord_algebra_vector_distinct_order.wat — two distinct atoms have some order
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [va (:wat::holon::encode (:wat::holon::to-holon "alpha"))
     vb (:wat::holon::encode (:wat::holon::to-holon "omega"))]
    (:wat::core::or (:wat::core::< va vb) (:wat::core::> va vb))))
