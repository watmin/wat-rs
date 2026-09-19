;; Tier B step C battery — door: schemes / typeenv.
;; A user type flows into a stdlib generic (`count` over Vector of user structs).
(:wat::core::defstruct :battery::Point [x <- :wat::core::i64])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [v (:wat::core::Vector :- [:battery::Point] (:battery::Point 1))]
    (:wat::kernel::println (:wat::i64::to-string (:wat::core::count v)))))
