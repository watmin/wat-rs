(:wat::core::defstruct :my::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :my::Point)
                    d1 tx
                    d2 rx]
    nil))
