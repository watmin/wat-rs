;; Stone 255.30 — a pure payload on a thread peer loads and runs.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::thread)
         (:wat::core::fn [self <- (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])]
             -> :wat::core::nil
           (:wat::core::match (:wat::kernel::send self
                                (:wat::core::match (:wat::kernel::recv self)
                                  [:wat::kernel::RecvOutcome.Message {:msg m} m]
                                  [:wat::kernel::RecvOutcome.Lost {:cause c}
                                    (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message c))]
                                  [:wat::kernel::RecvOutcome.Stopped {}
                                    (:wat::kernel::assertion-failed! :message "stopped")]
                                  [:wat::kernel::RecvOutcome.Closed {}
                                    (:wat::kernel::assertion-failed! :message "closed")]))
             [:wat::kernel::SendOutcome.Sent {} nil]
             [:wat::kernel::SendOutcome.Closed {} nil]
             [:wat::kernel::SendOutcome.Stopped {} nil]
             [:wat::kernel::SendOutcome.Lost {:cause _c} nil])))
     _ (:wat::core::match (:wat::kernel::send p 7)
         [:wat::kernel::SendOutcome.Sent {} nil]
         [:wat::kernel::SendOutcome.Closed {} nil]
         [:wat::kernel::SendOutcome.Stopped {} nil]
         [:wat::kernel::SendOutcome.Lost {:cause _c} nil])
     got (:wat::core::match (:wat::kernel::recv p)
           [:wat::kernel::RecvOutcome.Message {:msg m} m]
           [:wat::kernel::RecvOutcome.Lost {:cause c}
             (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message c))]
           [:wat::kernel::RecvOutcome.Stopped {}
             (:wat::kernel::assertion-failed! :message "stopped")]
           [:wat::kernel::RecvOutcome.Closed {}
             (:wat::kernel::assertion-failed! :message "closed")])]
    (:wat::kernel::println got)))
