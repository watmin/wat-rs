;; structs_builtin_capacity_exceeded.wat — built-in :wat::holon::CapacityExceeded struct.
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [e      (:wat::holon::CapacityExceeded 200 100)
     cost   (:wat::holon::CapacityExceeded/cost e)
     budget (:wat::holon::CapacityExceeded/budget e)]
    (:wat::core::i64::- cost budget)))
