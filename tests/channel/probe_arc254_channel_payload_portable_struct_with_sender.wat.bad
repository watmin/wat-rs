(:wat::core::defstruct :my::Capsule [snd <- :wat::kernel::Sender<wat::core::i64>])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [[tx rx] (:wat::kernel::make-channel :my::Capsule)
                    d1 tx
                    d2 rx]
    nil))
