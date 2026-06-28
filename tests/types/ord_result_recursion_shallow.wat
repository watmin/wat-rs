;; ord_result_recursion_shallow.wat — Err("alpha") < Err("beta")
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:wat::core::Err "alpha")
     b (:wat::core::Err "beta")]
    (:wat::core::< a b)))
