;; ord_result_recursion_shallow.wat — Err("alpha") < Err("beta")
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Result::Err {:error "alpha"})
     b (:wat::core::Result::Err {:error "beta"})]
    (:wat::core::< a b)))
