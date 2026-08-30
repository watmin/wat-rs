;; tests/comms/probe_arc209_c0b1b_select_listener.wat — co-located fixture slurped via startup_beside(file!()).
;; Arc 209 C0b.1b / C0b.2e-i-c — poll' as service multiplexer + (ServiceEvent :- [I O]) sum.

(:wat::core::defenum :user::Op :wat::enum::Pure
  :Compute [n <- :wat::core::i64])

(:wat::core::defn :user::serve
  [self    <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])
   l       <- (:wat::kernel::Listener :- [:user::Op :wat::core::i64])
   clients <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :user::Op])])]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::poll self l clients) 
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection peer)
      (:user::serve self l (:wat::core::conj clients peer)))
    ((:wat::spawn::ServiceEvent::Message idx msg)
      (:wat::core::match msg 
        ((:user::Op::Compute n)
          (:wat::core::let [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth clients idx)
                                 (:wat::core::* n 2)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil))] ;; arc 278 #73 — fire-and-forget reply; outcome ignored uniformly regardless of cause
            (:user::serve self l clients)))))
    ((:wat::spawn::ServiceEvent::Closed idx)
      (:user::serve self l (:wat::seq::remove-at clients idx)))
    ((:wat::spawn::ServiceEvent::Lost idx _cause)
      (:user::serve self l (:wat::seq::remove-at clients idx)))
    (_ nil)))

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener (:wat::spawn::thread) :user::Op :wat::core::i64)
     l    (:wat::spawn::Bound/listener pair)
     addr (:wat::spawn::Bound/address pair)
     svc  (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
              (:user::serve self l (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :user::Op])]))))
     c1   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c1 (:user::Op::Compute 5)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     r1   (:wat::core::match (:wat::kernel::recv c1)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': c1 closed unexpectedly" :wat::core::None :wat::core::None)))
     c2   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c2 (:user::Op::Compute 7)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil) (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     r2   (:wat::core::match (:wat::kernel::recv c2)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': c2 closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:wat::core::+ r1 r2)))

