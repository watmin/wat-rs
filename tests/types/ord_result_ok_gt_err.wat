;; ord_result_ok_gt_err.wat — Ok > Err
(:wat::core::defn :user::err [e <- :wat::core::String]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result.Err {:error e}))
(:wat::core::defn :user::ok [n <- :wat::core::i64]
  -> (:wat::core::Result :- [:wat::core::i64 :wat::core::String])
  (:wat::core::Result.Ok {:value n}))
(:wat::core::defn :user::compute [] -> :wat::core::bool
  (:wat::core::let
    [a (:user::ok 100)
     b (:user::err "anything")]
    (:wat::core::> a b)))
