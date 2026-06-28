(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :wat::kernel::Sender<:wat::core::i64>)
                    d1 tx
                    d2 rx]
    nil))
