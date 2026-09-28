;; ord_result_recursion_shallow.wat — Err("alpha") < Err("beta")
(:wat::core::defn :user::err [e <- wat.type/String]
  -> (:wat::core::Result :- [wat.type/i64 wat.type/String])
  (:wat::core::Result.Err {:error e}))
(:wat::core::defn :user::compute [] -> wat.type/bool
  (:wat::core::let
    [a (:user::err "alpha")
     b (:user::err "beta")]
    (:wat::core::< a b)))
