;; Proof 1: the owner is served via the birth-seed (regression guard).
;; A spawned (process) service: autobind a listener (no name — arc 272 capability handoff),
;; send the minted Address' to the owner over the self-peer (birth-seeds allow-set with
;; getppid() = the owner), then poll'-serve echo n+100.
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc  (:wat::test::spawn-peer (:wat::spawn::process)
            (:wat::core::forms
             (:wat::core::defn :user::serve
               [self    <- (:wat::kernel::Peer :- [(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64])
                l       <- (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])
                clients <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])])]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll self l clients) 
                 (:wat::spawn::ServiceEvent::Shutdown nil)
                 ((:wat::spawn::ServiceEvent::Connection peer)
                   (:user::serve self l (:wat::core::conj clients peer)))
                 ((:wat::spawn::ServiceEvent::Message idx n)
                   (:wat::core::let [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth clients idx)
                                          (:wat::core::+ n 100)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                     (:user::serve self l clients)))
                 ((:wat::spawn::ServiceEvent::Closed idx)
                   (:user::serve self l (:wat::seq::remove-at clients idx)))
                 ((:wat::spawn::ServiceEvent::Lost idx _cause)
                   (:user::serve self l (:wat::seq::remove-at clients idx)))
                 ;; Admin wildcard — arc 291 new variant; not exercised by this probe.
                 (_ nil)))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                  self (:wat::program::self-peer
                          (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :wat::core::i64)
                  _    (:wat::core::match (:wat::kernel::send self (:wat::spawn::Bound/address b)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                 (:user::serve self (:wat::spawn::Bound/listener b)
                   (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])]))))))
     ;; recv' the child's minted capability over the lineage channel.
     addr (:wat::core::match (:wat::kernel::recv svc)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped before sending the capability — the peer was ALIVE" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': svc closed before sending the capability" :wat::core::None :wat::core::None)))
     c    (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     _    (:wat::core::match (:wat::kernel::send c 5) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))  ;; arc 278 #73 — the recv' below already faces the stop
     got  (:wat::core::match (:wat::kernel::recv c)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped before replying — the peer was ALIVE" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': c closed before replying" :wat::core::None :wat::core::None)))]
    got))
