;; ord_result_err_lt_ok.wat — Err < Ok
(:wat::core::defn :user::err [e <- :wat::core::String]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result.Err {:error e}))
(:wat::core::defn :user::ok [n <- :wat::core::i64]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result.Ok {:value n}))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:user::err "boom")
     b (:user::ok 1)]
    (:wat::core::< a b)))
