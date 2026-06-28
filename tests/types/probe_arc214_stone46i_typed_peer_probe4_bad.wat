;; Negative fixture: spawn-program' :thread declared as :i64 return — must be a CHECK ERROR.
(:wat::core::defn :user::mk-wrong [] -> :wat::core::i64
  (:wat::kernel::spawn-program' (:wat::spawn::thread)
    (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
      (:wat::kernel::send' self (:wat::kernel::recv' self)))))
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
