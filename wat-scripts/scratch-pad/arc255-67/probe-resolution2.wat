(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [a (:wat::core::fn [x <- wat.type/Record] -> wat.type/Record x)
     b (:wat::core::fn [x <- wat.type/Struct] -> wat.type/Struct x)]
    nil))
