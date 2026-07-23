;; typed_if_match_match_result_in_let.wat — typed match result flows into enclosing let.
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [s
      (:wat::core::match (:wat::core::Some 1) 
        ((:wat::core::Some _) "yes")
        (:wat::core::None "no"))]
    s))
