;; typed_if_match_match_result_in_let.wat — typed match result flows into enclosing let.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [s
      (:wat::core::match (:wat::core::Option.Some {:value 1}) 
        [:wat::core::Option.Some {:value _} "yes"]
        [:wat::core::Option.None {} "no"])]
    s))
