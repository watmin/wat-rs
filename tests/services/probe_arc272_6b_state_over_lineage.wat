(:wat::core::defrecord :user::Counter [base <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::test::spawn-peer (:wat::spawn::process)
           (:wat::core::forms
             ;; the child runs a FRESH startup (stdlib + these forms only) — the record must be
             ;; defined here too so the crossed Counter reconstructs in the child universe.
             (:wat::core::defrecord :user::Counter [base <- :wat::core::i64])
             ;; the serve loop, now threading `state` (the Counter): reply derives base + n.
             (:wat::core::defn :user::serve
               [self    <- (:wat::kernel::Peer :- [(:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :user::Counter])
                l       <- (:wat::kernel::Listener :- [:wat::core::i64 :wat::core::i64])
                clients <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])])
                state   <- :user::Counter]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll self l clients) 
                 (:wat::spawn::ServiceEvent::Shutdown nil)
                 ((:wat::spawn::ServiceEvent::Connection peer)
                   (:user::serve self l (:wat::core::conj clients peer) state))
                 ;; SERVE — reply base + n, where base lives only in the crossed state0.
                 ((:wat::spawn::ServiceEvent::Message idx n)
                   (:wat::core::let [_ (:wat::core::match (:wat::kernel::send (:wat::core::nth clients idx)
                                          (:wat::core::+ (:user::Counter/base state) n)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                     (:user::serve self l clients state)))
                 ((:wat::spawn::ServiceEvent::Closed idx)
                   (:user::serve self l (:wat::std::list::remove-at clients idx) state))
                 ((:wat::spawn::ServiceEvent::Lost idx _cause)
                   (:user::serve self l (:wat::std::list::remove-at clients idx) state))
                 ;; Admin wildcard — arc 291 new variant; not exercised by this probe.
                 (_ nil)))
             ;; the child entry: autobind (no name), hand the capability up (child→parent, proven),
             ;; then RECEIVE state0 down from the parent over the lineage (parent→child — the gap),
             ;; then serve with it. self-peer: S = Address' (up), R = Counter (down).
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [b    (:wat::kernel::listener (:wat::spawn::process) :wat::core::i64 :wat::core::i64)
                  self (:wat::program::self-peer
                          (:wat::kernel::Address :- [:wat::core::i64 :wat::core::i64]) :user::Counter)
                  _    (:wat::core::match (:wat::kernel::send self (:wat::spawn::Bound/address b)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
                  st   (:wat::core::match (:wat::kernel::recv self)
                         ((:wat::kernel::RecvOutcome::Message m) m)
                         ((:wat::kernel::RecvOutcome::Lost cause)
                           (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                         (:wat::kernel::RecvOutcome::Stopped
                           (:wat::kernel::assertion-failed! "recv': stopped before the owner sent state0 — the peer was ALIVE" :wat::core::None :wat::core::None))
                         (:wat::kernel::RecvOutcome::Closed
                           (:wat::kernel::assertion-failed! "recv': self closed before the owner sent state0" :wat::core::None :wat::core::None)))]
                 (:user::serve self (:wat::spawn::Bound/listener b)
                   (:wat::core::Vector (:wat::kernel::Peer :- [:wat::core::i64 :wat::core::i64])) st)))))
     ;; recv' the child's minted capability over the lineage channel (blocks until the child sends it).
     addr (:wat::core::match (:wat::kernel::recv svc)
            ((:wat::kernel::RecvOutcome::Message m) m)
            ((:wat::kernel::RecvOutcome::Lost cause)
              (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Stopped
              (:wat::kernel::assertion-failed! "recv': stopped before sending the capability — the peer was ALIVE" :wat::core::None :wat::core::None))
            (:wat::kernel::RecvOutcome::Closed
              (:wat::kernel::assertion-failed! "recv': svc closed before sending the capability" :wat::core::None :wat::core::None)))
     ;; hand the child its initial state over the lineage channel (parent→child — the NEW direction).
     _    (:wat::core::match (:wat::kernel::send svc (:user::Counter :base 1000)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))  ;; arc 278 #73 — the recv' below already faces the stop
     ;; dial the capability; round-trip 5 -> base + 5 == 1005 (only if state0 crossed).
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
