;; allow'/deny' on a PROCESS listener' succeed (return nil). The allow-set is the SocketListener's.
;; Autobind: (listener' (process) :S :R) → Bound; (Bound/listener b) extracts the Listener'.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [b (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
     l (:wat::spawn::Bound/listener b)
     _ (:wat::kernel::allow l 12345)
     _ (:wat::kernel::deny l 12345)]
    42))
