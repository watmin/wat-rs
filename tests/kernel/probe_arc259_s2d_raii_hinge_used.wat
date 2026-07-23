;; Co-located fixture for probe_arc259_s2d_raii_hinge.rs — peer_used_then_dropped_without_close.
;; Peer used (send -> echo -> recv) then DROPPED without close': RAII reaps; returns 99.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
            (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:wat::core::match (:wat::kernel::recv' self)
              ((:wat::kernel::RecvOutcome::Message m)
                (:wat::core::match (:wat::kernel::send' self m)
                  (:wat::kernel::SendOutcome::Sent nil)
                  (:wat::kernel::SendOutcome::Closed nil)
                  ((:wat::kernel::SendOutcome::Lost _c) nil)))
              ((:wat::kernel::RecvOutcome::Lost cause)
                (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
              (:wat::kernel::RecvOutcome::Closed
                (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None)))))
     _ (:wat::core::match (:wat::kernel::send' peer 99)
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))
     got (:wat::core::match (:wat::kernel::recv' peer)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed before echoing" :wat::core::None :wat::core::None)))]
    got))

