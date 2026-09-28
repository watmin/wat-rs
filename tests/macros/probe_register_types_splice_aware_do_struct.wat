(:wat::core::do
  (:wat::core::defstruct :diag::Point
    [x <- wat.type/i64
     y <- wat.type/i64])
  (:wat::core::defn :diag::origin [] -> :diag::Point (:diag::Point :x 0 :y 0)))
