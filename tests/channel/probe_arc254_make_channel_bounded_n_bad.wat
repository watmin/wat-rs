(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-bounded-channel :wat::core::i64 1)
                    d1 tx
                    d2 rx]
    nil))
