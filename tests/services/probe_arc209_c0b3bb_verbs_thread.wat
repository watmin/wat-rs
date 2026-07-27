;; allow' on a THREAD listener' is a clean error — the crossbeam handle IS the grant.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l    (:wat::spawn::Bound/listener pair)
     _    (:wat::kernel::allow l 123)]
    42))
