;; Fixture: spawn-program' :thread against Thread'<i64,i64> annotation type-checks.
(:wat::core::defn :user::mk-echo-peer [] -> :wat::kernel::Thread'<wat::core::i64,wat::core::i64>
  (:wat::kernel::spawn-program' (:wat::spawn::thread)
    (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
      (:wat::kernel::send' self (:wat::kernel::recv' self)))))
