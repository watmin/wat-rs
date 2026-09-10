;; Parent spawns a process echo service; the child gets its self-peer and echoes owner→child + 100.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                  x    (:wat::core::match (:wat::kernel::recv self)
                         [:wat::kernel::RecvOutcome.Message {:msg m} m]
                         [:wat::kernel::RecvOutcome.Lost {:cause cause}
                           (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                         [:wat::kernel::RecvOutcome.Stopped {}
                           (:wat::kernel::assertion-failed! :message "recv': stopped before the owner sent the value — the peer was ALIVE")]
                         [:wat::kernel::RecvOutcome.Closed {}
                           (:wat::kernel::assertion-failed! :message "recv': self closed before the owner sent the value")])
                  _    (:wat::core::match (:wat::kernel::send self (:wat::core::+ x 100)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])]
                 nil))))
     _   (:wat::core::match (:wat::kernel::send svc 5) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.Closed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Lost {:cause _c} nil])  ;; arc 278 #73 — the recv' below already faces the stop
     got (:wat::core::match (:wat::kernel::recv svc)
           [:wat::kernel::RecvOutcome.Message {:msg m} m]
           [:wat::kernel::RecvOutcome.Lost {:cause cause}
             (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
           [:wat::kernel::RecvOutcome.Stopped {}
             (:wat::kernel::assertion-failed! :message "recv': stopped before echoing back — the peer was ALIVE")]
           [:wat::kernel::RecvOutcome.Closed {}
             (:wat::kernel::assertion-failed! :message "recv': svc closed before echoing back")])]
    got))
