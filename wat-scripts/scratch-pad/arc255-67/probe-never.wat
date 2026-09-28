(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [a (:wat::core::fn [x <- wat.type/Never] -> wat.type/Never x)]
    nil))
